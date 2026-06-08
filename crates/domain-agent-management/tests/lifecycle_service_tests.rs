//! Unit tests for LifecycleService types
//!
//! These tests verify that the LifecycleService types compile correctly.

use uuid::Uuid;

// Test LifecycleEventType enum variants exist
#[test]
fn test_lifecycle_event_type_variants() {
    use domain_agent_protocol::lifecycle::LifecycleEventType;

    let _ = LifecycleEventType::AgentCreated;
    let _ = LifecycleEventType::AgentRegistering;
    let _ = LifecycleEventType::AgentAuthenticating;
    let _ = LifecycleEventType::AgentPendingApproval;
    let _ = LifecycleEventType::AgentApproved;
    let _ = LifecycleEventType::AgentDenied;
    let _ = LifecycleEventType::AgentConnected;
    let _ = LifecycleEventType::AgentRegistered;
    let _ = LifecycleEventType::AgentReconnecting;
    let _ = LifecycleEventType::AgentDisconnected;
    let _ = LifecycleEventType::AgentClosed;
    let _ = LifecycleEventType::AgentError;
}

// Test EventSource enum variants exist
#[test]
fn test_event_source_variants() {
    use domain_agent_protocol::lifecycle::EventSource;

    let _ = EventSource::Agent;
    let _ = EventSource::Manager;
    let _ = EventSource::System;
}

// Test LifecycleEvent construction
#[test]
fn test_lifecycle_event_new() {
    use domain_agent_protocol::lifecycle::{LifecycleEvent, LifecycleEventType, EventSource};

    let agent_id = Uuid::new_v4();
    let event = LifecycleEvent::new(agent_id, LifecycleEventType::AgentCreated, EventSource::System);

    assert_eq!(event.agent_id, agent_id);
    assert_eq!(event.event_type, LifecycleEventType::AgentCreated);
    assert_eq!(event.source, EventSource::System);
    assert!(event.reason.is_none());
    assert!(event.metadata.is_none());
    assert!(event.triggered_by.is_none());
}

// Test LifecycleEvent with_reason
#[test]
fn test_lifecycle_event_with_reason() {
    use domain_agent_protocol::lifecycle::{LifecycleEvent, LifecycleEventType, EventSource};

    let agent_id = Uuid::new_v4();
    let event = LifecycleEvent::new(agent_id, LifecycleEventType::AgentError, EventSource::Agent)
        .with_reason("Connection timeout");

    assert_eq!(event.reason, Some("Connection timeout".to_string()));
}

// Test LifecycleEvent with_metadata
#[test]
fn test_lifecycle_event_with_metadata() {
    use domain_agent_protocol::lifecycle::{LifecycleEvent, LifecycleEventType, EventSource};

    let agent_id = Uuid::new_v4();
    let metadata = serde_json::json!({"key": "value"});
    let event = LifecycleEvent::new(agent_id, LifecycleEventType::AgentConnected, EventSource::Agent)
        .with_metadata(metadata.clone());

    assert_eq!(event.metadata, Some(metadata));
}

// Test LifecycleEvent with_triggered_by
#[test]
fn test_lifecycle_event_with_triggered_by() {
    use domain_agent_protocol::lifecycle::{LifecycleEvent, LifecycleEventType, EventSource};

    let agent_id = Uuid::new_v4();
    let event = LifecycleEvent::new(agent_id, LifecycleEventType::AgentApproved, EventSource::Manager)
        .with_triggered_by("admin@example.com");

    assert_eq!(event.triggered_by, Some("admin@example.com".to_string()));
}

// Test LifecycleEventType Display implementation
#[test]
fn test_lifecycle_event_type_display() {
    use domain_agent_protocol::lifecycle::LifecycleEventType;

    assert_eq!(LifecycleEventType::AgentCreated.to_string(), "agent_created");
    assert_eq!(LifecycleEventType::AgentRegistered.to_string(), "agent_registered");
    assert_eq!(LifecycleEventType::AgentError.to_string(), "agent_error");
}

// Test EventSource Display implementation
#[test]
fn test_event_source_display() {
    use domain_agent_protocol::lifecycle::EventSource;

    assert_eq!(EventSource::Agent.to_string(), "agent");
    assert_eq!(EventSource::Manager.to_string(), "manager");
    assert_eq!(EventSource::System.to_string(), "system");
}

// Test LifecycleEvent serialization
#[test]
fn test_lifecycle_event_serialization() {
    use domain_agent_protocol::lifecycle::{LifecycleEvent, LifecycleEventType, EventSource};

    let agent_id = Uuid::new_v4();
    let event = LifecycleEvent::new(agent_id, LifecycleEventType::AgentCreated, EventSource::System)
        .with_reason("Test reason");

    let json = serde_json::to_string(&event).unwrap();
    let deserialized: LifecycleEvent = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.agent_id, agent_id);
    assert_eq!(deserialized.event_type, LifecycleEventType::AgentCreated);
    assert_eq!(deserialized.reason, Some("Test reason".to_string()));
}
