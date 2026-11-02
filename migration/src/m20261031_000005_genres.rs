use sea_orm_migration::prelude::*;
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE genres (id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, name TEXT NOT NULL UNIQUE)",
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE songs ADD COLUMN genre_id INTEGER REFERENCES genres(id) ON DELETE RESTRICT",
            )
            .await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE songs DROP COLUMN genre_id")
            .await?;
        manager.get_connection().execute_unprepared("DROP TABLE genres").await?;
        Ok(())
    }
}
