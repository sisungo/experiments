use async_trait::async_trait;
use sea_orm_migration::prelude::*;

pub struct Migration;
impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250316_000001_create_session_table"
    }
}
#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Session::Table)
                    .col(
                        ColumnDef::new(Session::Numeral)
                            .big_integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(Session::Uid).big_integer().not_null())
                    .col(
                        ColumnDef::new(Session::RefreshToken)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Session::AccessToken)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Session::RefreshExpiry)
                            .date_time()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Session::AccessExpiry).date_time().not_null())
                    .col(ColumnDef::new(Session::Permissions).integer().not_null())
                    .col(ColumnDef::new(Session::Source).string().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Session::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Session {
    Table,
    Numeral,
    Uid,
    RefreshToken,
    AccessToken,
    RefreshExpiry,
    AccessExpiry,
    Permissions,
    Source,
}
