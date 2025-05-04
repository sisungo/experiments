use super::m20250216_000001_create_user_table::User;
use async_trait::async_trait;
use sea_orm_migration::prelude::*;

pub struct Migration;
impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250223_000001_create_user_auth_password_table"
    }
}
#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserAuthPassword::Table)
                    .col(
                        ColumnDef::new(UserAuthPassword::Uid)
                            .big_integer()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(UserAuthPassword::Password)
                            .string()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .from(UserAuthPassword::Table, UserAuthPassword::Uid)
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
            .drop_table(Table::drop().table(UserAuthPassword::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum UserAuthPassword {
    Table,
    Uid,
    Password,
}
