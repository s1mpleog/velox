use sqlx::PgConnection;
use uuid::Uuid;

use crate::error::VeloxError;

pub struct FileRepository {}

impl FileRepository {
    pub async fn insert(
        tx: &mut PgConnection,
        file_id: &Uuid,
        url: &str,
        file_name: &str,
        user_id: &Uuid,
        folder_id: Option<&Uuid>,
        file_size: i64,
        content_type: &str,
    ) -> Result<(), VeloxError> {
        sqlx::query(
            "INSERT INTO files (id, name, owner, url, file_type, size, folder_id) VALUES ($1, $2, $3, $4, $5, $6, $7)"
        ).bind(file_id).bind(file_name).bind(user_id).bind(url).bind(content_type).bind(file_size).bind(folder_id.as_ref()).execute(tx).await?;

        Ok(())
    }
}
