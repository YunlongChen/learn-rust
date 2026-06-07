//! Agent entity

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Agent entity representing an agent record in the database.
#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "agents")]
pub struct Model {
    /// Unique identifier for the agent.
    #[sea_orm(primary_key, column_type = "Uuid")]
    pub id: Uuid,

    /// Human-readable name of the agent.
    #[sea_orm(column_type = "Text")]
    pub name: String,

    /// WebSocket endpoint URL for connecting to the agent.
    #[sea_orm(column_type = "Text")]
    pub endpoint: String,

    /// Current operational status of the agent.
    #[sea_orm(column_type = "Text")]
    pub status: String,

    /// Approval state for agent registration.
    #[sea_orm(column_type = "Text")]
    pub approval_state: String,

    /// JSON-encoded capabilities supported by the agent.
    #[sea_orm(column_type = "Json")]
    pub capabilities: Json,

    /// SHA256 fingerprint of the agent's TLS certificate.
    #[sea_orm(column_type = "Text", nullable)]
    pub cert_fingerprint: Option<String>,

    /// Authentication method used by the agent.
    #[sea_orm(column_type = "Text")]
    pub auth_method: String,

    /// Version of the agent software.
    #[sea_orm(column_type = "Text", nullable)]
    pub version: Option<String>,

    /// Timestamp when the agent registered.
    #[sea_orm(column_type = "TimestampWithTimeZone", nullable)]
    pub registered_at: Option<DateTime<Utc>>,

    /// Timestamp when the agent was last seen.
    #[sea_orm(column_type = "TimestampWithTimeZone", nullable)]
    pub last_seen_at: Option<DateTime<Utc>>,

    /// Timestamp when the agent was created.
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,

    /// Timestamp when the agent was last updated.
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No relation defined for Agent")
    }
}

impl ActiveModelBehavior for ActiveModel {}
