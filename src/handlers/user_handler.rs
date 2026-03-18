use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{app_state::AppState, error::VeloxError, services::user_service::UserService};

pub struct UserHandler {}

impl UserHandler {
    pub async fn me(
        State(state): State<AppState>,
        Extension(user_id): Extension<Uuid>,
    ) -> Result<impl IntoResponse, VeloxError> {
        // let user = UserService::get_current(&*state.user_repo, &user_id).await?;
        let user = UserService::get_current(&state.pool, &user_id).await?;
        Ok((StatusCode::OK, Json(user)))
    }

    pub async fn find_by_id(
        State(state): State<AppState>,
        Path(user_id): Path<Uuid>,
    ) -> Result<impl IntoResponse, VeloxError> {
        let user = UserService::find_by_id(&state.pool, &user_id).await?;

        Ok((StatusCode::OK, Json(user)))
    }
}
