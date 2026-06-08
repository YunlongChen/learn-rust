//! Health score entity

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// HealthScore entity representing health metrics over time for agents.
#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "agent_health_scores")]
pub struct Model {
    /// Unique identifier for the health score record.
    #[sea_orm(primary_key, column_type = "Uuid")]
    pub id: Uuid,

    /// Reference to the agent this health score belongs to.
    #[sea_orm(column_type = "Uuid")]
    pub agent_id: Uuid,

    /// Timestamp when the score was calculated.
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub scored_at: DateTime<Utc>,

    /// Overall health score (0-100).
    #[sea_orm(column_type = "Double")]
    pub overall_score: f64,

    /// Network latency in milliseconds.
    #[sea_orm(column_type = "Double", nullable)]
    pub latency_ms: Option<f64>,

    /// Network jitter in milliseconds.
    #[sea_orm(column_type = "Double", nullable)]
    pub jitter_ms: Option<f64>,

    /// Packet loss percentage (0-100).
    #[sea_orm(column_type = "Double", nullable)]
    pub packet_loss_percent: Option<f64>,

    /// Bandwidth in kilobits per second.
    #[sea_orm(column_type = "Double", nullable)]
    pub bandwidth_kbps: Option<f64>,

    /// Component-level scores in JSON format.
    #[sea_orm(column_type = "Json", nullable)]
    pub component_scores: Option<Json>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No relation defined for HealthScore")
    }
}

impl ActiveModelBehavior for ActiveModel {}
