use axum::{
    Router,
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post}
};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use futures_util::StreamExt;
use sqlx::{SqlitePool, Row};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use recipe_core::{extract_recipe, Recipe, RecipeError};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;
use url::Url;
use std::time::Duration;
use std::str::FromStr;

mod database;

#[derive(Clone)]
struct AppState {
    http: reqwest::Client,
    db: SqlitePool,
}

#[derive(Deserialize)]
struct ParseRequest {
    url: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: ErrorBody
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
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
        .route("/recipes/import", post(import_recipe))
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

struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    fn bad_request<E: std::fmt::Display>(e: E) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "BAD_REQUEST",
            message: e.to_string(),
        }
    }

    fn upstream(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            return Self {
                status: StatusCode::GATEWAY_TIMEOUT,
                code: "UPSTREAM_TIMEOUT",
                message: "Timed out while fetching remote page".to_string(),
            };
        }

        Self {
            status: StatusCode::BAD_GATEWAY,
            code: "UPSTREAM_FETCH_FAILED",
            message: format!("Failed to fetch remote page: {e}"),
        }
    }

    fn upstream_too_large(max_bytes: usize) -> Self {
        Self {
            status: StatusCode::PAYLOAD_TOO_LARGE,
            code: "UPSTREAM_BODY_TOO_LARGE",
            message: format!("Remote page exceeded {max_bytes} bytes"),
        }
    }

    fn from_core(e: RecipeError) -> Self {
        match e {
            RecipeError::NotFound => Self {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                code: "NO_RECIPE_FOUND",
                message: "No recipe found at the provided URL".to_string(),
            },
            other => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                code: "RECIPE_PARSE_ERROR",
                message: other.to_string(),
            },
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let body = ErrorResponse {
            error: ErrorBody {
                code: self.code,
                message: self.message,
            },
        };

        (self.status, axum::Json(body)).into_response()
    }
}
