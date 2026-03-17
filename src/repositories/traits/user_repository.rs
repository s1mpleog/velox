use async_trait::async_trait;
use uuid::Uuid;

use crate::{error::VeloxError, models::user_model::User};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, VeloxError>;
    async fn insert(&self, email: &str) -> Result<User, VeloxError>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>, VeloxError>;
}
