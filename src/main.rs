use axum::{
    Router, extract::Json, http::StatusCode, response::IntoResponse, routing::{get, post}
};
use recipe_core::{extract_recipe, Recipe, RecipeError};
use serde::Deserialize;
use url::Url;


#[derive(Deserialize)]
struct ParseRequest {
    url: String,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health))
        .route("/parse", post(parse));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str {
    "ok"
}

async fn parse(Json(req): Json<ParseRequest>) -> Result<Json<Recipe>, ApiError> {
    let base_url = Url::parse(&req.url).map_err(ApiError::bad_request)?;

    let client = reqwest::Client::new();
    let html = client
        .get(base_url.clone())
        .header("User-Agent", "Mozilla/5.0 (compatible; RecipeWebsite/0.1)")
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
    message: String,
}

impl ApiError {
    fn bad_request<E: std::fmt::Display>(e: E) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: e.to_string()
        }
    }

    fn upstream<E: std::fmt::Display>(e: E) -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            message: format!("upstream fetch failed: {e}"),
        }
    }

    fn from_core(e: RecipeError) -> Self {
        match e {
            RecipeError::NotFound => Self {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                message: "no recipe found at that url".to_string(),
            },
            other => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: other.to_string(),
            },
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (self.status, self.message).into_response()
    }
}
