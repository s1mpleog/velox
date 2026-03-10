use std::time::Duration;

use aws_sdk_s3::presigning::{PresignedRequest, PresigningConfig};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{
    error::VeloxError,
    repositories::{file_repository::FileRepository, user_repository::UserRepository},
    storage,
    utils::Utils,
};

pub struct FileService {}

impl FileService {
    pub async fn upload(
        pool: &Pool<Postgres>,
        r2_client: &aws_sdk_s3::Client,
        user_email: &str,
        folder_id: Option<&Uuid>,
        files: Vec<(String, String, Vec<u8>)>,
    ) -> Result<(), VeloxError> {
        let mut tx = pool.begin().await.map_err(VeloxError::SqlxError)?;
        let user = UserRepository::find_by_email(&mut tx, user_email)
            .await?
            .ok_or(VeloxError::NotFound)?;

        let endpoint = Utils::load_env("R2_ENDPOINT")?;
        let bucket = Utils::load_env("R2_BUCKET_NAME")?;

        for (file_name, content_type, data) in files {
            let file_id = Uuid::new_v4();
            let key = format!("{}/{}/{}", user.id, file_id, file_name);
            let size = data.len() as i64;
            let url = format!("{}/{}/{}", endpoint, bucket, key);
            storage::upload(&r2_client, &key, &content_type, data).await?;
            tracing::info!("uploaded_url: {url}");

            FileRepository::insert(
                &mut tx,
                &file_id,
                &url,
                &file_name,
                &user.id,
                folder_id,
                size,
                &content_type,
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
}
