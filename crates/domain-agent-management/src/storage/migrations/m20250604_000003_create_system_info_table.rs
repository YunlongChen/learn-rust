//! Migration: Create system info table

use sea_orm_migration::prelude::*;

/// Create the agent_system_info table.
/// This table stores diagnostic reports from agents.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AgentSystemInfo::Table)
                    .col(
                        ColumnDef::new(AgentSystemInfo::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(AgentSystemInfo::AgentId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AgentSystemInfo::ReportedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AgentSystemInfo::InfoType)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AgentSystemInfo::OsInfo)
                            .json()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AgentSystemInfo::EnvironmentInfo)
                            .json()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AgentSystemInfo::ProcessInfo)
                            .json()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AgentSystemInfo::NetworkInfo)
                            .json()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AgentSystemInfo::ResourceInfo)
                            .json()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_system_info_agent_id")
                    .table(AgentSystemInfo::Table)
                    .col(AgentSystemInfo::AgentId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_system_info_info_type")
                    .table(AgentSystemInfo::Table)
                    .col(AgentSystemInfo::InfoType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_system_info_reported_at")
                    .table(AgentSystemInfo::Table)
                    .col(AgentSystemInfo::ReportedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_system_info_agent_id_reported_at")
                    .table(AgentSystemInfo::Table)
                    .col(AgentSystemInfo::AgentId)
                    .col(AgentSystemInfo::ReportedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AgentSystemInfo::Table).to_owned())
            .await?;
        Ok(())
    }
}

/// AgentSystemInfo table column names
#[derive(Iden)]
pub enum AgentSystemInfo {
    Table,
    Id,
    AgentId,
    ReportedAt,
    InfoType,
    OsInfo,
    EnvironmentInfo,
    ProcessInfo,
    NetworkInfo,
    ResourceInfo,
}
