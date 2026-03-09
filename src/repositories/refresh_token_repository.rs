use sqlx::PgConnection;
use uuid::Uuid;

use crate::error::VeloxError;

pub struct RefreshTokenRepository {}

impl RefreshTokenRepository {
    pub async fn insert(
        tx: &mut PgConnection,
        token: &str,
        user_id: &Uuid,
    ) -> Result<(), VeloxError> {
        sqlx::query("INSERT INTO refresh_tokens (token, user_id) VALUES ($1, $2)")
            .bind(token)
            .bind(user_id)
            .execute(tx)
            .await
            .map_err(VeloxError::SqlxError)?;

        Ok(())
    }
}
