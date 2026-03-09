use sqlx::PgConnection;

use crate::{error::VeloxError, models};

pub struct MagicTokenRepository {}

impl MagicTokenRepository {
    pub async fn delete_by_email(tx: &mut PgConnection, email: &str) -> Result<(), VeloxError> {
        sqlx::query("DELETE FROM magic_tokens WHERE email = $1")
            .bind(email)
            .execute(&mut *tx)
            .await
            .map_err(VeloxError::SqlxError)?;

        Ok(())
    }

    pub async fn insert(
        tx: &mut PgConnection,
        token: &str,
        email: &str,
        kind: models::magic_token_model::ResponseType,
    ) -> Result<(), VeloxError> {
        sqlx::query("INSERT INTO magic_tokens (token, email, kind) VALUES ($1, $2, $3)")
            .bind(token)
            .bind(email)
            .bind(kind)
            .execute(&mut *tx)
            .await
            .map_err(VeloxError::SqlxError)?;

        Ok(())
    }
}
