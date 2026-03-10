use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};
use validator::Validate;

use crate::{
    app_state::AppState, dto::folder_dto::CreateFolderRequest, error::VeloxError,
    services::folder_service::FolderService,
};

pub struct FolderHandler {}

impl FolderHandler {
    pub async fn create_folder(
        State(state): State<AppState>,
        Extension(user_email): Extension<String>,
        Json(body): Json<CreateFolderRequest>,
    ) -> Result<impl IntoResponse, VeloxError> {
        body.validate()
            .map_err(|e| VeloxError::ValidationError(e.to_string()))?;
        FolderService::create(&state.pool, &body, &user_email).await?;

        Ok((StatusCode::CREATED, "folder created successfully"))
    }
}
