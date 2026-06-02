use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Provider::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Provider::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Provider::Name).string().not_null())
                    .col(ColumnDef::new(Provider::ApiKey).string().not_null())
                    .col(ColumnDef::new(Provider::ApiSecret).string().not_null())
                    .col(ColumnDef::new(Provider::ExtraConfig).json())
                    .col(
                        ColumnDef::new(Provider::CreatedAt)
                            .timestamp()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        ColumnDef::new(Provider::UpdatedAt)
                            .timestamp()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Domain::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Domain::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Domain::Name).string().not_null())
                    .col(ColumnDef::new(Domain::ProviderId).uuid().not_null())
                    .col(ColumnDef::new(Domain::Status).string().default("active"))
                    .col(
                        ColumnDef::new(Domain::CreatedAt)
                            .timestamp()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        ColumnDef::new(Domain::UpdatedAt)
                            .timestamp()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_domain_provider")
                            .from(Domain::Table, Domain::ProviderId)
                            .to(Provider::Table, Provider::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(DnsRecord::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DnsRecord::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(DnsRecord::DomainId).uuid().not_null())
                    .col(ColumnDef::new(DnsRecord::RecordType).string().not_null())
                    .col(ColumnDef::new(DnsRecord::Name).string().not_null())
                    .col(ColumnDef::new(DnsRecord::Value).string().not_null())
                    .col(ColumnDef::new(DnsRecord::Ttl).integer().not_null())
                    .col(ColumnDef::new(DnsRecord::Priority).integer().null())
                    .col(
                        ColumnDef::new(DnsRecord::CreatedAt)
                            .timestamp()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        ColumnDef::new(DnsRecord::UpdatedAt)
                            .timestamp()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_dnsrecord_domain")
                            .from(DnsRecord::Table, DnsRecord::DomainId)
                            .to(Domain::Table, Domain::Id),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DnsRecord::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Domain::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Provider::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Provider {
    Table,
    Id,
    Name,
    ApiKey,
    ApiSecret,
    ExtraConfig,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Domain {
    Table,
    Id,
    Name,
    ProviderId,
    Status,
    CreatedAt,
    UpdatedAt,
}

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
