use std::os::linux::raw::stat;

use axum::{
    Router,
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post}
};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use recipe_core::{extract_recipe, Recipe, RecipeError};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone)]
struct AppState {
    http: reqwest::Client
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

#[tokio::main]
async fn main() {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("Mozilla/5.0 (compatible; RecipeWebsite/0.1)")
    );

    let http = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .expect("failed to build reqwest client");

    let state = AppState { http };

    let app = Router::new()
        .route("/health", get(health))
        .route("/parse", post(parse))
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

    let html = state
        .http
        .get(base_url.clone())
        .send()
        .await
        .map_err(ApiError::upstream)?
        .text()
        .await
        .map_err(ApiError::upstream)?;

    let recipe = extract_recipe(&html, &base_url).map_err(ApiError::from_core)?;

    Ok(Json(recipe))
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

    fn upstream<E: std::fmt::Display>(e: E) -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            code: "UPSTREAM_FETCH_FAILED",
            message: format!("Failed to fetch remote page: {e}"),
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
