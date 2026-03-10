use sqlx::PgConnection;
use uuid::Uuid;

use crate::{error::VeloxError, models::refresh_token_model::RefreshToken};

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

    pub async fn find_by_token(
        tx: &mut PgConnection,
        token: &str,
    ) -> Result<Option<RefreshToken>, VeloxError> {
        sqlx::query_as::<_, RefreshToken>("SELECT * FROM refresh_tokens WHERE token = $1")
            .bind(token)
            .fetch_optional(tx)
            .await
            .map_err(VeloxError::SqlxError)
    }

    pub async fn delete_by_token(tx: &mut PgConnection, token: &str) -> Result<(), VeloxError> {
        sqlx::query("DELETE FROM refresh_tokens WHERE token = $1")
            .bind(token)
            .execute(tx)
            .await
            .map_err(VeloxError::SqlxError)?;

        Ok(())
    }
}
