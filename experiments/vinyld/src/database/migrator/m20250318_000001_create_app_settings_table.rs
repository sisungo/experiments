use async_trait::async_trait;
use sea_orm_migration::prelude::*;

pub struct Migration;
impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250318_000001_create_app_settings_table"
    }
}
#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AppSettings::Table)
                    .col(ColumnDef::new(AppSettings::Key).string().primary_key())
                    .col(ColumnDef::new(AppSettings::Value).json().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AppSettings::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum AppSettings {
    Table,
    Key,
    Value,
}
