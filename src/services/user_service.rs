use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{
    error::VeloxError, models::user_model::User, repositories::user_repository::UserRepository,
};

pub struct UserService {}

impl UserService {
    pub async fn get_current(pool: &Pool<Postgres>, user_id: &Uuid) -> Result<User, VeloxError> {
        let mut tx = pool.begin().await.map_err(VeloxError::SqlxError)?;
        let user = UserRepository::find_by_id(&mut tx, user_id)
            .await?
            .ok_or(VeloxError::NotFound)?;

        tx.commit().await.map_err(VeloxError::SqlxError)?;

        Ok(user)
    }

    pub async fn find_by_id(pool: &Pool<Postgres>, user_id: &Uuid) -> Result<User, VeloxError> {
        let mut tx = pool.begin().await.map_err(VeloxError::SqlxError)?;
        let user = UserRepository::find_by_id(&mut tx, &user_id)
            .await?
            .ok_or(VeloxError::NotFound)?;

        tx.commit().await.map_err(VeloxError::SqlxError)?;
        Ok(user)
    }
}
