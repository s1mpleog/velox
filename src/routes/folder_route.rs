use axum::{
    Router, middleware,
    routing::{delete, patch, post},
};

use crate::{
    app_state::AppState,
    handlers::folder_handler::FolderHandler,
    middlewares::auth_middleware::{self},
};

pub fn folder_route() -> Router<AppState> {
    Router::new()
        .route("/create", post(FolderHandler::create_folder))
        .route("/{id}/rename", patch(FolderHandler::rename))
        .route("/{id}", delete(FolderHandler::delete))
        .layer(middleware::from_fn(auth_middleware::auth_middleware))
}
