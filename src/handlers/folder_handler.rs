use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;
use validator::Validate;

use serde_json::json;

use crate::{
    app_state::AppState,
    dto::folder_dto::{CreateFolderRequest, RenameFolderRequest},
    error::VeloxError,
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

    pub async fn rename(
        State(state): State<AppState>,
        Path(folder_id): Path<Uuid>,
        Extension(user_email): Extension<String>,
        Json(body): Json<RenameFolderRequest>,
    ) -> Result<impl IntoResponse, VeloxError> {
        tracing::info!("folder id: {}", folder_id);
        body.validate()
            .map_err(|e| VeloxError::ValidationError(e.to_string()))?;

        FolderService::rename(&state.pool, &folder_id, &user_email, &body).await?;

        Ok((StatusCode::OK, "Folder rename successfully"))
    }

    pub async fn delete(
        State(state): State<AppState>,
        Path(folder_id): Path<Uuid>,
        Extension(user_email): Extension<String>,
    ) -> Result<impl IntoResponse, VeloxError> {
        FolderService::delete(&state.pool, &user_email, &folder_id).await?;
        Ok((StatusCode::OK, "Folder deleted successfully"))
    }

    pub async fn find_all(
        State(state): State<AppState>,
        Extension(user_email): Extension<String>,
    ) -> Result<impl IntoResponse, VeloxError> {
        let folders = FolderService::get_all(&state.pool, &user_email).await?;

        let response = Json(json!({"folders": folders}));

        Ok((StatusCode::OK, response))
    }

    pub async fn get_root_contents(
        State(state): State<AppState>,
        Extension(user_email): Extension<String>,
    ) -> Result<impl IntoResponse, VeloxError> {
        let contents = FolderService::get_root_contents(&state.pool, &user_email).await?;
        Ok((StatusCode::OK, Json(contents)))
    }

    pub async fn get_contents(
        State(state): State<AppState>,
        Path(folder_id): Path<Uuid>,
        Extension(user_email): Extension<String>,
    ) -> Result<impl IntoResponse, VeloxError> {
        let contents = FolderService::get_contents(&state.pool, &user_email, &folder_id).await?;

        let response = Json(json!(contents));

        Ok((StatusCode::OK, response))
    }
}
