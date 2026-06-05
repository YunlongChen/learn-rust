//! REST API server implementation for agent management service

use axum::{
    Router,
    routing::{get, patch, post, delete},
    extract::{Path, State},
    response::{IntoResponse, Json, Response},
    http::StatusCode,
};
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::service::agent::{AgentFilters, UpdateAgentInput};
use crate::service::Service;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub service: Service,
}

/// REST server configuration
#[derive(Clone)]
pub struct RestConfig {
    pub addr: String,
}

impl Default for RestConfig {
    fn default() -> Self {
        Self {
            addr: "0.0.0.0:8080".to_string(),
        }
    }
}

// Request/Response types

#[derive(Serialize, Deserialize)]
pub struct AgentResponse {
    pub id: String,
    pub name: String,
    #[serde(rename = "agentType")]
    pub agent_type: String,
    pub status: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Serialize, Deserialize)]
pub struct ListAgentsResponse {
    pub agents: Vec<AgentResponse>,
    pub total: usize,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateAgentRequest {
    pub name: Option<String>,
    pub status: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SystemInfoResponse {
    #[serde(rename = "osInfo")]
    pub os_info: OsInfoResponse,
    #[serde(rename = "environmentInfo")]
    pub environment_info: EnvironmentInfoResponse,
    #[serde(rename = "processInfo")]
    pub process_info: ProcessInfoResponse,
    #[serde(rename = "networkInfo")]
    pub network_info: NetworkInfoResponse,
    #[serde(rename = "resourceInfo")]
    pub resource_info: ResourceInfoResponse,
}

#[derive(Serialize, Deserialize)]
pub struct OsInfoResponse {
    pub os: String,
    #[serde(rename = "osVersion")]
    pub os_version: String,
    pub architecture: String,
    pub hostname: String,
}

#[derive(Serialize, Deserialize)]
pub struct EnvironmentInfoResponse {
    pub rust_version: String,
    pub runtime_version: String,
    pub agent_version: String,
}

#[derive(Serialize, Deserialize)]
pub struct ProcessInfoResponse {
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct NetworkInfoResponse {
    pub interfaces: Vec<NetworkInterfaceResponse>,
    pub connections: Vec<NetworkConnectionResponse>,
}

#[derive(Serialize, Deserialize)]
pub struct NetworkInterfaceResponse {
    pub name: String,
    pub mac_address: String,
    pub ipv4: Vec<String>,
    pub ipv6: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct NetworkConnectionResponse {
    pub protocol: String,
    pub local_address: String,
    pub remote_address: String,
    pub state: String,
}

#[derive(Serialize, Deserialize)]
pub struct ResourceInfoResponse {
    pub cpu: CpuInfoResponse,
    pub memory: MemoryInfoResponse,
    pub disk: Vec<DiskInfoResponse>,
}

#[derive(Serialize, Deserialize)]
pub struct CpuInfoResponse {
    pub physical_cores: u32,
    pub logical_cores: u32,
    pub usage_percent: f32,
}

#[derive(Serialize, Deserialize)]
pub struct MemoryInfoResponse {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
}

#[derive(Serialize, Deserialize)]
pub struct DiskInfoResponse {
    pub name: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
}

#[derive(Serialize, Deserialize)]
pub struct HealthScoreResponse {
    pub score: f32,
    pub components: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct LifecycleEventResponse {
    pub id: String,
    pub event_type: String,
    pub timestamp: String,
    pub details: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct LifecycleEventsResponse {
    pub events: Vec<LifecycleEventResponse>,
}

#[derive(Serialize, Deserialize)]
pub struct QuerySystemInfoRequest {
    pub query: String,
}

// Conversion helpers

fn agent_info_to_response(info: &crate::service::AgentInfo) -> AgentResponse {
    AgentResponse {
        id: info.id.to_string(),
        name: info.name.clone(),
        agent_type: info.name.clone(), // Use name as agent_type
        status: info.status.clone(),
        created_at: info.created_at.to_rfc3339(),
        updated_at: info.updated_at.to_rfc3339(),
    }
}

fn parse_uuid(id: &str) -> Result<Uuid, StatusCode> {
    Uuid::parse_str(id).map_err(|_| StatusCode::BAD_REQUEST)
}

/// Handler for GET /api/v1/agents - list all agents
async fn list_agents(
    State(state): State<Arc<AppState>>,
) -> Response {
    match state.service.agent_service.list_agents(AgentFilters::default()).await {
        Ok(agents) => {
            let response = ListAgentsResponse {
                agents: agents.iter().map(agent_info_to_response).collect(),
                total: agents.len(),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list agents: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to list agents"
            }))).into_response()
        }
    }
}

/// Handler for GET /api/v1/agents/:id - get a specific agent
async fn get_agent(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Response {
    let agent_id = match parse_uuid(&id) {
        Ok(id) => id,
        Err(status) => return status.into_response(),
    };

    match state.service.agent_service.get_agent(agent_id).await {
        Ok(Some(agent)) => {
            (StatusCode::OK, Json(agent_info_to_response(&agent))).into_response()
        }
        Ok(None) => {
            (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "Agent not found"
            }))).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get agent: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to get agent"
            }))).into_response()
        }
    }
}

/// Handler for PATCH /api/v1/agents/:id - update an agent
async fn update_agent(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<UpdateAgentRequest>,
) -> Response {
    let agent_id = match parse_uuid(&id) {
        Ok(id) => id,
        Err(status) => return status.into_response(),
    };

    let input = UpdateAgentInput {
        name: body.name,
        endpoint: None,
        status: body.status,
        approval_state: None,
        capabilities: None,
        cert_fingerprint: None,
        auth_method: None,
        version: None,
        registered_at: None,
        last_seen_at: None,
    };

    match state.service.agent_service.update_agent(agent_id, input).await {
        Ok(Some(agent)) => {
            (StatusCode::OK, Json(agent_info_to_response(&agent))).into_response()
        }
        Ok(None) => {
            (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "Agent not found"
            }))).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to update agent: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to update agent"
            }))).into_response()
        }
    }
}

/// Handler for DELETE /api/v1/agents/:id - delete an agent
async fn delete_agent(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Response {
    let agent_id = match parse_uuid(&id) {
        Ok(id) => id,
        Err(status) => return status.into_response(),
    };

    match state.service.agent_service.delete_agent(agent_id).await {
        Ok(true) => {
            (StatusCode::NO_CONTENT).into_response()
        }
        Ok(false) => {
            (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "Agent not found"
            }))).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to delete agent: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to delete agent"
            }))).into_response()
        }
    }
}

/// Handler for POST /api/v1/agents/:id/approve - approve an agent
async fn approve_agent(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Response {
    let agent_id = match parse_uuid(&id) {
        Ok(id) => id,
        Err(status) => return status.into_response(),
    };

    match state.service.agent_service.approve_agent(agent_id, "rest-api".to_string()).await {
        Ok(Some(agent)) => {
            (StatusCode::OK, Json(agent_info_to_response(&agent))).into_response()
        }
        Ok(None) => {
            (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "Agent not found"
            }))).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to approve agent: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to approve agent"
            }))).into_response()
        }
    }
}

/// Handler for POST /api/v1/agents/:id/deny - deny an agent
async fn deny_agent(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Response {
    let agent_id = match parse_uuid(&id) {
        Ok(id) => id,
        Err(status) => return status.into_response(),
    };

    match state.service.agent_service.deny_agent(agent_id, "denied via REST API".to_string(), "rest-api".to_string()).await {
        Ok(Some(agent)) => {
            (StatusCode::OK, Json(agent_info_to_response(&agent))).into_response()
        }
        Ok(None) => {
            (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "Agent not found"
            }))).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to deny agent: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to deny agent"
            }))).into_response()
        }
    }
}

/// Handler for GET /api/v1/agents/:id/system-info - get system info
async fn get_system_info(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Response {
    let agent_id = match parse_uuid(&id) {
        Ok(id) => id,
        Err(status) => return status.into_response(),
    };

    match state.service.diagnostic_service.get_system_info(agent_id).await {
        Ok(Some(info)) => {
            let os_info = info.os_info.and_then(|j| serde_json::from_value(j).ok()).unwrap_or(OsInfoResponse {
                os: "Unknown".to_string(),
                os_version: "Unknown".to_string(),
                architecture: "Unknown".to_string(),
                hostname: "Unknown".to_string(),
            });
            let environment_info = info.environment_info.and_then(|j| serde_json::from_value(j).ok()).unwrap_or(EnvironmentInfoResponse {
                rust_version: "Unknown".to_string(),
                runtime_version: "Unknown".to_string(),
                agent_version: "Unknown".to_string(),
            });
            let process_info = info.process_info.and_then(|j| serde_json::from_value(j).ok()).unwrap_or(ProcessInfoResponse {
                pid: 0,
                parent_pid: None,
                name: "Unknown".to_string(),
            });
            let network_info = info.network_info.and_then(|j| serde_json::from_value(j).ok()).unwrap_or(NetworkInfoResponse {
                interfaces: vec![],
                connections: vec![],
            });
            let resource_info = info.resource_info.and_then(|j| serde_json::from_value(j).ok()).unwrap_or(ResourceInfoResponse {
                cpu: CpuInfoResponse {
                    physical_cores: 0,
                    logical_cores: 0,
                    usage_percent: 0.0,
                },
                memory: MemoryInfoResponse {
                    total_bytes: 0,
                    used_bytes: 0,
                    available_bytes: 0,
                },
                disk: vec![],
            });

            let response = SystemInfoResponse {
                os_info,
                environment_info,
                process_info,
                network_info,
                resource_info,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => {
            (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "System info not found"
            }))).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get system info: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to get system info"
            }))).into_response()
        }
    }
}

/// Handler for POST /api/v1/agents/:id/system-info/query - query system info
async fn query_system_info(
    Path(id): Path<String>,
    Json(_body): Json<QuerySystemInfoRequest>,
) -> Response {
    (StatusCode::NOT_IMPLEMENTED, Json(serde_json::json!({
        "message": "query_system_info not implemented",
        "id": id
    }))).into_response()
}

/// Handler for GET /api/v1/agents/:id/health - get health score
async fn get_health_score(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Response {
    let agent_id = match parse_uuid(&id) {
        Ok(id) => id,
        Err(status) => return status.into_response(),
    };

    match state.service.health_service.get_latest_score(agent_id).await {
        Ok(Some(score)) => {
            let response = HealthScoreResponse {
                score: score.overall_score as f32,
                components: score.component_scores.unwrap_or_default(),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => {
            (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "Health score not found"
            }))).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get health score: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to get health score"
            }))).into_response()
        }
    }
}

/// Handler for GET /api/v1/agents/:id/lifecycle - get lifecycle events
async fn get_lifecycle_events(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Response {
    let agent_id = match parse_uuid(&id) {
        Ok(id) => id,
        Err(status) => return status.into_response(),
    };

    match state.service.lifecycle_service.get_events_for_agent(agent_id).await {
        Ok(events) => {
            let event_responses: Vec<LifecycleEventResponse> = events.iter().map(|e| {
                let details = serde_json::json!({
                    "reason": e.reason,
                    "metadata": e.metadata,
                    "triggered_by": e.triggered_by,
                    "source": format!("{:?}", e.source),
                });
                LifecycleEventResponse {
                    id: e.event_id.to_string(),
                    event_type: format!("{:?}", e.event_type),
                    timestamp: e.timestamp.to_rfc3339(),
                    details,
                }
            }).collect();
            let response = LifecycleEventsResponse {
                events: event_responses,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get lifecycle events: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to get lifecycle events"
            }))).into_response()
        }
    }
}

/// Create and configure the REST server
pub async fn create_rest_server(
    config: RestConfig,
    state: AppState,
) -> Result<Router, Box<dyn std::error::Error + Send + Sync>> {
    let app = Router::new()
        .route("/api/v1/agents", get(list_agents))
        .route("/api/v1/agents/{id}", get(get_agent))
        .route("/api/v1/agents/{id}", patch(update_agent))
        .route("/api/v1/agents/{id}", delete(delete_agent))
        .route("/api/v1/agents/{id}/approve", post(approve_agent))
        .route("/api/v1/agents/{id}/deny", post(deny_agent))
        .route("/api/v1/agents/{id}/system-info", get(get_system_info))
        .route("/api/v1/agents/{id}/system-info/query", post(query_system_info))
        .route("/api/v1/agents/{id}/health", get(get_health_score))
        .route("/api/v1/agents/{id}/lifecycle", get(get_lifecycle_events))
        .with_state(Arc::new(state));

    tracing::info!("REST server configured on {}", config.addr);
    Ok(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rest_config_default() {
        let config = RestConfig::default();
        assert_eq!(config.addr, "0.0.0.0:8080");
    }

    #[test]
    fn test_agent_response_serialization() {
        let response = AgentResponse {
            id: "test-id".to_string(),
            name: "Test Agent".to_string(),
            agent_type: "TestType".to_string(),
            status: "active".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"id\":\"test-id\""));
        assert!(json.contains("\"name\":\"Test Agent\""));
    }

    #[test]
    fn test_parse_uuid_valid() {
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(parse_uuid(valid_uuid).is_ok());
    }

    #[test]
    fn test_parse_uuid_invalid() {
        let invalid_uuid = "not-a-uuid";
        assert!(parse_uuid(invalid_uuid).is_err());
    }
}
