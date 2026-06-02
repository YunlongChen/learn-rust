use sea_orm_migration::{prelude::*, schema::*};
use tracing::info;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Domains {
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
        // Replace the sample below with your own migration2 scripts
        info!("迁移accounts数据库。。。");
        manager
            .create_table(
                Table::create()
                    .table(Domains::Table)
                    .if_not_exists()
                    .col(pk_auto(Domains::Id).big_integer())
                    .col(string(Domains::Name))
                    .col(big_integer(Domains::ProviderId))
                    .col(ColumnDef::new(Domains::Status).string().not_null())
                    .col(
                        ColumnDef::new(Domains::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Domains::UpdatedAt).date_time().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration2 scripts
        manager
            .drop_table(Table::drop().table(Domains::Table).to_owned())
            .await
    }
}
