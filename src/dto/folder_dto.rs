use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct CreateFolderRequest {
    #[validate(length(min = 1, max = 50, message = "invalid folder name"))]
    pub name: String,
    pub parent_id: Option<Uuid>,
}

#[derive(Deserialize, Validate)]
pub struct RenameFolderRequest {
    #[validate(length(min = 1, max = 50, message = "invalid folder name"))]
    pub new_name: String,
}
