use sea_orm_migration::{prelude::*, schema};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(schema::pk_auto(User::Id).not_null())
                    .col(schema::string(User::Email).not_null())
                    .col(schema::string(User::PasswordHash).not_null())
                    .col(schema::date_time(User::CreationTime).not_null())
                    .col(schema::date_time(User::LastActionTime))
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

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Email,
    PasswordHash,
    CreationTime,
    LastActionTime,
}
