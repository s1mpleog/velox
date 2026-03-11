use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct RenameFileRequest {
    #[validate(length(min = 1, max = 100, message = "invalid file name"))]
    pub new_name: String,
}
