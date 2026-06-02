use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Agents {
    #[sea_orm(iden = "agents")]
    Table,
    Id,
    Name,
    Description,
    Endpoint,
    AuthKey,
    Capabilities,
    Status,
    Tags,
    SystemInfo,
    ConnectionInfo,
    LastHeartbeat,
    Enabled,
    CreatedAt,
    UpdatedAt,
    ApprovalState,
    AgentKeyHash,
    ApprovedAt,
    ApprovedBy,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Agents::Table)
                    .if_not_exists()
                    .col(uuid_uniq(Agents::Id))
                    .col(string(Agents::Name))
                    .col(ColumnDef::new(Agents::Description).text().null())
                    .col(string(Agents::Endpoint))
                    .col(ColumnDef::new(Agents::AuthKey).string().null())
                    .col(ColumnDef::new(Agents::Capabilities).json().null())
                    .col(
                        string(Agents::Status)
                            .not_null()
                            .default("offline".to_string()),
                    )
                    .col(ColumnDef::new(Agents::Tags).json().null())
                    .col(ColumnDef::new(Agents::SystemInfo).json().null())
                    .col(ColumnDef::new(Agents::ConnectionInfo).json().null())
                    .col(ColumnDef::new(Agents::LastHeartbeat).date_time().null())
                    .col(boolean(Agents::Enabled).not_null().default(false))
                    .col(ColumnDef::new(Agents::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(Agents::UpdatedAt).date_time().null())
                    .col(ColumnDef::new(Agents::ApprovedBy).string().null())
                    .col(ColumnDef::new(Agents::ApprovedAt).date_time().null())
                    .col(ColumnDef::new(Agents::AgentKeyHash).string().null())
                    .col(
                        ColumnDef::new(Agents::ApprovalState)
                            .string()
                            .not_null()
                            .default("pending".to_string()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Agents::Table).to_owned())
            .await
    }
}
