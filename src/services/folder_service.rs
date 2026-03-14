use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{
    dto::folder_dto::{CreateFolderRequest, FolderContents, RenameFolderRequest},
    error::VeloxError,
    models::folder_model::Folder,
    repositories::{file_repository::FileRepository, folder_repository::FolderRepository},
};

pub struct FolderService {}

impl FolderService {
    pub async fn create(
        pool: &Pool<Postgres>,
        request_data: &CreateFolderRequest,
        user_id: &Uuid,
    ) -> Result<(), VeloxError> {
        let mut tx = pool.begin().await.map_err(VeloxError::SqlxError)?;
        // check if we have parent folder or not if we have parent id
        // then check if folder exists and it belongs to user
        // create the folder set its parent_id to parent_id
        // if we did not have a parent id then just create a new folder

        if let Some(id) = request_data.parent_id {
            FolderRepository::find_parent_folder(&mut tx, &id, user_id)
                .await?
                .ok_or(VeloxError::NotFound)?;
        }

        FolderRepository::create(
            &mut tx,
            &request_data.name,
            user_id,
            request_data.parent_id.as_ref(),
        )
        .await?;

        tx.commit().await.map_err(VeloxError::SqlxError)?;

        Ok(())
    }

    pub async fn rename(
        pool: &Pool<Postgres>,
        folder_id: &Uuid,
        user_id: &Uuid,
        request_data: &RenameFolderRequest,
    ) -> Result<(), VeloxError> {
        let mut tx = pool.begin().await.map_err(VeloxError::SqlxError)?;
        FolderRepository::rename(&mut tx, &request_data.new_name, user_id, folder_id).await?;
        tx.commit().await.map_err(VeloxError::SqlxError)?;
        Ok(())
    }

    pub async fn delete(
        pool: &Pool<Postgres>,
        user_id: &Uuid,
        folder_id: &Uuid,
    ) -> Result<(), VeloxError> {
        let mut tx = pool.begin().await.map_err(VeloxError::SqlxError)?;
        FolderRepository::delete(&mut tx, folder_id, user_id).await?;
        tx.commit().await.map_err(VeloxError::SqlxError)?;
        Ok(())
    }

    pub async fn get_all(pool: &Pool<Postgres>, user_id: &Uuid) -> Result<Vec<Folder>, VeloxError> {
        let mut tx = pool.begin().await.map_err(VeloxError::SqlxError)?;
        let folders = FolderRepository::find_all(&mut tx, &user_id).await?;
        tx.commit().await.map_err(VeloxError::SqlxError)?;
        Ok(folders)
    }

    pub async fn get_contents(
        pool: &Pool<Postgres>,
        user_id: &Uuid,
        folder_id: &Uuid,
    ) -> Result<FolderContents, VeloxError> {
        let mut tx = pool.begin().await.map_err(VeloxError::SqlxError)?;
        let folders = FolderRepository::find_by_parent_id(&mut tx, &folder_id, user_id).await?;
        let files = FileRepository::find_by_folder_id(&mut tx, user_id, &folder_id).await?;
        tx.commit().await.map_err(VeloxError::SqlxError)?;
        Ok(FolderContents { files, folders })
    }

    pub async fn get_root_contents(
        pool: &Pool<Postgres>,
        user_id: &Uuid,
    ) -> Result<FolderContents, VeloxError> {
        let mut tx = pool.begin().await.map_err(VeloxError::SqlxError)?;
        let folders = FolderRepository::find_root_folders(&mut tx, user_id).await?;
        let files = FileRepository::find_root_files(&mut tx, user_id).await?;
        tx.commit().await.map_err(VeloxError::SqlxError)?;
        Ok(FolderContents { folders, files })
    }
}
