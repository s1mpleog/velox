use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VeloxError {
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
    #[error("validation failed: {0}")]
    ValidationError(String),
    #[error("email service failed")]
    EmailError,
    #[error("token signing failed")]
    JwtError,
    #[error("invalid or expired token")]
    InvalidToken,
    #[error("couldn't find the requested resource")]
    NotFound,
    #[error("route does not exist")]
    RouteNotFound,
    #[error("missing or invalid auth token")]
    AuthError,
    #[error("internal server error")]
    InternalError,
    #[error("missing environment variable: {0}")]
    EnvError(String),
    #[error("Storage quota reached")]
    StorageLimitExceeded,
}

impl IntoResponse for VeloxError {
    fn into_response(self) -> axum::response::Response {
        match &self {
            Self::SqlxError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR"),
            Self::ValidationError(_msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR"),
            Self::EmailError => (StatusCode::INTERNAL_SERVER_ERROR, "EMAIL_ERROR"),
            Self::JwtError => (StatusCode::INTERNAL_SERVER_ERROR, "JWT_ERROR"),
            Self::InvalidToken => (StatusCode::UNAUTHORIZED, "INVALID_TOKEN"),
            Self::NotFound => (StatusCode::NOT_FOUND, "RESOURCE_NOT_FOUND"),
            Self::RouteNotFound => (StatusCode::NOT_FOUND, "ROUTE_NOT_FOUND"),
            Self::AuthError => (StatusCode::UNAUTHORIZED, "AUTH_ERROR"),
            Self::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
            Self::EnvError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "ENV_ERROR"),
            Self::StorageLimitExceeded => (StatusCode::BAD_REQUEST, "StorageLimitExceeded"),
        }
        .into_response()
    }
}
