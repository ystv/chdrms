use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;

pub type Result<T, E = AppError> = std::result::Result<T, E>;

#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
}

impl ErrorResponse {
    fn internal_server_error(message: String) -> axum::response::Response {
        tracing::error!("internal server error: {message}");
        #[cfg(not(debug_assertions))]
        let message = "internal server error";
        Self::error(StatusCode::INTERNAL_SERVER_ERROR, message)
    }

    fn error(status: StatusCode, message: impl Into<String>) -> axum::response::Response {
        (
            status,
            Json(Self {
                error: message.into(),
            }),
        )
            .into_response()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("unauthorized")]
    Unauthorized,
    #[error("bad request: {0}")]
    BadRequest(String),
}

impl AppError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::DatabaseError(e) => {
                ErrorResponse::internal_server_error(format!("database error: {e:?}"))
            }
            Self::Unauthorized => ErrorResponse::error(StatusCode::UNAUTHORIZED, "unauthorized"),
            Self::BadRequest(message) => ErrorResponse::error(StatusCode::BAD_REQUEST, message),
        }
    }
}
