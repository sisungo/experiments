use super::m20250302_000001_create_album_table::Album;
use async_trait::async_trait;
use sea_orm_migration::prelude::*;

pub struct Migration;
impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250302_000002_create_song_table"
    }
}
#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Song::Table)
                    .col(
                        ColumnDef::new(Song::SongId)
                            .big_integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(Song::Title).text().not_null())
                    .col(ColumnDef::new(Song::Album).big_integer().not_null())
                    .col(ColumnDef::new(Song::Uploader).big_integer().not_null())
                    .col(ColumnDef::new(Song::OriginAudio).string().not_null())
                    .col(ColumnDef::new(Song::ListenPolicy).json())
                    .col(ColumnDef::new(Song::CreatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .from(Song::Table, Song::Album)
                    .to(Album::Table, Album::AlbumId)
                    .on_delete(ForeignKeyAction::Restrict)
                    .on_update(ForeignKeyAction::Restrict)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Song::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Song {
    Table,
    SongId,
    Title,
    Album,
    Uploader,
    OriginAudio,
    ListenPolicy,
    CreatedAt,
}
