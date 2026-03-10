use axum::{
    Extension,
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{app_state::AppState, error::VeloxError, services::file_service::FileService};

pub struct FileHandler {}

impl FileHandler {
    pub async fn upload(
        State(state): State<AppState>,
        Extension(user_email): Extension<String>,
        mut multipart: Multipart,
    ) -> Result<impl IntoResponse, VeloxError> {
        let mut files: Vec<(String, String, Vec<u8>)> = Vec::new();
        let mut folder_id: Option<Uuid> = None;

        tracing::debug!("reached here?");

        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|_| VeloxError::InternalError)?
        {
            let name = field.name().unwrap_or("").to_string();
            let file_name = field.file_name().unwrap_or("unknown").to_string();
            let content_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();

            tracing::info!("before data");

            let data = field.bytes().await.map_err(|_| VeloxError::InternalError)?;

            tracing::info!("after data");

            if name == "file" {
                files.push((file_name, content_type, data.to_vec()));
            } else if name == "folder_id" {
                let folder_id_str = String::from_utf8(data.to_vec())
                    .map_err(|_| VeloxError::ValidationError("invalid folder_id".to_string()))?;

                folder_id = Uuid::parse_str(&folder_id_str)
                    .map_err(|_| VeloxError::ValidationError("invalid folder_id".to_string()))
                    .ok();
            }
        }

        tracing::debug!("after loop before upload service here?");

        FileService::upload(
            &state.pool,
            &state.r2,
            &user_email,
            folder_id.as_ref(),
            files,
        )
        .await?;

        Ok((StatusCode::OK, "File uploaded successfully"))
    }
}
