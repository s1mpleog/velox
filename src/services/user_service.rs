use sqlx::{Pool, Postgres};

use crate::{
    error::VeloxError, models::user_model::User, repositories::user_repository::UserRepository,
};

pub struct UserService {}

impl UserService {
    pub async fn get_current(pool: &Pool<Postgres>, email: &str) -> Result<User, VeloxError> {
        let mut tx = pool.begin().await.map_err(VeloxError::SqlxError)?;
        let user = UserRepository::find_by_email(&mut tx, email)
            .await?
            .ok_or(VeloxError::NotFound)?;

        tracing::info!("found user");

        Ok(user)
    }
}
