use sqlx::PgConnection;

use crate::error::VeloxError;

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
}
