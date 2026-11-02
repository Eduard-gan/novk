pub use super::_entities::playlists::*;
use crate::errors::{LibraryError, LibraryResult};
use sea_orm::{
    sea_query::OnConflict, ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter,
};

impl Model {
    pub async fn default_for<C: ConnectionTrait>(db: &C, user_id: i32) -> LibraryResult<Self> {
        if let Some(playlist) = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::IsDefault.eq(true))
            .one(db)
            .await?
        {
            return Ok(playlist);
        }
        Entity::insert(ActiveModel {
            user_id: Set(user_id),
            name: Set("Медиатека".to_string()),
            is_default: Set(true),
            ..Default::default()
        })
        .on_conflict(
            OnConflict::columns([Column::UserId, Column::Name])
                .update_column(Column::IsDefault)
                .to_owned(),
        )
        .exec_without_returning(db)
        .await?;
        Entity::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::IsDefault.eq(true))
            .one(db)
            .await?
            .ok_or_else(|| LibraryError::Internal(loco_rs::Error::string("default playlist missing after creation")))
    }

    pub async fn owned<C: ConnectionTrait>(db: &C, user_id: i32, playlist_id: i32) -> LibraryResult<Self> {
        Entity::find_by_id(playlist_id)
            .filter(Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(LibraryError::PlaylistNotFound)
    }

    pub async fn create<C: ConnectionTrait>(db: &C, user_id: i32, name: &str) -> LibraryResult<Self> {
        let name = name.trim();
        if name.is_empty() {
            return Err(LibraryError::PlaylistNameEmpty);
        }
        if name.chars().count() > 32 {
            return Err(LibraryError::PlaylistNameTooLong);
        }

        let result = ActiveModel {
            user_id: Set(user_id),
            name: Set(name.to_string()),
            is_default: Set(false),
            ..Default::default()
        }
        .insert(db)
        .await;
        match result {
            Ok(playlist) => Ok(playlist),
            Err(e) if matches!(e.sql_err(), Some(sea_orm::SqlErr::UniqueConstraintViolation(_))) => {
                Err(LibraryError::PlaylistNameTaken)
            }
            Err(e) => Err(e.into()),
        }
    }
}
