use sea_orm_migration::prelude::*;
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared("CREATE UNIQUE INDEX idx_playlists_one_default ON playlists(user_id) WHERE is_default = 1").await?;
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE playlist_songs DROP COLUMN position")
            .await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP INDEX idx_playlists_one_default")
            .await?;
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE playlist_songs ADD COLUMN position INTEGER")
            .await?;
        Ok(())
    }
}
