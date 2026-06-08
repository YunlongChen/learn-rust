//! Migration: Create health scores table

use sea_orm_migration::prelude::*;

/// Create the agent_health_scores table.
/// This table stores health metrics over time for agents.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AgentHealthScores::Table)
                    .col(
                        ColumnDef::new(AgentHealthScores::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(AgentHealthScores::AgentId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AgentHealthScores::ScoredAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AgentHealthScores::OverallScore)
                            .double()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AgentHealthScores::LatencyMs)
                            .double()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AgentHealthScores::JitterMs)
                            .double()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AgentHealthScores::PacketLossPercent)
                            .double()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AgentHealthScores::BandwidthKbps)
                            .double()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AgentHealthScores::ComponentScores)
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
                    .name("idx_health_scores_agent_id")
                    .table(AgentHealthScores::Table)
                    .col(AgentHealthScores::AgentId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_health_scores_scored_at")
                    .table(AgentHealthScores::Table)
                    .col(AgentHealthScores::ScoredAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_health_scores_agent_id_scored_at")
                    .table(AgentHealthScores::Table)
                    .col(AgentHealthScores::AgentId)
                    .col(AgentHealthScores::ScoredAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AgentHealthScores::Table).to_owned())
            .await?;
        Ok(())
    }
}

/// AgentHealthScores table column names
#[derive(Iden)]
pub enum AgentHealthScores {
    Table,
    Id,
    AgentId,
    ScoredAt,
    OverallScore,
    LatencyMs,
    JitterMs,
    PacketLossPercent,
    BandwidthKbps,
    ComponentScores,
}
