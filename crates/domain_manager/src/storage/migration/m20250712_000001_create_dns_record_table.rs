use sea_orm_migration::{prelude::*, schema::*};
use tracing::info;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum DnsRecords {
    Table,
    Id,
    DomainId,
    RecordType,
    Name,
    Value,
    Ttl,
    Priority,
    CreatedAt,
    UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration2 scripts
        info!("迁移 dns_record 数据库。。。");
        manager
            .create_table(
                Table::create()
                    .table(DnsRecords::Table)
                    .if_not_exists()
                    .col(pk_auto(DnsRecords::Id).big_integer())
                    .col(ColumnDef::new(DnsRecords::Name).string().null())
                    .col(ColumnDef::new(DnsRecords::DomainId).big_integer().null())
                    .col(ColumnDef::new(DnsRecords::RecordType).string().null())
                    .col(
                        ColumnDef::new(DnsRecords::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(DnsRecords::UpdatedAt).date_time().null())
                    .col(ColumnDef::new(DnsRecords::Value).string().not_null())
                    .col(ColumnDef::new(DnsRecords::Ttl).integer().not_null())
                    .col(ColumnDef::new(DnsRecords::Priority).integer().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration2 scripts
        manager
            .drop_table(Table::drop().table(DnsRecords::Table).to_owned())
            .await
    }
}
