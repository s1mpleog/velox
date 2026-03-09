use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "token_type", rename_all = "lowercase")]
pub enum ResponseType {
    SignIn,
    LogIn,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct MagicToken {
    pub token: String,
    pub email: String,

    pub kind: ResponseType,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}
