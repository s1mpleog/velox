use axum::{Router, middleware, routing::post};

use crate::{
    app_state::AppState,
    handlers::folder_handler::FolderHandler,
    middlewares::auth_middleware::{self},
};

pub fn folder_route() -> Router<AppState> {
    Router::new()
        .route("/create", post(FolderHandler::create_folder))
        .layer(middleware::from_fn(auth_middleware::auth_middleware))
}
