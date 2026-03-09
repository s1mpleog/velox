use sqlx::PgConnection;

use crate::{error::VeloxError, models::temp_user_model::TempUser};

pub struct TempUserRepository {}

impl TempUserRepository {
    pub async fn upsert(tx: &mut PgConnection, email: &str) -> Result<(), VeloxError> {
        sqlx::query("INSERT INTO temp_users (email) VALUES ($1) ON CONFLICT (email) DO NOTHING")
            .bind(email)
            .execute(&mut *tx)
            .await
            .map_err(VeloxError::SqlxError)?;

        Ok(())
    }

    pub async fn find_by_email(
        tx: &mut PgConnection,
        email: &str,
    ) -> Result<Option<TempUser>, VeloxError> {
        sqlx::query_as::<_, TempUser>("SELECT * FROM temp_users WHERE email = $1")
            .bind(email)
            .fetch_optional(&mut *tx)
            .await
            .map_err(VeloxError::SqlxError)
    }

    pub async fn delete_by_email(tx: &mut PgConnection, email: &str) -> Result<(), VeloxError> {
        sqlx::query("DELETE FROM temp_users WHERE email = $1")
            .bind(email)
            .execute(&mut *tx)
            .await
            .map_err(VeloxError::SqlxError)?;

        Ok(())
    }
}
