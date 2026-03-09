use sqlx::PgConnection;

use crate::{error::VeloxError, models::user_model::User};

pub struct UserRepository {}

impl UserRepository {
    pub async fn find_by_email(
        tx: &mut PgConnection,
        email: &str,
    ) -> Result<Option<User>, VeloxError> {
        sqlx::query_as::<_, User>("SELECT email FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(tx)
            .await
            .map_err(VeloxError::SqlxError)
    }
}
