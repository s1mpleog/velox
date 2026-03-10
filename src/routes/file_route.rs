use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware,
    routing::{delete, patch, post},
};

use crate::{
    app_state::AppState,
    handlers::file_handler::FileHandler,
    middlewares::auth_middleware::{self},
};

pub fn file_route() -> Router<AppState> {
    Router::new()
        .route("/upload", post(FileHandler::upload))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 100))
        .layer(middleware::from_fn(auth_middleware::auth_middleware))
}
