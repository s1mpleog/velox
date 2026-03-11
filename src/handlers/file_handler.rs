use axum::{
    Extension, Json,
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use bytes::Bytes;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

use crate::{
    app_state::AppState,
    dto::file_dto::RenameFileRequest,
    error::VeloxError,
    services::file_service::{FileService, FileStream, FileUpload},
};

#[derive(Deserialize)]
pub struct ListFilesQuery {
    pub folder_id: Option<Uuid>,
}

pub struct FileHandler {}

impl FileHandler {
    pub async fn upload(
        State(state): State<AppState>,
        Extension(user_email): Extension<String>,
        mut multipart: Multipart,
    ) -> Result<impl IntoResponse, VeloxError> {
        let mut files: Vec<FileUpload> = Vec::new();
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

            if name == "file" {
                let mut buffer: Vec<u8> = Vec::new();
                let mut field = field;

                while let Some(chunk) =
                    field.chunk().await.map_err(|_| VeloxError::InternalError)?
                {
                    buffer.extend_from_slice(&chunk);
                }
                let stream: FileStream = Box::pin(futures_util::stream::once(async move {
                    Ok::<Bytes, VeloxError>(Bytes::from(buffer))
                }));
                files.push(FileUpload {
                    file_name,
                    content_type,
                    stream,
                });
            } else if name == "folder_id" {
                let data = field.bytes().await.map_err(|_| VeloxError::InternalError)?;
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

    pub async fn get_all(
        State(state): State<AppState>,
        Extension(user_email): Extension<String>,
        Query(params): Query<ListFilesQuery>,
    ) -> Result<impl IntoResponse, VeloxError> {
        let files =
            FileService::get_all(&state.pool, &user_email, params.folder_id.as_ref()).await?;

        let response = Json(json!({"files": files}));

        Ok((StatusCode::OK, response))
    }

    pub async fn delete(
        State(state): State<AppState>,
        Extension(user_email): Extension<String>,
        Path(file_id): Path<Uuid>,
    ) -> Result<impl IntoResponse, VeloxError> {
        FileService::delete(&state.pool, &state.r2, &user_email, &file_id).await?;

        Ok((StatusCode::OK, "File deleted successfully"))
    }

    pub async fn rename(
        State(state): State<AppState>,
        Extension(user_email): Extension<String>,
        Path(file_id): Path<Uuid>,
        Json(body): Json<RenameFileRequest>,
    ) -> Result<impl IntoResponse, VeloxError> {
        body.validate()
            .map_err(|e| VeloxError::ValidationError(e.to_string()))?;

        FileService::rename(&state.pool, &user_email, &file_id, &body).await?;

        Ok((StatusCode::OK, "File renamed successfully"))
    }
}
