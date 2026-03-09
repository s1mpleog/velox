use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::{app_state::AppState, dto, error::VeloxError, services::auth_service::AuthService};

use validator::Validate;

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
}
