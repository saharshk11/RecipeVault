use thiserror::Error;

#[derive(Debug, Error)]
pub enum RecipeError {
    #[error("no recipe data found in HTML")]
    NotFound,

    #[error("failed to parse JSON-LD: {0}")]
    JsonLd(#[from] serde_json::Error),

    #[error("invalid URL: {0}")]
    Url(#[from] url::ParseError),

    #[error("HTML parse error: {0}")]
    Html(String)
}