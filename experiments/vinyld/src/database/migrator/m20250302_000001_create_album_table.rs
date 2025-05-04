use async_trait::async_trait;
use sea_orm_migration::prelude::*;

pub struct Migration;
impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250302_000003_create_album_table"
    }
}
#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Album::Table)
                    .col(
                        ColumnDef::new(Album::AlbumId)
                            .big_integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(Album::Title).text().not_null())
                    .col(ColumnDef::new(Album::Description).text())
                    .col(ColumnDef::new(Album::Uploader).big_integer().not_null())
                    .col(ColumnDef::new(Album::WritePolicy).json().not_null())
                    .col(ColumnDef::new(Album::Cover).string())
                    .col(ColumnDef::new(Album::CreatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Album::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Album {
    Table,
    AlbumId,
    Title,
    Description,
    Uploader,
    WritePolicy,
    Cover,
    CreatedAt,
}
