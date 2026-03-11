use axum::{
    Router, middleware,
    routing::{delete, get, patch, post},
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
        .route("/all", get(FolderHandler::find_all))
        .route("/{id}/contents", get(FolderHandler::get_contents))
        .layer(middleware::from_fn(auth_middleware::auth_middleware))
}
