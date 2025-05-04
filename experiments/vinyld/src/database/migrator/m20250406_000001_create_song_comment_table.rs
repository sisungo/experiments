use super::{m20250216_000001_create_user_table::User, m20250302_000002_create_song_table::Song};
use async_trait::async_trait;
use sea_orm_migration::prelude::*;

pub struct Migration;
impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250406_000001_create_song_comment_table"
    }
}
#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SongComment::Table)
                    .col(
                        ColumnDef::new(SongComment::SongCommentId)
                            .big_integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(SongComment::Song).big_integer().not_null())
                    .col(ColumnDef::new(SongComment::Parent).big_integer())
                    .col(
                        ColumnDef::new(SongComment::CreatedBy)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SongComment::Text).text().not_null())
                    .col(ColumnDef::new(SongComment::Attachments).array(ColumnType::Text))
                    .col(
                        ColumnDef::new(SongComment::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .from(SongComment::Table, SongComment::Song)
                    .to(Song::Table, Song::SongId)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Restrict)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .from(SongComment::Table, SongComment::CreatedBy)
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
            .drop_table(Table::drop().table(SongComment::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum SongComment {
    Table,
    SongCommentId,
    Song,
    Parent,
    CreatedBy,
    Text,
    Attachments,
    CreatedAt,
}
