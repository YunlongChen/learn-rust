use sea_orm_migration::{prelude::*, schema::*};
use tracing::info;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum Providers {
    Table,
    Id,
    Name,
    ProviderId,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        info!("迁移accounts数据库。。。");
        manager
            .create_table(
                Table::create()
                    .table(Providers::Table)
                    .if_not_exists()
                    .col(pk_auto(Providers::Id).big_integer())
                    .col(string(Providers::Name))
                    .col(string(Providers::ProviderId).not_null())
                    .col(ColumnDef::new(Providers::Status).string().not_null())
                    .col(ColumnDef::new(Providers::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(Providers::UpdatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration2 scripts
        manager
            .drop_table(Table::drop().table(Providers::Table).to_owned())
            .await
    }
}
