use axum::middleware;
use axum::{Router, routing::get, routing::post};

use crate::app_state::AppState;

use crate::handlers::auth_handler::AuthHandler;
use crate::middlewares::auth_middleware;

pub fn auth_route(_state: AppState) -> Router<AppState> {
    let public = Router::new()
        .route("/login", post(AuthHandler::login_user))
        .route("/authorize", get(AuthHandler::authorize));

    let protected = Router::new()
        .route("/refresh", get(AuthHandler::refresh))
        .route("/logout", post(AuthHandler::logout))
        .route_layer(middleware::from_fn(auth_middleware::auth_middleware));

    public.merge(protected)
}
