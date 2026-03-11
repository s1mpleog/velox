use std::time::Duration;

use aws_sdk_s3::presigning::{PresignedRequest, PresigningConfig};
use bytes::Bytes;
use futures_util::Stream;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use std::pin::Pin;

use crate::{
    dto::file_dto::RenameFileRequest,
    error::VeloxError,
    models::file_model::File,
    repositories::{file_repository::FileRepository, user_repository::UserRepository},
    storage,
    utils::Utils,
};

const GB_IN_NUMS: i64 = 1_073_741_824;

pub type FileStream = Pin<Box<dyn Stream<Item = Result<Bytes, VeloxError>> + Send>>;

pub struct FileUpload {
    pub file_name: String,
    pub content_type: String,
    pub stream: FileStream,
}

pub struct FileService {}

impl FileService {
    pub async fn upload(
        pool: &Pool<Postgres>,
        r2_client: &aws_sdk_s3::Client,
        user_email: &str,
        folder_id: Option<&Uuid>,
        files: Vec<FileUpload>,
    ) -> Result<(), VeloxError> {
        let mut tx = pool.begin().await.map_err(VeloxError::SqlxError)?;
        let user = UserRepository::find_by_email(&mut tx, user_email)
            .await?
            .ok_or(VeloxError::NotFound)?;

        tracing::debug!("files count: {}", files.len());

        // TODO: premium user can get 2GB limit
        let total_used = FileRepository::get_total_storage(&mut tx, &user.id).await?;

        // tracing::error!("{:?}", total_used.as_ref().err());

        let endpoint = Utils::load_env("R2_ENDPOINT")?;
        let bucket = Utils::load_env("R2_BUCKET_NAME")?;

        for file in files {
            tracing::debug!("processing file: {}", file.file_name);
            let file_id = Uuid::new_v4();
            let key = format!("{}/{}/{}", user.id, file_id, file.file_name);
            let url = format!("{}/{}/{}", endpoint, bucket, key);

            let file_size =
                storage::upload_streaming(r2_client, &key, &file.content_type, file.stream).await?;

            // TODO: take care of premium users
            if total_used + file_size > GB_IN_NUMS {
                tx.rollback().await.map_err(VeloxError::SqlxError)?;
                storage::delete_from_r2(&r2_client, &key).await?;
                return Err(VeloxError::StorageLimitExceeded);
            }

            // storage::upload(&r2_client, &key, &content_type, data).await?;
            tracing::info!("uploaded_url: {url}");

            FileRepository::insert(
                &mut tx,
                &file_id,
                &url,
                &file.file_name,
                &user.id,
                folder_id,
                file_size,
                &file.content_type,
            )
            .await?;
        }
        tx.commit().await.map_err(VeloxError::SqlxError)?;
        Ok(())
    }

    pub async fn download(
        pool: &Pool<Postgres>,
        r2_client: &aws_sdk_s3::Client,
        user_email: &str,
        file_id: &Uuid,
    ) -> Result<PresignedRequest, VeloxError> {
        let mut tx = pool.begin().await.map_err(VeloxError::SqlxError)?;

        let user = UserRepository::find_by_email(&mut tx, user_email)
            .await?
            .ok_or(VeloxError::NotFound)?;

        let file = FileRepository::find_by_id(&mut tx, file_id, &user.id)
            .await?
            .ok_or(VeloxError::NotFound)?;

        let endpoint =
            Utils::load_env("R2_ENDPOINT").map_err(|e| VeloxError::EnvError(e.to_string()))?;

        let bucket =
            Utils::load_env("R2_BUCKET_NAME").map_err(|e| VeloxError::EnvError(e.to_string()))?;

        let key = file.url.replace(&format!("{}/{}/", endpoint, bucket), "");

        let presigned = r2_client
            .get_object()
            .bucket(&bucket)
            .key(&key)
            .presigned(
                // TODO: stop hardcoding
                PresigningConfig::expires_in(Duration::from_secs(900))
                    .map_err(|_| VeloxError::InternalError)?,
            )
            .await
            .map_err(|_| VeloxError::InternalError)?;

        tx.commit().await.map_err(VeloxError::SqlxError)?;

        Ok(presigned)
    }

    pub async fn get_all(
        pool: &Pool<Postgres>,
        user_email: &str,
        folder_id: Option<&Uuid>,
    ) -> Result<Vec<File>, VeloxError> {
        let mut tx = pool.begin().await.map_err(VeloxError::SqlxError)?;

        let user = UserRepository::find_by_email(&mut tx, user_email)
            .await?
            .ok_or(VeloxError::NotFound)?;

        let files = FileRepository::find_all(&mut tx, &user.id, folder_id).await?;

        tx.commit().await.map_err(VeloxError::SqlxError)?;
        Ok(files)
    }

    pub async fn delete(
        pool: &Pool<Postgres>,
        r2_client: &aws_sdk_s3::Client,
        user_email: &str,
        file_id: &Uuid,
    ) -> Result<(), VeloxError> {
        let mut tx = pool.begin().await.map_err(VeloxError::SqlxError)?;

        let user = UserRepository::find_by_email(&mut tx, user_email)
            .await?
            .ok_or(VeloxError::NotFound)?;

        let url = FileRepository::delete(&mut tx, &user.id, file_id)
            .await?
            .ok_or(VeloxError::NotFound)?;

        let endpoint =
            Utils::load_env("R2_ENDPOINT").map_err(|e| VeloxError::EnvError(e.to_string()))?;

        let bucket =
            Utils::load_env("R2_BUCKET_NAME").map_err(|e| VeloxError::EnvError(e.to_string()))?;

        let key = url.replace(&format!("{}/{}/", endpoint, bucket), "");

        storage::delete_from_r2(r2_client, &key).await?;

        tx.commit().await.map_err(VeloxError::SqlxError)?;
        Ok(())
    }

    pub async fn rename(
        pool: &Pool<Postgres>,
        user_email: &str,
        file_id: &Uuid,
        request_body: &RenameFileRequest,
    ) -> Result<(), VeloxError> {
        let mut tx = pool.begin().await.map_err(VeloxError::SqlxError)?;

        let user = UserRepository::find_by_email(&mut tx, user_email)
            .await?
            .ok_or(VeloxError::NotFound)?;

        // TODO: handle this later
        // let extension = file.name.rsplit('.').next().unwrap_or("");
        // let new_name = format!("{}.{}", request_body.new_name, extension);

        FileRepository::rename(&mut tx, &request_body.new_name, &user.id, file_id).await?;

        tx.commit().await.map_err(VeloxError::SqlxError)?;
        Ok(())
    }
}
