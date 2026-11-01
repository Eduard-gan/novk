pub use super::_entities::playlist_songs::*;
use super::{playlists, songs};
use crate::errors::{LibraryError, LibraryResult};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ConnectionTrait, EntityTrait};

impl Model {
    pub async fn add<C: ConnectionTrait>(
        db: &C,
        user_id: i32,
        playlist_id: i32,
        song_id: i32,
    ) -> LibraryResult<Self> {
        playlists::Model::owned(db, user_id, playlist_id).await?;
        if songs::Entity::find_by_id(song_id).one(db).await?.is_none() {
            return Err(LibraryError::SongNotFound);
        }

        match (ActiveModel {
            playlist_id: Set(playlist_id),
            song_id: Set(song_id),
            ..Default::default()
        })
        .insert(db)
        .await
        {
            Ok(link) => Ok(link),
            Err(e)
                if matches!(
                    e.sql_err(),
                    Some(sea_orm::SqlErr::UniqueConstraintViolation(_))
                ) =>
            {
                Err(LibraryError::SongAlreadyInPlaylist)
            }
            Err(e) => Err(e.into()),
        }
    }
}
