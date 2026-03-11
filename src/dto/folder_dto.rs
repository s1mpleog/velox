use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::{file_model::File, folder_model::Folder};

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

#[derive(Serialize)]
pub struct FolderContents {
    pub files: Vec<File>,
    pub folders: Vec<Folder>,
}
