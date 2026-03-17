use std::sync::Arc;

use sqlx::{Pool, Postgres};

use crate::repositories::traits::user_repository::UserRepository;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    pub r2: aws_sdk_s3::Client,
    pub user_repo: Arc<dyn UserRepository>,
}
