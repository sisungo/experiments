use async_trait::async_trait;
use sea_orm_migration::prelude::*;

pub struct Migration;
impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250216_000001_create_user_table"
    }
}
#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .col(
                        ColumnDef::new(User::Uid)
                            .big_integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(User::Username)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(User::Nickname).text().not_null())
                    .col(ColumnDef::new(User::Avatar).string())
                    .col(ColumnDef::new(User::Gender).string())
                    .col(ColumnDef::new(User::DateOfBirth).date())
                    .col(ColumnDef::new(User::Country).string())
                    .col(ColumnDef::new(User::City).string())
                    .col(ColumnDef::new(User::Signature).text())
                    .col(ColumnDef::new(User::BannerImage).string())
                    .col(ColumnDef::new(User::Banned).json())
                    .col(
                        ColumnDef::new(User::Groups)
                            .array(ColumnType::String(StringLen::None))
                            .not_null(),
                    )
                    .col(ColumnDef::new(User::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(User::LastLoginedAt).date_time().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum User {
    Table,
    Uid,
    Username,
    Nickname,
    Avatar,
    Gender,
    DateOfBirth,
    Country,
    City,
    Signature,
    BannerImage,
    Banned,
    Groups,
    CreatedAt,
    LastLoginedAt,
}
