use sqlx::PgConnection;
use uuid::Uuid;

use crate::{error::VeloxError, models::folder_model::Folder};

pub struct FolderRepository {}

impl FolderRepository {
    pub async fn find_parent_folder(
        tx: &mut PgConnection,
        parent_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Option<Folder>, VeloxError> {
        sqlx::query_as::<_, Folder>("SELECT * from folders WHERE parent_id = $1 AND user_id = $2")
            .bind(parent_id)
            .bind(user_id)
            .fetch_optional(tx)
            .await
            .map_err(VeloxError::SqlxError)
    }

    pub async fn find_by_id(
        tx: &mut PgConnection,
        id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Option<Folder>, VeloxError> {
        sqlx::query_as::<_, Folder>("SELECT * from folders WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .fetch_optional(tx)
            .await
            .map_err(VeloxError::SqlxError)
    }

    pub async fn create(
        tx: &mut PgConnection,
        name: &str,
        user_id: &Uuid,
        parent_id: Option<&Uuid>,
    ) -> Result<(), VeloxError> {
        sqlx::query("INSERT INTO folders (name, user_id, parent_id) VALUES ($1, $2, $3)")
            .bind(name)
            .bind(user_id)
            .bind(parent_id)
            .execute(tx)
            .await
            .map_err(VeloxError::SqlxError)?;

        Ok(())
    }

    pub async fn rename(
        tx: &mut PgConnection,
        new_name: &str,
        user_id: &Uuid,
        folder_id: &Uuid,
    ) -> Result<(), VeloxError> {
        let result = sqlx::query("UPDATE folders SET name = $1 WHERE id = $2 AND user_id = $3")
            .bind(new_name)
            .bind(folder_id)
            .bind(user_id)
            .execute(tx)
            .await
            .map_err(VeloxError::SqlxError)?;

        if result.rows_affected() == 0 {
            return Err(VeloxError::NotFound);
        }

        Ok(())
    }
}
