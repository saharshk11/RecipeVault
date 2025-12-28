mod database;
mod error;

use axum::{
    Router,
    extract::{Json, Path, State},
    http::StatusCode,
    routing::{get, post, patch}
};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use futures_util::StreamExt;
use sqlx::{SqlitePool, Row};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use recipe_core::{extract_recipe, Recipe};
use error::ApiError;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;
use url::Url;
use std::time::Duration;
use std::str::FromStr;

#[derive(Clone)]
struct AppState {
    http: reqwest::Client,
    db: SqlitePool,
}

#[derive(Deserialize)]
struct ParseRequest {
    url: String,
}

#[derive(Deserialize)]
struct ImportRecipeRequest {
    url: String,
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
    updated_at: String,
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
    tags: Option<Vec<String>>
}

const MAX_HTML_BYTES: usize = 2 * 1024 * 1023; // 2 MiB


#[tokio::main]
async fn main() {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("Mozilla/5.0 (compatible; RecipeWebsite/0.1)")
    );

    let http = reqwest::Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(15))
        .build()
        .expect("failed to build reqwest client");

    let options = SqliteConnectOptions::from_str("sqlite:./dev.db")
        .expect("bad sqlite options")
        .create_if_missing(true);

    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .expect("failed to connect to database");

    database::init_db(&db).await.expect("failed to init db");
    let state = AppState { http, db };

    let app = Router::new()
        .route("/health", get(health))
        .route("/parse", post(parse))
        .route("/recipes", get(list_recipes))
        .route("/recipes/import", post(import_recipe))
        .route("/recipes/{id}", get(get_recipe))
        .route("/recipes/{id}", patch(patch_recipe))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();

    println!("listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str {
    "ok"
}

async fn parse(
    State(state): State<AppState>,
    Json(req): Json<ParseRequest>
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
    State(state): State<AppState>
) -> Result<axum::Json<Vec<RecipeListItem>>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT id, title, source_url, image_url, updated_at
        FROM recipes
        ORDER BY updated_at DESC
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
        .map(|row| RecipeListItem {
            id: row.get::<String, _>("id"),
            title: row.get::<String, _>("title"),
            source_url: row.get::<String, _>("source_url"),
            image_url: row.get::<Option<String>, _>("image_url"),
            updated_at: row.get::<String, _>("updated_at"),
        })
        .collect();

    Ok(axum::Json(items))
}

async fn import_recipe(
    State(state): State<AppState>,
    Json(req): Json<ImportRecipeRequest>
) -> Result<Json<ImportRecipeResponse>, ApiError> {
    let base_url = Url::parse(&req.url).map_err(ApiError::bad_request)?;

    // Fetch HTML
    let resp = state
        .http
        .get(base_url.clone())
        .send()
        .await
        .map_err(ApiError::upstream)?;

    let html = resp.text().await.map_err(ApiError::upstream)?;

    // Parse recipe bia recipe-core
    let recipe = extract_recipe(&html, &base_url).map_err(ApiError::from_core)?;

    // Serialize full recipe payload to JSON for storage
    let recipe_json = serde_json::to_string(&recipe)
        .map_err(|e| ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "SERIALIZATION_ERROR",
            message: e.to_string()
        })?;
    
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();

    // Upsert into DB by source_url
    sqlx::query(
        r#"
        INSERT INTO recipes (id, source_url, title, image_url, recipe_json, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ON CONFLICT(source_url) DO UPDATE SET
            title = excluded.title,
            image_url = excluded.image_url,
            recipe_json = excluded.recipe_json,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(&id)
    .bind(base_url.as_str())
    .bind(&recipe.title)
    .bind(recipe.image_url.as_ref().map(|u| u.as_str()))
    .bind(&recipe_json)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        code: "DB_WRITE_FAILED",
        message: e.to_string(),
    })?;

    // Select stored row
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
    // Fetch row by id
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
    // 1) Fetch current row
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

    // Stored payloads
    let recipe_json: String = row.get("recipe_json");
    let mut recipe: Recipe = serde_json::from_str(&recipe_json).map_err(ApiError::serialization)?;

    // Some DBs may store tags as NULL if older schema existed; be defensive
    let tags_json: Option<String> = row.get("tags");
    let mut tags: Vec<String> = match tags_json {
        Some(s) => serde_json::from_str(&s).unwrap_or_default(),
        None => Vec::new(),
    };

    let mut title: String = row.get("title");

    // 2) Apply patch fields
    if let Some(new_title) = &patch.title {
        title = new_title.clone();
        recipe.title = title.clone();
    }

    if let Some(new_tags) = &patch.tags {
        tags = new_tags.clone();
        recipe.tags = tags.clone();
    }

    // If nothing to update, just return current state (optional behavior)
    // (You can remove this if you prefer always writing updated_at.)
    if patch.title.is_none() && patch.tags.is_none() {
        return Ok(Json(GetRecipeResponse {
            id: stored_id,
            recipe,
            created_at,
            updated_at: row.get("updated_at"),
        }));
    }

    // 3) Write updated row
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

    // 4) Return updated record
    Ok(Json(GetRecipeResponse {
        id: stored_id,
        recipe,
        created_at,
        updated_at,
    }))
}