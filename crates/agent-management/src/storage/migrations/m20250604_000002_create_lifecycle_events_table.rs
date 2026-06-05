//! Migration: Create lifecycle events table

use sea_orm_migration::prelude::*;

/// Create the agent_lifecycle_events table.
/// This table stores audit log entries for agent lifecycle events.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AgentLifecycleEvents::Table)
                    .col(
                        ColumnDef::new(AgentLifecycleEvents::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(AgentLifecycleEvents::AgentId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AgentLifecycleEvents::EventType)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AgentLifecycleEvents::Timestamp)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AgentLifecycleEvents::Source)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AgentLifecycleEvents::Reason)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AgentLifecycleEvents::Metadata)
                            .json()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AgentLifecycleEvents::TriggeredBy)
                            .text()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_lifecycle_events_agent_id")
                    .table(AgentLifecycleEvents::Table)
                    .col(AgentLifecycleEvents::AgentId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_lifecycle_events_event_type")
                    .table(AgentLifecycleEvents::Table)
                    .col(AgentLifecycleEvents::EventType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_lifecycle_events_timestamp")
                    .table(AgentLifecycleEvents::Table)
                    .col(AgentLifecycleEvents::Timestamp)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_lifecycle_events_agent_id_timestamp")
                    .table(AgentLifecycleEvents::Table)
                    .col(AgentLifecycleEvents::AgentId)
                    .col(AgentLifecycleEvents::Timestamp)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AgentLifecycleEvents::Table).to_owned())
            .await?;
        Ok(())
    }
}

/// AgentLifecycleEvents table column names
#[derive(Iden)]
pub enum AgentLifecycleEvents {
    Table,
    Id,
    AgentId,
    EventType,
    Timestamp,
    Source,
    Reason,
    Metadata,
    TriggeredBy,
}
