use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration2 scripts
        println!("迁移accounts数据库。。。");
        manager
            .create_table(
                Table::create()
                    .table(Accounts::Table)
                    .if_not_exists()
                    .col(pk_auto(Accounts::Id).integer())
                    .col(string(Accounts::Name))
                    .col(string(Accounts::ApiKey))
                    .col(ColumnDef::new(Accounts::ApiSecret).string().null())
                    .col(ColumnDef::new(Accounts::ExtraConfig).json().null())
                    .col(ColumnDef::new(Accounts::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(Accounts::UpdatedAt).date_time().not_null())
                    .col(ColumnDef::new(Accounts::Username).string().not_null())
                    .col(ColumnDef::new(Accounts::Email).string().not_null())
                    .col(ColumnDef::new(Accounts::Salt).string().null())
                    .col(ColumnDef::new(Accounts::LastLogin).date_time().null())
                    .col(ColumnDef::new(Accounts::CredentialType).string().null())
                    .col(ColumnDef::new(Accounts::CredentialData).string().null())
                    .col(ColumnDef::new(Accounts::ProviderType).string().null())
                    .index(
                        Index::create()
                            .unique()
                            .name("idx-name-id")
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

//    pub id: Uuid,
//     pub name: String,
//     pub api_key: String,
//     pub api_secret: String,
//     #[sea_orm(column_type = "JsonBinary", nullable)]
//     pub extra_config: Option<serde_json::Value>,
//     pub created_at: DateTimeWithTimeZone,
//     pub updated_at: DateTimeWithTimeZone,
//
//     pub username: String,
//     pub email: String,
//     pub salt: String,
//     pub last_login: Option<String>,
//     pub credential_type: String,
//     pub credential_data: String,
//     pub provider_type: String,
#[derive(DeriveIden)]
enum Accounts {
    Table,
    Id,
    Name,
    ApiKey,
    ApiSecret,
    ExtraConfig,
    CreatedAt,
    UpdatedAt,
    Username,
    Email,
    Salt,
    LastLogin,
    CredentialType,
    CredentialData,
    ProviderType,
}
