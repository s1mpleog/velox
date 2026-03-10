use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct File {
    pub id: Uuid,
    pub name: String,
    pub owner: Uuid,
    pub url: String,
    pub file_type: String,
    pub size: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
