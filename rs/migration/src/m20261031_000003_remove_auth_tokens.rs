use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const COLUMNS: [&str; 7] = ["reset_token", "reset_sent_at", "email_verification_token", "email_verification_sent_at", "email_verified_at", "magic_link_token", "magic_link_expiration"];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for column in COLUMNS {
            manager.alter_table(Table::alter().table(Alias::new("users")).drop_column(Alias::new(column)).to_owned()).await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for column in COLUMNS {
            let mut definition = ColumnDef::new(Alias::new(column));
            if column.ends_with("_at") || column.ends_with("expiration") {
                definition.timestamp_with_time_zone();
            } else {
                definition.string();
            }
            manager.alter_table(Table::alter().table(Alias::new("users")).add_column(&mut definition).to_owned()).await?;
        }
        Ok(())
    }
}
