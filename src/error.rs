use axum::{
    http::StatusCode,
    response::IntoResponse,
};
use recipe_core::RecipeError;
use serde::Serialize;

pub struct ApiError {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    pub error: ErrorBody
}

#[derive(Serialize)]
struct ErrorBody {
    pub code: &'static str,
    pub message: String,
}

impl ApiError {
    pub fn bad_request<E: std::fmt::Display>(e: E) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "BAD_REQUEST",
            message: e.to_string(),
        }
    }

    pub fn upstream(e: reqwest::Error) -> Self {
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

    pub fn upstream_too_large(max_bytes: usize) -> Self {
        Self {
            status: StatusCode::PAYLOAD_TOO_LARGE,
            code: "UPSTREAM_BODY_TOO_LARGE",
            message: format!("Remote page exceeded {max_bytes} bytes"),
        }
    }

    pub fn from_core(e: RecipeError) -> Self {
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

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "RECIPE_NOT_FOUND",
            message: message.into(),
        }
    }

    pub fn db_read(e: sqlx::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "DB_READ_FAILED",
            message: e.to_string(),
        }
    }

    pub fn db_write(e: sqlx::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "DB_WRITE_FAILED",
            message: e.to_string(),
        }
    }

    pub fn serialization(e: serde_json::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "SERIALIZATION_ERROR",
            message: e.to_string(),
        }
    }

    pub fn password_hash(e: argon2::password_hash::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "PASSWORD_HASH_FAILED",
            message: e.to_string(),
        }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "UNAUTHORIZED",
            message: message.into(),
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            code: "FORBIDDEN",
            message: message.into(),
        }
    }

    pub fn password_reset_required() -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            code: "PASSWORD_RESET_REQUIRED",
            message: "Password reset required before accessing recipes".to_string(),
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
