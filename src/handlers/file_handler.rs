use axum::{
    Extension, Json,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
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

            let data = field.bytes().await.map_err(|_| VeloxError::InternalError)?;

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

    pub async fn download(
        State(state): State<AppState>,
        Extension(user_email): Extension<String>,
        Path(file_id): Path<Uuid>,
    ) -> Result<impl IntoResponse, VeloxError> {
        let presigned =
            FileService::download(&state.pool, &state.r2, &user_email, &file_id).await?;
        let url = Json(json!({"url": presigned.uri().to_string() }));
        Ok((StatusCode::OK, url))
    }
}
