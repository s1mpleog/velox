use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware,
    routing::{delete, get, patch, post},
};

use crate::{
    app_state::AppState,
    handlers::file_handler::FileHandler,
    middlewares::auth_middleware::{self},
};

pub fn file_route() -> Router<AppState> {
    Router::new()
        .route("/upload", post(FileHandler::upload))
        // TODO: don't hardcode limit
        .layer(DefaultBodyLimit::max(1024 * 1024 * 100))
        .route("/{id}/download", get(FileHandler::download))
        .layer(middleware::from_fn(auth_middleware::auth_middleware))
}
