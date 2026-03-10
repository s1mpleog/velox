use sqlx::{Pool, Postgres};

use crate::{
    dto::folder_dto::CreateFolderRequest,
    error::VeloxError,
    repositories::{folder_repository::FolderRepository, user_repository::UserRepository},
};

pub struct FolderService {}

impl FolderService {
    pub async fn create(
        pool: &Pool<Postgres>,
        request_data: &CreateFolderRequest,
        user_email: &str,
    ) -> Result<(), VeloxError> {
        let mut tx = pool.begin().await.map_err(VeloxError::SqlxError)?;
        // check if we have parent folder or not if we have parent id
        // then check if folder exists and it belongs to user
        // create the folder set its parent_id to parent_id
        // if we did not have a parent id then just create a new folder

        let is_user_exists = UserRepository::find_by_email(&mut tx, user_email).await?;

        let Some(user) = is_user_exists else {
            return Err(VeloxError::NotFound);
        };
        if let Some(id) = request_data.parent_id {
            FolderRepository::find_parent_folder(&mut tx, &id, &user.id)
                .await?
                .ok_or(VeloxError::NotFound)?;
        }

        FolderRepository::create(
            &mut tx,
            &request_data.name,
            &user.id,
            request_data.parent_id.as_ref(),
        )
        .await?;

        tx.commit().await.map_err(VeloxError::SqlxError)?;

        Ok(())
    }
}
