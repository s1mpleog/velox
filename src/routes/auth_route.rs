use axum::{Router, routing::get, routing::post};

use crate::app_state::AppState;

use crate::handlers::auth_handler::{self, AuthHandler};

pub fn auth_route(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/login", post(AuthHandler::login_user))
        .route("/authorize", get(AuthHandler::authorize))
}
