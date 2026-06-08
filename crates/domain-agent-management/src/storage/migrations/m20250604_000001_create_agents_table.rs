//! Migration: Create agents table

use sea_orm_migration::prelude::*;

/// Create the agents table.
/// This table stores agent registration records with their status and capabilities.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Agents::Table)
                    .col(
                        ColumnDef::new(Agents::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Agents::Name).text().not_null())
                    .col(ColumnDef::new(Agents::Endpoint).text().not_null())
                    .col(ColumnDef::new(Agents::Status).text().not_null())
                    .col(ColumnDef::new(Agents::ApprovalState).text().not_null())
                    .col(ColumnDef::new(Agents::Capabilities).json().not_null())
                    .col(ColumnDef::new(Agents::CertFingerprint).text().null())
                    .col(ColumnDef::new(Agents::AuthMethod).text().not_null())
                    .col(ColumnDef::new(Agents::Version).text().null())
                    .col(
                        ColumnDef::new(Agents::RegisteredAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Agents::LastSeenAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Agents::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    .col(
                        ColumnDef::new(Agents::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_agents_name")
                    .table(Agents::Table)
                    .col(Agents::Name)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_agents_status")
                    .table(Agents::Table)
                    .col(Agents::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_agents_approval_state")
                    .table(Agents::Table)
                    .col(Agents::ApprovalState)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_agents_endpoint")
                    .table(Agents::Table)
                    .col(Agents::Endpoint)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_agents_created_at")
                    .table(Agents::Table)
                    .col(Agents::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_agents_updated_at")
                    .table(Agents::Table)
                    .col(Agents::UpdatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_agents_last_seen_at")
                    .table(Agents::Table)
                    .col(Agents::LastSeenAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_agents_registered_at")
                    .table(Agents::Table)
                    .col(Agents::RegisteredAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Agents::Table).to_owned())
            .await?;
        Ok(())
    }
}

/// Agents table column names
#[derive(Iden)]
pub enum Agents {
    Table,
    Id,
    Name,
    Endpoint,
    Status,
    ApprovalState,
    Capabilities,
    CertFingerprint,
    AuthMethod,
    Version,
    RegisteredAt,
    LastSeenAt,
    CreatedAt,
    UpdatedAt,
}
