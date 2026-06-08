//! Lifecycle event entity

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// LifecycleEvent entity representing an audit log entry for agent lifecycle events.
#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "agent_lifecycle_events")]
pub struct Model {
    /// Unique identifier for the lifecycle event.
    #[sea_orm(primary_key, column_type = "Uuid")]
    pub id: Uuid,

    /// Reference to the agent this event belongs to.
    #[sea_orm(column_type = "Uuid")]
    pub agent_id: Uuid,

    /// Type of lifecycle event (e.g., "registered", "approved", "rejected", "disconnected").
    #[sea_orm(column_type = "Text")]
    pub event_type: String,

    /// Timestamp when the event occurred.
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub timestamp: DateTime<Utc>,

    /// Source of the event (e.g., "agent", "system", "admin").
    #[sea_orm(column_type = "Text")]
    pub source: String,

    /// Reason or description for the event.
    #[sea_orm(column_type = "Text", nullable)]
    pub reason: Option<String>,

    /// Additional metadata associated with the event in JSON format.
    #[sea_orm(column_type = "Json", nullable)]
    pub metadata: Option<Json>,

    /// Identity of the entity that triggered this event.
    #[sea_orm(column_type = "Text", nullable)]
    pub triggered_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No relation defined for LifecycleEvent")
    }
}

impl ActiveModelBehavior for ActiveModel {}
