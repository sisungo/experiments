use async_trait::async_trait;
use sea_orm_migration::prelude::*;

use super::{m20250216_000001_create_user_table::User, m20250302_000002_create_song_table::Song};

pub struct Migration;
impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250501_000001_create_lyrics_table"
    }
}
#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Lyrics::Table)
                    .col(
                        ColumnDef::new(Lyrics::LyricsId)
                            .big_integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(Lyrics::Song).big_integer().not_null())
                    .col(ColumnDef::new(Lyrics::Title).text().not_null())
                    .col(ColumnDef::new(Lyrics::Description).text().not_null())
                    .col(ColumnDef::new(Lyrics::Uploader).big_integer().not_null())
                    .col(ColumnDef::new(Lyrics::Language).string().not_null())
                    .col(ColumnDef::new(Lyrics::IsAuthoritative).boolean().not_null())
                    .col(ColumnDef::new(Lyrics::CreatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .from(Lyrics::Table, Lyrics::Song)
                    .to(Song::Table, Song::SongId)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Restrict)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .from(Lyrics::Table, Lyrics::Uploader)
                    .to(User::Table, User::Uid)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Restrict)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Lyrics::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Lyrics {
    Table,
    LyricsId,
    Song,
    Title,
    Description,
    Uploader,
    Language,
    IsAuthoritative,
    CreatedAt,
}
