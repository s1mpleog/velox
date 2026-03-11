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

    pub async fn find_all(
        tx: &mut PgConnection,
        user_id: &Uuid,
        folder_id: Option<&Uuid>,
    ) -> Result<Vec<File>, VeloxError> {
        let files = sqlx::query_as::<_, File>(
            "SELECT * FROM files WHERE owner = $1 AND folder_id IS NOT DISTINCT FROM $2
            ",
        )
        .bind(user_id)
        .bind(folder_id)
        .fetch_all(tx)
        .await?;

        Ok(files)
    }

    pub async fn find_by_folder_id(
        tx: &mut PgConnection,
        user_id: &Uuid,
        folder_id: &Uuid,
    ) -> Result<Vec<File>, VeloxError> {
        let files = sqlx::query_as::<_, File>(
            "SELECT * FROM files WHERE owner = $1 AND folder_id = $2
              ",
        )
        .bind(user_id)
        .bind(folder_id)
        .fetch_all(tx)
        .await?;

        Ok(files)
    }

    pub async fn delete(
        tx: &mut PgConnection,
        user_id: &Uuid,
        file_id: &Uuid,
    ) -> Result<Option<String>, VeloxError> {
        let url = sqlx::query_scalar::<_, String>(
            "DELETE FROM files WHERE id = $1 AND owner = $2 RETURNING url",
        )
        .bind(file_id)
        .bind(user_id)
        .fetch_optional(tx)
        .await?;

        Ok(url)
    }

    pub async fn rename(
        tx: &mut PgConnection,
        new_name: &str,
        user_id: &Uuid,
        file_id: &Uuid,
    ) -> Result<(), VeloxError> {
        let query = sqlx::query("UPDATE files SET name = $1 WHERE id = $2 AND owner = $3")
            .bind(new_name)
            .bind(file_id)
            .bind(user_id)
            .execute(tx)
            .await?;

        if query.rows_affected() == 0 {
            return Err(VeloxError::NotFound);
        }

        Ok(())
    }

    pub async fn get_total_storage(
        tx: &mut PgConnection,
        user_id: &Uuid,
    ) -> Result<i64, VeloxError> {
        let total_used = sqlx::query_scalar::<_, i64>(
            "SELECT COALESCE(SUM(size), 0)::BIGINT FROM files WHERE owner = $1",
        )
        .bind(user_id)
        .fetch_one(tx)
        .await?;

        Ok(total_used)
    }
}
