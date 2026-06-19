use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use utoipa::ToSchema;

use crate::auth::oidc::AuthSetupError;

pub type Result<T, E = AppError> = std::result::Result<T, E>;

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    error: String,
}

impl ErrorResponse {
    pub fn internal_server_error(message: String) -> axum::response::Response {
        tracing::error!("internal server error: {message}");
        #[cfg(not(debug_assertions))]
        let message = "internal server error";
        Self::error(StatusCode::INTERNAL_SERVER_ERROR, message)
    }

    pub fn not_found() -> axum::response::Response {
        ErrorResponse::error(StatusCode::NOT_FOUND, "not found")
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
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("not found")]
    NotFound,
    #[error("auth config error: {0}")]
    AuthSetup(#[from] AuthSetupError),
    #[error("{0}")]
    InternalServerError(String),
}

impl AppError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    pub fn internal_server_error(message: impl Into<String>) -> Self {
        Self::InternalServerError(message.into())
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
            Self::NotFound => ErrorResponse::not_found(),
            Self::AuthSetup(e) => {
                ErrorResponse::internal_server_error(format!("auth configuration error: {e:?}"))
            }
            Self::Forbidden(message) => ErrorResponse::error(StatusCode::FORBIDDEN, message),
            Self::InternalServerError(message) => ErrorResponse::internal_server_error(message),
        }
    }
}
