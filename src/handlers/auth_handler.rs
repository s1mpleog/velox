use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{app_state::AppState, dto, error::VeloxError, services::auth_service::AuthService};

use validator::Validate;

#[derive(Debug, Deserialize)]
pub struct AuthorizeQuery {
    pub token: String,
}

pub struct AuthHandler {}

impl AuthHandler {
    pub async fn login_user(
        State(state): State<AppState>,
        Json(body): Json<dto::auth_dto::LoginRequest>,
    ) -> Result<impl IntoResponse, VeloxError> {
        body.validate()
            .map_err(|e| VeloxError::ValidationError(e.to_string()))?;

        AuthService::login(&state.pool, &body.email).await?;

        Ok((StatusCode::OK, "user logged in successfully"))
    }

    pub async fn authorize(
        Query(params): Query<AuthorizeQuery>,
        State(state): State<AppState>,
    ) -> Result<impl IntoResponse, VeloxError> {
        tracing::info!("got token as: {}", params.token);

        AuthService::authorize(&state.pool, &params.token).await?;

        Ok((StatusCode::OK, "user authorized successfully"))
    }
}
