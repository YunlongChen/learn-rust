use sea_orm_migration::{prelude::*, schema::*};
use tracing::info;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Accounts {
    Table,
    Id,
    Name,
    Salt,
    LastLogin,
    ApiSecret,
    ExtraConfig,
    CreatedAt,
    UpdatedAt,
    CredentialType,
    CredentialData,
    ProviderType,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        info!("迁移accounts数据库。。。");
        manager
            .create_table(
                Table::create()
                    .table(Accounts::Table)
                    .if_not_exists()
                    .col(pk_auto(Accounts::Id).big_integer())
                    .col(string(Accounts::Name))
                    .col(string(Accounts::Salt).not_null())
                    .col(ColumnDef::new(Accounts::LastLogin).date_time().null())
                    .col(ColumnDef::new(Accounts::ExtraConfig).json().null())
                    .col(ColumnDef::new(Accounts::ProviderType).string().not_null())
                    .col(ColumnDef::new(Accounts::CredentialType).string().null())
                    .col(ColumnDef::new(Accounts::CredentialData).string().null())
                    .col(
                        ColumnDef::new(Accounts::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Accounts::UpdatedAt).date_time().null())
                    .index(
                        Index::create()
                            .unique()
                            .name("idx_accounts_name")
                            .col(Accounts::Name),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration2 scripts
        manager
            .drop_table(Table::drop().table(Accounts::Table).to_owned())
            .await
    }
}
