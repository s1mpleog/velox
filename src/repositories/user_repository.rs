use sqlx::PgConnection;
use uuid::Uuid;

use crate::{error::VeloxError, models::user_model::User};

pub struct UserRepository {}

impl UserRepository {
    pub async fn find_by_email(
        tx: &mut PgConnection,
        email: &str,
    ) -> Result<Option<User>, VeloxError> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(tx)
            .await
            .map_err(VeloxError::SqlxError)
    }

    pub async fn insert(tx: &mut PgConnection, email: &str) -> Result<User, VeloxError> {
        sqlx::query_as::<_, User>("INSERT INTO users (email) VALUES ($1) RETURNING *")
            .bind(email)
            .fetch_one(tx)
            .await
            .map_err(VeloxError::SqlxError)
    }

    pub async fn find_by_id(tx: &mut PgConnection, id: &Uuid) -> Result<Option<User>, VeloxError> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(tx)
            .await
            .map_err(VeloxError::SqlxError)
    }
}
