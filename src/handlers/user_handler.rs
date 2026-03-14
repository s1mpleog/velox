use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};
use uuid::Uuid;

use crate::{app_state::AppState, error::VeloxError, services::user_service::UserService};

pub struct UserHandler {}

impl UserHandler {
    pub async fn me(
        State(state): State<AppState>,
        Extension(user_id): Extension<Uuid>,
    ) -> Result<impl IntoResponse, VeloxError> {
        let user = UserService::get_current(&state.pool, &user_id).await?;

        Ok((StatusCode::OK, Json(user)))
    }
}
