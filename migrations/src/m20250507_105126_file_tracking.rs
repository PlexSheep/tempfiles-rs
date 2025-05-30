use sea_orm_migration::{prelude::*, schema};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(File::Table)
                    .if_not_exists()
                    .col(schema::pk_auto(File::Id))
                    .col(ColumnDef::new(File::UserId).integer().null().take())
                    .col(schema::date_time(File::ExpirationTime))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(File::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum File {
    Table,
    Id,
    UserId,
    ExpirationTime,
}
