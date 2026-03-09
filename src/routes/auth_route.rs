use axum::{Router, routing::post};

use crate::app_state::AppState;

use crate::handlers::auth_handler::{self};

pub fn auth_route(state: AppState) -> Router<AppState> {
    Router::new().route("/login", post(auth_handler::AuthHandler::login_user))
}
