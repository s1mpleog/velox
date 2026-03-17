use uuid::Uuid;

use crate::{
    error::VeloxError, models::user_model::User,
    repositories::traits::user_repository::UserRepository,
};

pub struct UserService {}

impl UserService {
    pub async fn get_current(
        user_repo: &dyn UserRepository,
        user_id: &Uuid,
    ) -> Result<User, VeloxError> {
        let user = user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(VeloxError::NotFound)?;

        tracing::info!("found user");

        Ok(user)
    }

    pub async fn find_by_id(
        user_repo: &dyn UserRepository,
        user_id: &Uuid,
    ) -> Result<User, VeloxError> {
        let user = user_repo
            .find_by_id(&user_id)
            .await?
            .ok_or(VeloxError::NotFound)?;
        Ok(user)
    }
}
