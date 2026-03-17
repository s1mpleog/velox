use sqlx::PgPool;

use async_trait::async_trait;

use crate::{
    error::VeloxError, models::user_model::User,
    repositories::traits::user_repository::UserRepository,
};

use uuid::Uuid;

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, VeloxError> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(VeloxError::SqlxError)
    }

    async fn insert(&self, email: &str) -> Result<User, VeloxError> {
        sqlx::query_as::<_, User>("INSERT INTO users (email) VALUES ($1) RETURNING *")
            .bind(email)
            .fetch_one(&self.pool)
            .await
            .map_err(VeloxError::SqlxError)
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>, VeloxError> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(VeloxError::SqlxError)
    }
}
