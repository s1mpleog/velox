use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::{app_state::AppState, error::VeloxError, services::user_service::UserService};

pub struct UserHandler {}

impl UserHandler {
    pub async fn me(
        State(state): State<AppState>,
        Extension(user_email): Extension<String>,
    ) -> Result<impl IntoResponse, VeloxError> {
        let user = UserService::get_current(&state.pool, &user_email).await?;

        Ok((StatusCode::OK, Json(user)))
    }
}
