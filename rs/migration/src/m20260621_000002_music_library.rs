use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
enum Users {
    Table,
    Id,
}

#[derive(Iden)]
enum Songs {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    Title,
    Artist,
    Album,
    Year,
    Comment,
    Track,
    FilePath,
    OriginalFilename,
    FileSizeBytes,
    UploadedAt,
}

#[derive(Iden)]
enum Playlists {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    UserId,
    Name,
    IsDefault,
}

#[derive(Iden)]
enum PlaylistSongs {
    Table,
    Id,
    PlaylistId,
    SongId,
    Position,
    AddedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Songs::Table)
                    .if_not_exists()
                    .col(pk_auto(Songs::Id))
                    .col(
                        timestamp_with_time_zone(Songs::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Songs::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(string(Songs::Title))
                    .col(string(Songs::Artist))
                    .col(string_null(Songs::Album))
                    .col(integer_null(Songs::Year))
                    .col(text_null(Songs::Comment))
                    .col(integer_null(Songs::Track))
                    .col(string_uniq(Songs::FilePath))
                    .col(string_null(Songs::OriginalFilename))
                    .col(big_integer_null(Songs::FileSizeBytes))
                    .col(
                        timestamp_with_time_zone(Songs::UploadedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Playlists::Table)
                    .if_not_exists()
                    .col(pk_auto(Playlists::Id))
                    .col(
                        timestamp_with_time_zone(Playlists::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Playlists::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(integer(Playlists::UserId))
                    .col(string(Playlists::Name))
                    .col(boolean(Playlists::IsDefault).default(false))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_playlists_user")
                            .from(Playlists::Table, Playlists::UserId)
                            .to(Users::Table, Users::Id)
                            .on_update(ForeignKeyAction::NoAction)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_playlists_user_name")
                            .col(Playlists::UserId)
                            .col(Playlists::Name)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PlaylistSongs::Table)
                    .if_not_exists()
                    .col(pk_auto(PlaylistSongs::Id))
                    .col(integer(PlaylistSongs::PlaylistId))
                    .col(integer(PlaylistSongs::SongId))
                    .col(integer_null(PlaylistSongs::Position))
                    .col(
                        timestamp_with_time_zone(PlaylistSongs::AddedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_playlist_songs_playlist")
                            .from(PlaylistSongs::Table, PlaylistSongs::PlaylistId)
                            .to(Playlists::Table, Playlists::Id)
                            .on_update(ForeignKeyAction::NoAction)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_playlist_songs_song")
                            .from(PlaylistSongs::Table, PlaylistSongs::SongId)
                            .to(Songs::Table, Songs::Id)
                            .on_update(ForeignKeyAction::NoAction)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_playlist_songs_unique")
                            .col(PlaylistSongs::PlaylistId)
                            .col(PlaylistSongs::SongId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PlaylistSongs::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Playlists::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Songs::Table).to_owned())
            .await?;
        Ok(())
    }
}
