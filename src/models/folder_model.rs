use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow, Validate)]
pub struct Folder {
    pub id: Uuid,
    #[validate(length(min = 3, max = 50, message = "invalid folder name"))]
    pub name: String,
    pub user_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
