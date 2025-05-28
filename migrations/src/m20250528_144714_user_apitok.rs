use sea_orm_migration::{prelude::*, schema};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserToken::Table)
                    .if_not_exists()
                    .col(
                        schema::string(UserToken::Token)
                            .not_null()
                            .unique_key()
                            .primary_key(),
                    )
                    .col(schema::integer(UserToken::UserId).not_null())
                    .col(schema::date_time(UserToken::CreationTime).not_null())
                    .col(schema::date_time(UserToken::ExpirationTime).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserToken::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UserToken {
    Table,
    Token,
    UserId,
    ExpirationTime,
    CreationTime,
}
