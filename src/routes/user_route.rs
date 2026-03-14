use axum::{Router, middleware};

use axum::routing::{get, post};

use crate::app_state::AppState;
use crate::handlers::user_handler::UserHandler;
use crate::middlewares::auth_middleware::auth_middleware;

pub fn user_route() -> Router<AppState> {
    Router::new()
        .route("/me", get(UserHandler::me))
        .layer(middleware::from_fn(auth_middleware))
}
