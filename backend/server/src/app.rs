use axum::{
    Extension,
    Router,
    extract::{Json, Path, State},
    http::{StatusCode, Method, HeaderValue, header::CONTENT_TYPE},
    middleware,
    response::Response,
    routing::{get, post, delete, patch},
};
use axum::body::Body;
use axum::http::Request;
use axum::middleware::Next;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use futures_util::StreamExt;
use reqwest::header::{HeaderMap, USER_AGENT};
use sqlx::{SqlitePool, Row};
use recipe_core::{extract_recipe, Recipe};
use crate::error::ApiError;
use crate::auth::{self, AuthUser};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;
use url::Url;
use std::time::Duration;
use tower_http::cors::{AllowOrigin, CorsLayer};

#[derive(Clone)]
pub struct AppState {
    pub http: reqwest::Client,
    pub db: SqlitePool,
}

#[derive(Deserialize)]
struct ParseRequest {
    url: String,
}

#[derive(Deserialize)]
struct ImportRecipeRequest {
    url: String,
    tags: Option<Vec<String>>,
}

#[derive(Serialize)]
struct ImportRecipeResponse {
    id: String,
    recipe: Recipe,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct RecipeListItem {
    id: String,
    title: String,
    source_url: String,
    image_url: Option<String>,
    created_at: String,
    updated_at: String,
    tags: Vec<String>,
}

#[derive(Serialize)]
struct GetRecipeResponse {
    id: String,
    recipe: Recipe,
    created_at: String,
    updated_at: String,
}

#[derive(Deserialize)]
struct PatchRecipeRequest {
    title: Option<String>,
    tags: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct ChangeCredentialsRequest {
    current_password: String,
    new_username: String,
    new_password: String,
}

#[derive(Serialize)]
struct AuthUserResponse {
    id: String,
    username: String,
    role: String,
    must_change_password: bool,
}

#[derive(Serialize)]
struct LoginResponse {
    user: AuthUserResponse,
}

const MAX_HTML_BYTES: usize = 2 * 1024 * 1023; // 2 MiB

pub fn build_app(state: AppState) -> Router {
    let protected_auth_routes = Router::new()
        .route("/auth/logout", post(logout))
        .route("/auth/change-credentials", post(change_credentials))
        .route("/auth/me", get(auth_me))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    let recipe_routes = Router::new()
        .route("/parse", post(parse))
        .route("/recipes", get(list_recipes))
        .route("/recipes/import", post(import_recipe))
        .route("/recipes/{id}", get(get_recipe))
        .route("/recipes/{id}", patch(patch_recipe))
        .route("/recipes/{id}", delete(delete_recipe))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_fresh_auth));

    Router::new()
        .route("/health", get(health))
        .route("/auth/login", post(login))
        .merge(protected_auth_routes)
        .merge(recipe_routes)
        .with_state(state)
        .layer(cors_layer())
}

pub fn default_http_client() -> reqwest::Client {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("Mozilla/5.0 (compatible; RecipeWebsite/0.1)"),
    );

    reqwest::Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(15))
        .build()
        .expect("failed to build reqwest client")
}

fn cors_layer() -> CorsLayer {
    let origins = std::env::var("CORS_ALLOW_ORIGIN")
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty())
                .collect::<Vec<_>>()
        });

    let fallback = vec![
        "http://localhost:5173".to_string(),
        "http://127.0.0.1:5173".to_string(),
    ];

    let origin_values = origins.unwrap_or(fallback);
    let header_values: Vec<HeaderValue> = origin_values
        .iter()
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect();

    let allow_origin = if header_values.is_empty() {
        AllowOrigin::list([
            HeaderValue::from_static("http://localhost:5173"),
            HeaderValue::from_static("http://127.0.0.1:5173"),
        ])
    } else {
        AllowOrigin::list(header_values)
    };

    CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([CONTENT_TYPE])
        .allow_credentials(true)
}

async fn health() -> &'static str {
    "ok"
}

fn to_auth_response(user: &AuthUser) -> AuthUserResponse {
    AuthUserResponse {
        id: user.id.clone(),
        username: user.username.clone(),
        role: user.role.clone(),
        must_change_password: user.must_change_password,
    }
}

fn cookie_secure() -> bool {
    std::env::var("RECIPE_COOKIE_SECURE")
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE"))
        .unwrap_or(false)
}

fn build_session_cookie(session_id: &str) -> Cookie<'static> {
    let mut cookie = Cookie::build((auth::SESSION_COOKIE_NAME, session_id.to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build();

    if cookie_secure() {
        cookie.set_secure(true);
    }

    cookie
}

fn remove_session_cookie(jar: CookieJar) -> CookieJar {
    let cookie = Cookie::build((auth::SESSION_COOKIE_NAME, ""))
        .path("/")
        .build();
    jar.remove(cookie)
}

async fn require_auth(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    let jar = CookieJar::from_headers(req.headers());
    let session_id = jar
        .get(auth::SESSION_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string())
        .ok_or_else(|| ApiError::unauthorized("Authentication required"))?;

    let user = auth::find_user_by_session_id(&state.db, &session_id)
        .await?
        .ok_or_else(|| ApiError::unauthorized("Invalid or expired session"))?;

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

async fn require_fresh_auth(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    let jar = CookieJar::from_headers(req.headers());
    let session_id = jar
        .get(auth::SESSION_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string())
        .ok_or_else(|| ApiError::unauthorized("Authentication required"))?;

    let user = auth::find_user_by_session_id(&state.db, &session_id)
        .await?
        .ok_or_else(|| ApiError::unauthorized("Invalid or expired session"))?;

    if user.must_change_password {
        return Err(ApiError::password_reset_required());
    }

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<LoginRequest>,
) -> Result<(CookieJar, Json<LoginResponse>), ApiError> {
    let user = auth::verify_credentials(&state.db, &req.username, &req.password)
        .await?
        .ok_or_else(|| ApiError::unauthorized("Invalid credentials"))?;

    let session = auth::create_session(&state.db, &user.id, auth::DEFAULT_SESSION_TTL_SECS)
        .await?;

    let jar = jar.add(build_session_cookie(&session.id));

    Ok((
        jar,
        Json(LoginResponse {
            user: to_auth_response(&user),
        }),
    ))
}

async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, StatusCode), ApiError> {
    if let Some(cookie) = jar.get(auth::SESSION_COOKIE_NAME) {
        auth::delete_session(&state.db, cookie.value()).await?;
    }

    Ok((remove_session_cookie(jar), StatusCode::NO_CONTENT))
}

async fn change_credentials(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(req): Json<ChangeCredentialsRequest>,
) -> Result<Json<AuthUserResponse>, ApiError> {
    let valid = auth::verify_user_password(&state.db, &user.id, &req.current_password).await?;
    if !valid {
        return Err(ApiError::unauthorized("Invalid credentials"));
    }

    let updated = auth::update_user_credentials(
        &state.db,
        &user.id,
        &req.new_username,
        &req.new_password,
        &user.role,
        false,
    )
    .await?;

    Ok(Json(to_auth_response(&updated)))
}

async fn auth_me(
    Extension(user): Extension<AuthUser>,
) -> Result<Json<AuthUserResponse>, ApiError> {
    Ok(Json(to_auth_response(&user)))
}

async fn parse(
    State(state): State<AppState>,
    Json(req): Json<ParseRequest>,
) -> Result<Json<Recipe>, ApiError> {
    let base_url = Url::parse(&req.url).map_err(ApiError::bad_request)?;

    let resp = state
        .http
        .get(base_url.clone())
        .send()
        .await
        .map_err(ApiError::upstream)?;

    let mut buf: Vec<u8> = Vec::new();
    let mut stream = resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(ApiError::upstream)?;
        if buf.len() + chunk.len() > MAX_HTML_BYTES {
            return Err(ApiError::upstream_too_large(MAX_HTML_BYTES));
        }
        buf.extend_from_slice(&chunk);
    }

    let html = String::from_utf8_lossy(&buf).to_string();

    let recipe = extract_recipe(&html, &base_url).map_err(ApiError::from_core)?;

    Ok(Json(recipe))
}

async fn list_recipes(
    State(state): State<AppState>,
) -> Result<axum::Json<Vec<RecipeListItem>>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT id, title, source_url, image_url, created_at, updated_at, tags
        FROM recipes
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| ApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        code: "DB_READ_FAILED",
        message: e.to_string(),
    })?;

    let items = rows
        .into_iter()
        .map(|row| {
            let tags_json: Option<String> = row.get("tags");
            let tags: Vec<String> = match tags_json {
                Some(value) => serde_json::from_str(&value).unwrap_or_default(),
                None => Vec::new(),
            };
            RecipeListItem {
                id: row.get::<String, _>("id"),
                title: row.get::<String, _>("title"),
                source_url: row.get::<String, _>("source_url"),
                image_url: row.get::<Option<String>, _>("image_url"),
                created_at: row.get::<String, _>("created_at"),
                updated_at: row.get::<String, _>("updated_at"),
                tags,
            }
        })
        .collect();

    Ok(axum::Json(items))
}

async fn import_recipe(
    State(state): State<AppState>,
    Json(req): Json<ImportRecipeRequest>,
) -> Result<Json<ImportRecipeResponse>, ApiError> {
    let base_url = Url::parse(&req.url).map_err(ApiError::bad_request)?;

    let resp = state
        .http
        .get(base_url.clone())
        .send()
        .await
        .map_err(ApiError::upstream)?;

    let html = resp.text().await.map_err(ApiError::upstream)?;

    let mut recipe = extract_recipe(&html, &base_url).map_err(ApiError::from_core)?;
    let tags = req.tags.unwrap_or_default();
    recipe.tags = tags.clone();

    let recipe_json = serde_json::to_string(&recipe)
        .map_err(|e| ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "SERIALIZATION_ERROR",
            message: e.to_string(),
        })?;
    let tags_json = serde_json::to_string(&tags).map_err(|e| ApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        code: "SERIALIZATION_ERROR",
        message: e.to_string(),
    })?;
    
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"
        INSERT INTO recipes (id, source_url, title, image_url, recipe_json, tags, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(source_url) DO UPDATE SET
            title = excluded.title,
            image_url = excluded.image_url,
            recipe_json = excluded.recipe_json,
            tags = excluded.tags,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(&id)
    .bind(base_url.as_str())
    .bind(&recipe.title)
    .bind(recipe.image_url.as_ref().map(|u| u.as_str()))
    .bind(&recipe_json)
    .bind(&tags_json)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        code: "DB_WRITE_FAILED",
        message: e.to_string(),
    })?;

    let row = sqlx::query(
        r#"
        SELECT id, recipe_json, created_at, updated_at
        FROM recipes
        WHERE source_url = ?1
        "#,
    )
    .bind(base_url.as_str())
    .fetch_one(&state.db)
    .await
    .map_err(|e| ApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        code: "DB_READ_FAILED",
        message: e.to_string(),
    })?;

    let stored_id: String = row.get("id");
    let stored_recipe_json: String = row.get("recipe_json");
    let created_at: String = row.get("created_at");
    let updated_at: String = row.get("updated_at");

    let stored_recipe: Recipe = serde_json::from_str(&stored_recipe_json).map_err(|e| ApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        code: "DB_DESERIALIZATION_ERROR",
        message: e.to_string(),
    })?;

    Ok(Json(ImportRecipeResponse {
        id: stored_id,
        recipe: stored_recipe,
        created_at,
        updated_at,
    }))
}

async fn get_recipe(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<axum::Json<GetRecipeResponse>, ApiError> {
    let row_opt = sqlx::query(
        r#"
        SELECT id, recipe_json, created_at, updated_at
        FROM recipes
        WHERE id = ?1
        "#,
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        code: "DB_READ_FAILED",
        message: e.to_string(),
    })?;

    let row = match row_opt {
        Some(r) => r,
        None => return Err(ApiError::not_found("Recipe not found")),
    };

    let stored_id: String = row.get("id");
    let recipe_json: String = row.get("recipe_json");
    let created_at: String = row.get("created_at");
    let updated_at: String = row.get("updated_at");

    let recipe: Recipe = serde_json::from_str(&recipe_json).map_err(|e| ApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        code: "DB_DESERIALIZATION_ERROR",
        message: e.to_string(),
    })?;

    Ok(axum::Json(GetRecipeResponse {
        id: stored_id,
        recipe,
        created_at,
        updated_at,
    }))
}

async fn patch_recipe(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(patch): Json<PatchRecipeRequest>,
) -> Result<Json<GetRecipeResponse>, ApiError> {
    let row_opt = sqlx::query(
        r#"
        SELECT id, recipe_json, title, tags, created_at, updated_at
        FROM recipes
        WHERE id = ?1
        "#,
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(ApiError::db_read)?;

    let row = match row_opt {
        Some(r) => r,
        None => return Err(ApiError::not_found("Recipe not found")),
    };

    let stored_id: String = row.get("id");
    let created_at: String = row.get("created_at");

    let recipe_json: String = row.get("recipe_json");
    let mut recipe: Recipe = serde_json::from_str(&recipe_json).map_err(ApiError::serialization)?;

    let tags_json: Option<String> = row.get("tags");
    let mut tags: Vec<String> = match tags_json {
        Some(s) => serde_json::from_str(&s).unwrap_or_default(),
        None => Vec::new(),
    };

    let mut title: String = row.get("title");

    if let Some(new_title) = &patch.title {
        title = new_title.clone();
        recipe.title = title.clone();
    }

    if let Some(new_tags) = &patch.tags {
        tags = new_tags.clone();
        recipe.tags = tags.clone();
    }

    if patch.title.is_none() && patch.tags.is_none() {
        return Ok(Json(GetRecipeResponse {
            id: stored_id,
            recipe,
            created_at,
            updated_at: row.get("updated_at"),
        }));
    }

    let updated_at = Utc::now().to_rfc3339();
    let new_recipe_json = serde_json::to_string(&recipe).map_err(ApiError::serialization)?;
    let new_tags_json = serde_json::to_string(&tags).map_err(ApiError::serialization)?;

    sqlx::query(
        r#"
        UPDATE recipes
        SET title = ?1,
            tags = ?2,
            recipe_json = ?3,
            updated_at = ?4
        WHERE id = ?5
        "#,
    )
    .bind(&title)
    .bind(&new_tags_json)
    .bind(&new_recipe_json)
    .bind(&updated_at)
    .bind(&stored_id)
    .execute(&state.db)
    .await
    .map_err(ApiError::db_write)?;

    Ok(Json(GetRecipeResponse {
        id: stored_id,
        recipe,
        created_at,
        updated_at,
    }))
}

async fn delete_recipe(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let result = sqlx::query(
        r#"
        DELETE FROM recipes
        WHERE id = ?1
        "#,
    )
    .bind(&id)
    .execute(&state.db)
    .await
    .map_err(ApiError::db_write)?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("Recipe not found"));
    }

    Ok(StatusCode::NO_CONTENT)
}
