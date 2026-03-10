use sqlx::PgConnection;
use uuid::Uuid;

use crate::{error::VeloxError, models::file_model::File};

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

    pub async fn find_by_id(
        tx: &mut PgConnection,
        file_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Option<File>, VeloxError> {
        let file = sqlx::query_as::<_, File>(
            "SELECT * FROM files WHERE id = $1 AND owner = $2
        ",
        )
        .bind(file_id)
        .bind(user_id)
        .fetch_optional(tx)
        .await?;

        Ok(file)
    }
}
