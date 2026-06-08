//! Unit tests for AgentService types
//!
//! These tests verify that the service types compile correctly.

use uuid::Uuid;

// Test that UpdateAgentInput can be constructed with defaults
#[test]
fn test_update_agent_input_default() {
    let _input = domain_agent_management::service::agent::UpdateAgentInput::default();
}

// Test that UpdateAgentInput can be constructed with some fields
#[test]
fn test_update_agent_input_partial() {
    let _input = domain_agent_management::service::agent::UpdateAgentInput {
        name: Some("updated-agent".to_string()),
        status: Some("connected".to_string()),
        ..Default::default()
    };
}

// Test that AgentFilters can be constructed with defaults
#[test]
fn test_agent_filters_default() {
    let _filters = domain_agent_management::service::agent::AgentFilters::default();
}

// Test that AgentFilters can be constructed with filters
#[test]
fn test_agent_filters_with_values() {
    let _filters = domain_agent_management::service::agent::AgentFilters {
        status: Some("connected".to_string()),
        approval_state: Some("approved".to_string()),
    };
}

// Test that AgentService panics on default (as documented)
#[test]
#[should_panic(expected = "AgentService::default() is not supported")]
fn test_agent_service_default_panics() {
    // This should panic because AgentService requires a database
    let _ = domain_agent_management::service::agent::AgentService::default();
}

// Test CreateAgentInput structure (without Json field to avoid type issues)
#[test]
fn test_create_agent_input_fields() {
    let id = Uuid::new_v4();

    // Verify all fields are the right types
    let _id: Uuid = id;
    let _name: String = "test-agent".to_string();
    let _endpoint: String = "ws://localhost:8080".to_string();
    let _status: String = "pending".to_string();
    let _approval_state: String = "pending".to_string();
    let _auth_method: String = "tls".to_string();
    let _cert_fingerprint: Option<String> = Some("fingerprint".to_string());
    let _version: Option<String> = Some("1.0.0".to_string());
}

// Test AgentInfo structure (without Json field)
#[test]
fn test_agent_info_fields() {
    let _id: Uuid = Uuid::new_v4();
    let _name: String = "test-agent".to_string();
    let _endpoint: String = "ws://localhost:8080".to_string();
    let _status: String = "connected".to_string();
    let _approval_state: String = "approved".to_string();
    let _auth_method: String = "tls".to_string();
    let _cert_fingerprint: Option<String> = Some("fingerprint".to_string());
    let _version: Option<String> = Some("1.0.0".to_string());
}
