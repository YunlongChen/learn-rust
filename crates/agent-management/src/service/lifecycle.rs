//! Lifecycle event recording service
//!
//! This module provides the LifecycleService for recording and querying
//! lifecycle events in the database.

use agent_protocol::lifecycle::{EventSource, LifecycleEvent, LifecycleEventType};
use sea_orm::ActiveModelTrait;
use sea_orm::entity::prelude::*;
use sea_orm::{Set, QueryOrder, EntityTrait, JsonValue};
use uuid::Uuid;

use crate::storage::Database;
use crate::storage::entities::lifecycle_event::{Entity as LifecycleEventEntity, ActiveModel, Model as LifecycleEventModel};

/// Service for managing agent lifecycle events.
///
/// This service provides methods to record lifecycle events to the database
/// and query event history for agents.
#[derive(Clone, Debug)]
pub struct LifecycleService {
    db: Database,
}

impl LifecycleService {
    /// Creates a new LifecycleService with the given database connection.
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Records a lifecycle event to the database.
    ///
    /// # Arguments
    ///
    /// * `event` - The lifecycle event to record
    ///
    /// # Returns
    ///
    /// Returns the created event's ID on success.
    pub async fn record_event(
        &self,
        event: &LifecycleEvent,
    ) -> Result<Uuid, sea_orm::DbErr> {
        let event_id = event.event_id;
        let event_type = event.event_type.to_string();
        let source = event.source.to_string();

        let active_model = ActiveModel {
            id: Set(event.event_id),
            agent_id: Set(event.agent_id),
            event_type: Set(event_type),
            timestamp: Set(event.timestamp),
            source: Set(source),
            reason: Set(event.reason.clone()),
            metadata: Set(event.metadata.clone()),
            triggered_by: Set(event.triggered_by.clone()),
        };

        active_model.insert(self.db.get_conn()).await?;

        Ok(event_id)
    }

    /// Retrieves all lifecycle events for a specific agent.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The ID of the agent to get events for
    ///
    /// # Returns
    ///
    /// Returns a vector of lifecycle events ordered by timestamp.
    pub async fn get_events_for_agent(
        &self,
        agent_id: Uuid,
    ) -> Result<Vec<LifecycleEvent>, sea_orm::DbErr> {
        use crate::storage::entities::lifecycle_event::Column;

        let events: Vec<LifecycleEventModel> = LifecycleEventEntity::find()
            .filter(Column::AgentId.eq(agent_id))
            .order_by_asc(Column::Timestamp)
            .all(self.db.get_conn())
            .await?;

        Ok(events
            .into_iter()
            .map(|model| {
                let event_type = match model.event_type.as_str() {
                    "agent_created" => LifecycleEventType::AgentCreated,
                    "agent_registering" => LifecycleEventType::AgentRegistering,
                    "agent_authenticating" => LifecycleEventType::AgentAuthenticating,
                    "agent_pending_approval" => LifecycleEventType::AgentPendingApproval,
                    "agent_approved" => LifecycleEventType::AgentApproved,
                    "agent_denied" => LifecycleEventType::AgentDenied,
                    "agent_connected" => LifecycleEventType::AgentConnected,
                    "agent_registered" => LifecycleEventType::AgentRegistered,
                    "agent_reconnecting" => LifecycleEventType::AgentReconnecting,
                    "agent_disconnected" => LifecycleEventType::AgentDisconnected,
                    "agent_closed" => LifecycleEventType::AgentClosed,
                    "agent_error" => LifecycleEventType::AgentError,
                    other => panic!("Unknown event type: {}", other),
                };

                let source = match model.source.as_str() {
                    "agent" => EventSource::Agent,
                    "manager" => EventSource::Manager,
                    "system" => EventSource::System,
                    other => panic!("Unknown event source: {}", other),
                };

                // Convert sea_orm Json to serde_json::Value
                let metadata = model.metadata.map(|j| match j {
                    JsonValue::Null => serde_json::Value::Null,
                    JsonValue::Bool(b) => serde_json::Value::Bool(b),
                    JsonValue::Number(n) => serde_json::Value::Number(n),
                    JsonValue::String(s) => serde_json::Value::String(s),
                    JsonValue::Array(arr) => serde_json::Value::Array(arr),
                    JsonValue::Object(obj) => serde_json::Value::Object(obj),
                });

                LifecycleEvent {
                    event_id: model.id,
                    agent_id: model.agent_id,
                    event_type,
                    timestamp: model.timestamp,
                    source,
                    reason: model.reason,
                    metadata,
                    triggered_by: model.triggered_by,
                }
            })
            .collect())
    }
}
