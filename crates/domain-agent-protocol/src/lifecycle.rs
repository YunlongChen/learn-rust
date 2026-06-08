//! Lifecycle event types and structures for agent state management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents the type of lifecycle event that occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleEventType {
    AgentCreated,
    AgentRegistering,
    AgentAuthenticating,
    AgentPendingApproval,
    AgentApproved,
    AgentDenied,
    AgentConnected,
    AgentRegistered,
    AgentReconnecting,
    AgentDisconnected,
    AgentClosed,
    AgentError,
}

impl std::fmt::Display for LifecycleEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LifecycleEventType::AgentCreated => write!(f, "agent_created"),
            LifecycleEventType::AgentRegistering => write!(f, "agent_registering"),
            LifecycleEventType::AgentAuthenticating => write!(f, "agent_authenticating"),
            LifecycleEventType::AgentPendingApproval => write!(f, "agent_pending_approval"),
            LifecycleEventType::AgentApproved => write!(f, "agent_approved"),
            LifecycleEventType::AgentDenied => write!(f, "agent_denied"),
            LifecycleEventType::AgentConnected => write!(f, "agent_connected"),
            LifecycleEventType::AgentRegistered => write!(f, "agent_registered"),
            LifecycleEventType::AgentReconnecting => write!(f, "agent_reconnecting"),
            LifecycleEventType::AgentDisconnected => write!(f, "agent_disconnected"),
            LifecycleEventType::AgentClosed => write!(f, "agent_closed"),
            LifecycleEventType::AgentError => write!(f, "agent_error"),
        }
    }
}

/// Indicates the source of a lifecycle event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventSource {
    Agent,
    Manager,
    System,
}

impl std::fmt::Display for EventSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventSource::Agent => write!(f, "agent"),
            EventSource::Manager => write!(f, "manager"),
            EventSource::System => write!(f, "system"),
        }
    }
}

/// Represents a lifecycle event for an agent state transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleEvent {
    /// Unique identifier for this event.
    pub event_id: Uuid,
    /// The agent identifier this event belongs to.
    pub agent_id: Uuid,
    /// The type of lifecycle event.
    pub event_type: LifecycleEventType,
    /// When the event occurred.
    pub timestamp: DateTime<Utc>,
    /// The source of the event.
    pub source: EventSource,
    /// Optional reason for the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Optional additional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// Optional identifier of what triggered this event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triggered_by: Option<String>,
}

impl LifecycleEvent {
    /// Creates a new LifecycleEvent with the given parameters.
    pub fn new(
        agent_id: Uuid,
        event_type: LifecycleEventType,
        source: EventSource,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            agent_id,
            event_type,
            timestamp: Utc::now(),
            source,
            reason: None,
            metadata: None,
            triggered_by: None,
        }
    }

    /// Sets the reason for this event.
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Sets the metadata for this event.
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Sets what triggered this event.
    pub fn with_triggered_by(mut self, triggered_by: impl Into<String>) -> Self {
        self.triggered_by = Some(triggered_by.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_event_creation() {
        let agent_id = Uuid::new_v4();
        let event = LifecycleEvent::new(agent_id, LifecycleEventType::AgentCreated, EventSource::System);

        assert_eq!(event.agent_id, agent_id);
        assert_eq!(event.event_type, LifecycleEventType::AgentCreated);
        assert_eq!(event.source, EventSource::System);
        assert!(event.reason.is_none());
        assert!(event.metadata.is_none());
        assert!(event.triggered_by.is_none());
    }

    #[test]
    fn test_lifecycle_event_with_reason() {
        let agent_id = Uuid::new_v4();
        let event = LifecycleEvent::new(agent_id, LifecycleEventType::AgentError, EventSource::Agent)
            .with_reason("Connection timeout");

        assert_eq!(event.reason, Some("Connection timeout".to_string()));
    }

    #[test]
    fn test_lifecycle_event_with_metadata() {
        let agent_id = Uuid::new_v4();
        let metadata = serde_json::json!({"key": "value"});
        let event = LifecycleEvent::new(agent_id, LifecycleEventType::AgentConnected, EventSource::Agent)
            .with_metadata(metadata.clone());

        assert_eq!(event.metadata, Some(metadata));
    }

    #[test]
    fn test_lifecycle_event_with_triggered_by() {
        let agent_id = Uuid::new_v4();
        let event = LifecycleEvent::new(agent_id, LifecycleEventType::AgentApproved, EventSource::Manager)
            .with_triggered_by("admin@example.com");

        assert_eq!(event.triggered_by, Some("admin@example.com".to_string()));
    }

    #[test]
    fn test_lifecycle_event_type_display() {
        assert_eq!(LifecycleEventType::AgentCreated.to_string(), "agent_created");
        assert_eq!(LifecycleEventType::AgentRegistered.to_string(), "agent_registered");
        assert_eq!(LifecycleEventType::AgentError.to_string(), "agent_error");
    }

    #[test]
    fn test_event_source_display() {
        assert_eq!(EventSource::Agent.to_string(), "agent");
        assert_eq!(EventSource::Manager.to_string(), "manager");
        assert_eq!(EventSource::System.to_string(), "system");
    }

    #[test]
    fn test_lifecycle_event_serialization() {
        let agent_id = Uuid::new_v4();
        let event = LifecycleEvent::new(agent_id, LifecycleEventType::AgentCreated, EventSource::System)
            .with_reason("Test reason");

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: LifecycleEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.agent_id, agent_id);
        assert_eq!(deserialized.event_type, LifecycleEventType::AgentCreated);
        assert_eq!(deserialized.reason, Some("Test reason".to_string()));
    }
}
