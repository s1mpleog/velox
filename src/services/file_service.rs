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
}
