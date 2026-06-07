//! System info entity

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// SystemInfo entity representing diagnostic reports from agents.
#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "agent_system_info")]
pub struct Model {
    /// Unique identifier for the system info record.
    #[sea_orm(primary_key, column_type = "Uuid")]
    pub id: Uuid,

    /// Reference to the agent that reported this info.
    #[sea_orm(column_type = "Uuid")]
    pub agent_id: Uuid,

    /// Timestamp when the info was reported.
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub reported_at: DateTime<Utc>,

    /// Type of system info (e.g., "os", "environment", "process", "network", "resource").
    #[sea_orm(column_type = "Text")]
    pub info_type: String,

    /// Operating system information in JSON format.
    #[sea_orm(column_type = "Json", nullable)]
    pub os_info: Option<Json>,

    /// Environment information in JSON format.
    #[sea_orm(column_type = "Json", nullable)]
    pub environment_info: Option<Json>,

    /// Process information in JSON format.
    #[sea_orm(column_type = "Json", nullable)]
    pub process_info: Option<Json>,

    /// Network information in JSON format.
    #[sea_orm(column_type = "Json", nullable)]
    pub network_info: Option<Json>,

    /// Resource usage information in JSON format.
    #[sea_orm(column_type = "Json", nullable)]
    pub resource_info: Option<Json>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No relation defined for SystemInfo")
    }
}

impl ActiveModelBehavior for ActiveModel {}
