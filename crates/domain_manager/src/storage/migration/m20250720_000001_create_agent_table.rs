use sea_orm_migration::{prelude::*, schema::*};
use tracing::info;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Agent {
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
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        info!("创建 Agent 表...");

        manager
            .create_table(
                Table::create()
                    .table(Agent::Table)
                    .if_not_exists()
                    .col(pk_uuid(Agent::Id))
                    .col(string(Agent::Name).not_null())
                    .col(ColumnDef::new(Agent::Description).text().null())
                    .col(string(Agent::Endpoint).not_null())
                    .col(ColumnDef::new(Agent::AuthKey).string().unique_key().null())
                    .col(ColumnDef::new(Agent::Capabilities).json().not_null())
                    .col(string(Agent::Status).not_null().default("offline"))
                    .col(ColumnDef::new(Agent::Tags).json().null())
                    .col(ColumnDef::new(Agent::SystemInfo).json().null())
                    .col(ColumnDef::new(Agent::ConnectionInfo).json().null())
                    .col(ColumnDef::new(Agent::LastHeartbeat).timestamp_with_time_zone().null())
                    .col(boolean(Agent::Enabled).not_null().default(true))
                    .col(
                        timestamp_with_time_zone(Agent::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Agent::UpdatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        info!("删除 Agent 表...");

        manager
            .drop_table(Table::drop().table(Agent::Table).to_owned())
            .await
    }
}
