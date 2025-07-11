use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum DnsRecord {
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
        println!("迁移 dns_record 数据库。。。");
        manager
            .create_table(
                Table::create()
                    .table(DnsRecord::Table)
                    .if_not_exists()
                    .col(pk_auto(DnsRecord::Id).integer())
                    .col(ColumnDef::new(DnsRecord::Name).string().null())
                    .col(ColumnDef::new(DnsRecord::DomainId).integer().null())
                    .col(ColumnDef::new(DnsRecord::RecordType).string().null())
                    .col(ColumnDef::new(DnsRecord::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(DnsRecord::UpdatedAt).date_time().not_null())
                    .col(ColumnDef::new(DnsRecord::Value).string().not_null())
                    .col(ColumnDef::new(DnsRecord::Ttl).integer().not_null())
                    .col(ColumnDef::new(DnsRecord::Priority).integer().null())
                    .index(
                        Index::create()
                            .unique()
                            .name("idx-name-id")
                            .col(DnsRecord::Name),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration2 scripts
        manager
            .drop_table(Table::drop().table(DnsRecord::Table).to_owned())
            .await
    }
}
