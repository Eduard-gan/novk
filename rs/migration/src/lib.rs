#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;
mod m20220101_000001_users;
mod m20260621_000002_music_library;
mod m20261031_000003_remove_auth_tokens;
mod m20261031_000004_playlist_order;
mod m20261031_000005_genres;
mod m20261031_000006_song_storage;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20260621_000002_music_library::Migration),
            Box::new(m20261031_000003_remove_auth_tokens::Migration),
            Box::new(m20261031_000004_playlist_order::Migration),
            Box::new(m20261031_000005_genres::Migration),
            Box::new(m20261031_000006_song_storage::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}
