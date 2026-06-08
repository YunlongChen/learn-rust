//! gRPC server implementation for agent management service

use tonic::{Request, Response, Status};
use async_trait::async_trait;
use std::pin::Pin;
use futures_util::Stream;
use futures_util::stream;
use chrono::Utc;
use uuid::Uuid;
use prost::alloc::string::ToString;

pub use crate::generated::agent_management::agent_management_service_server::{
    AgentManagementService, AgentManagementServiceServer,
};

pub use crate::generated::agent_management::{
    Empty, RegisterRequest, RegisterResponse, Agent, GetAgentRequest,
    ListAgentsRequest, ListAgentsResponse, UpdateAgentRequest, DeleteAgentRequest,
    ApproveRequest, DenyRequest, GetSystemInfoRequest, SystemInfo, OsInfo,
    EnvironmentInfo, ProcessInfo, NetworkInfo, NetworkInterface, NetworkConnection,
    ResourceInfo, CpuInfo, MemoryInfo, DiskInfo, StreamEventsRequest, AgentEvent,
    GetAgentHealthRequest, HealthScore, StreamHealthRequest, GetLifecycleRequest,
    LifecycleEventsResponse, StreamLifecycleRequest,
};

use crate::service::Service;

/// gRPC server for agent management service
#[derive(Debug, Clone)]
pub struct GrpcServer {
    service: Service,
}

impl GrpcServer {
    /// Create a new gRPC server instance
    pub fn new(service: Service) -> Self {
        Self { service }
    }
}

impl Default for GrpcServer {
    fn default() -> Self {
        panic!("GrpcServer::default() is not supported, use GrpcServer::new(service)")
    }
}

#[async_trait]
impl AgentManagementService for GrpcServer {
    type StreamAgentEventsStream = Pin<Box<dyn Stream<Item = Result<AgentEvent, Status>> + Send>>;
    type StreamAgentHealthStream = Pin<Box<dyn Stream<Item = Result<HealthScore, Status>> + Send>>;
    type StreamLifecycleEventsStream = Pin<Box<dyn Stream<Item = Result<AgentEvent, Status>> + Send>>;

    // Agent CRUD operations

    async fn register_agent(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();

        let id = Uuid::new_v4();
        let now = Utc::now();

        let input = crate::service::agent::CreateAgentInput {
            id,
            name: req.name.to_string(),
            endpoint: String::new(), // Agent endpoint not provided in register request
            status: "pending".to_string(),
            approval_state: "pending".to_string(),
            capabilities: serde_json::Value::Null.into(),
            cert_fingerprint: None,
            auth_method: "unknown".to_string(),
            version: if req.version.is_empty() { None } else { Some(req.version.to_string()) },
            registered_at: Some(now),
            last_seen_at: Some(now),
        };

        let agent_info = self.service.agent_service.create_agent(input)
            .await
            .map_err(|e| Status::internal(format!("Failed to create agent: {}", e)))?;

        let proto_agent = agent_to_proto(&agent_info);
        let approved = agent_info.approval_state == "approved";

        Ok(Response::new(RegisterResponse {
            agent: Some(proto_agent),
            approved,
        }))
    }

    async fn get_agent(
        &self,
        request: Request<GetAgentRequest>,
    ) -> Result<Response<Agent>, Status> {
        let req = request.into_inner();

        let agent_id = Uuid::parse_str(&req.agent_id)
            .map_err(|_| Status::invalid_argument("Invalid agent_id format"))?;

        let agent_info = self.service.agent_service.get_agent(agent_id)
            .await
            .map_err(|e| Status::internal(format!("Failed to get agent: {}", e)))?;

        match agent_info {
            Some(info) => Ok(Response::new(agent_to_proto(&info))),
            None => Err(Status::not_found("Agent not found")),
        }
    }

    async fn list_agents(
        &self,
        request: Request<ListAgentsRequest>,
    ) -> Result<Response<ListAgentsResponse>, Status> {
        let req = request.into_inner();

        let filters = crate::service::agent::AgentFilters {
            status: req.status_filter.map(|s| s.to_string()),
            approval_state: req.approval_status_filter.map(|s| s.to_string()),
        };

        let agents = self.service.agent_service.list_agents(filters)
            .await
            .map_err(|e| Status::internal(format!("Failed to list agents: {}", e)))?;

        let proto_agents: Vec<Agent> = agents.iter().map(agent_to_proto).collect();

        Ok(Response::new(ListAgentsResponse {
            agents: proto_agents,
            next_page_token: None,
        }))
    }

    async fn update_agent(
        &self,
        request: Request<UpdateAgentRequest>,
    ) -> Result<Response<Agent>, Status> {
        let req = request.into_inner();

        let agent_id = Uuid::parse_str(&req.agent_id)
            .map_err(|_| Status::invalid_argument("Invalid agent_id format"))?;

        let input = crate::service::agent::UpdateAgentInput {
            name: req.name.map(|s| s.to_string()),
            endpoint: None,
            status: req.status.map(|s| s.to_string()),
            approval_state: None,
            capabilities: None,
            cert_fingerprint: None,
            auth_method: None,
            version: req.version.map(|s| s.to_string()),
            registered_at: None,
            last_seen_at: None,
        };

        let agent_info = self.service.agent_service.update_agent(agent_id, input)
            .await
            .map_err(|e| Status::internal(format!("Failed to update agent: {}", e)))?;

        match agent_info {
            Some(info) => Ok(Response::new(agent_to_proto(&info))),
            None => Err(Status::not_found("Agent not found")),
        }
    }

    async fn delete_agent(
        &self,
        request: Request<DeleteAgentRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();

        let agent_id = Uuid::parse_str(&req.agent_id)
            .map_err(|_| Status::invalid_argument("Invalid agent_id format"))?;

        self.service.agent_service.delete_agent(agent_id)
            .await
            .map_err(|e| Status::internal(format!("Failed to delete agent: {}", e)))?;

        Ok(Response::new(Empty {}))
    }

    // Approval management

    async fn approve_agent(
        &self,
        request: Request<ApproveRequest>,
    ) -> Result<Response<Agent>, Status> {
        let req = request.into_inner();

        let agent_id = Uuid::parse_str(&req.agent_id)
            .map_err(|_| Status::invalid_argument("Invalid agent_id format"))?;

        let approved_by = req.approved_by.as_ref().map(|s| s.to_string()).unwrap_or_default();

        self.service.agent_service.approve_agent(agent_id, approved_by)
            .await
            .map_err(|e| Status::internal(format!("Failed to approve agent: {}", e)))?;

        // Get updated agent
        let agent_info = self.service.agent_service.get_agent(agent_id)
            .await
            .map_err(|e| Status::internal(format!("Failed to get updated agent: {}", e)))?;

        match agent_info {
            Some(info) => Ok(Response::new(agent_to_proto(&info))),
            None => Err(Status::not_found("Agent not found after approval")),
        }
    }

    async fn deny_agent(
        &self,
        request: Request<DenyRequest>,
    ) -> Result<Response<Agent>, Status> {
        let req = request.into_inner();

        let agent_id = Uuid::parse_str(&req.agent_id)
            .map_err(|_| Status::invalid_argument("Invalid agent_id format"))?;

        let denied_by = req.denied_by.as_ref().map(|s| s.to_string()).unwrap_or_default();

        self.service.agent_service.deny_agent(agent_id, req.reason.to_string(), denied_by)
            .await
            .map_err(|e| Status::internal(format!("Failed to deny agent: {}", e)))?;

        // Get updated agent
        let agent_info = self.service.agent_service.get_agent(agent_id)
            .await
            .map_err(|e| Status::internal(format!("Failed to get updated agent: {}", e)))?;

        match agent_info {
            Some(info) => Ok(Response::new(agent_to_proto(&info))),
            None => Err(Status::not_found("Agent not found after denial")),
        }
    }

    // Diagnostic info

    async fn get_agent_system_info(
        &self,
        request: Request<GetSystemInfoRequest>,
    ) -> Result<Response<SystemInfo>, Status> {
        let req = request.into_inner();

        let agent_id = Uuid::parse_str(&req.agent_id)
            .map_err(|_| Status::invalid_argument("Invalid agent_id format"))?;

        let system_info = self.service.diagnostic_service.get_system_info(agent_id)
            .await
            .map_err(|e| Status::internal(format!("Failed to get system info: {}", e)))?;

        match system_info {
            Some(info) => Ok(Response::new(system_info_to_proto(&info))),
            None => Err(Status::not_found("System info not found for agent")),
        }
    }

    // Event streaming

    async fn stream_agent_events(
        &self,
        _request: Request<StreamEventsRequest>,
    ) -> Result<Response<Self::StreamAgentEventsStream>, Status> {
        // Return empty stream for now - can be enhanced later with actual event streaming
        let output_stream = stream::empty();
        Ok(Response::new(Box::pin(output_stream)))
    }

    // Health scoring

    async fn get_agent_health(
        &self,
        request: Request<GetAgentHealthRequest>,
    ) -> Result<Response<HealthScore>, Status> {
        let req = request.into_inner();

        let agent_id = Uuid::parse_str(&req.agent_id)
            .map_err(|_| Status::invalid_argument("Invalid agent_id format"))?;

        let health_score = self.service.health_service.get_latest_score(agent_id)
            .await
            .map_err(|e| Status::internal(format!("Failed to get health score: {}", e)))?;

        match health_score {
            Some(score) => Ok(Response::new(health_score_to_proto(&score))),
            None => Err(Status::not_found("Health score not found for agent")),
        }
    }

    async fn stream_agent_health(
        &self,
        _request: Request<StreamHealthRequest>,
    ) -> Result<Response<Self::StreamAgentHealthStream>, Status> {
        // Return empty stream for now - can be enhanced later with actual health streaming
        let output_stream = stream::empty();
        Ok(Response::new(Box::pin(output_stream)))
    }

    // Lifecycle events

    async fn get_agent_lifecycle_events(
        &self,
        request: Request<GetLifecycleRequest>,
    ) -> Result<Response<LifecycleEventsResponse>, Status> {
        let req = request.into_inner();

        let agent_id = Uuid::parse_str(&req.agent_id)
            .map_err(|_| Status::invalid_argument("Invalid agent_id format"))?;

        let events = self.service.lifecycle_service.get_events_for_agent(agent_id)
            .await
            .map_err(|e| Status::internal(format!("Failed to get lifecycle events: {}", e)))?;

        let proto_events: Vec<AgentEvent> = events.iter().map(lifecycle_event_to_proto).collect();

        Ok(Response::new(LifecycleEventsResponse {
            events: proto_events,
        }))
    }

    async fn stream_lifecycle_events(
        &self,
        _request: Request<StreamLifecycleRequest>,
    ) -> Result<Response<Self::StreamLifecycleEventsStream>, Status> {
        // Return empty stream for now - can be enhanced later with actual lifecycle streaming
        let output_stream = stream::empty();
        Ok(Response::new(Box::pin(output_stream)))
    }
}

/// Create and configure the gRPC server
pub async fn create_grpc_server(
    addr: &str,
    service: Service,
) -> Result<AgentManagementServiceServer<GrpcServer>, Box<dyn std::error::Error>> {
    let _ = addr.parse::<std::net::SocketAddr>()?;
    let server = GrpcServer::new(service);
    let server = AgentManagementServiceServer::new(server);
    Ok(server)
}

// Helper functions to convert domain types to proto types

fn agent_to_proto(info: &crate::service::agent::AgentInfo) -> Agent {
    Agent {
        id: info.id.to_string(),
        name: info.name.clone(),
        version: info.version.clone().unwrap_or_default(),
        status: info.status.clone(),
        approval_status: info.approval_state.clone(),
        owner_id: None,
        description: None,
        registered_at: info.registered_at
            .map(|dt| dt.timestamp())
            .unwrap_or(0),
        last_seen_at: info.last_seen_at
            .map(|dt| dt.timestamp())
            .unwrap_or(0),
        approved_at: None,
        denied_at: None,
        denial_reason: None,
    }
}

fn system_info_to_proto(model: &crate::storage::entities::system_info::Model) -> SystemInfo {
    SystemInfo {
        agent_id: model.agent_id.to_string(),
        os_info: model.os_info.as_ref().map(|j| OsInfo {
            os_name: j.get("os_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            os_version: j.get("os_version").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            kernel_version: j.get("kernel_version").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            hostname: j.get("hostname").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            arch: j.get("arch").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        }),
        environment_info: model.environment_info.as_ref().map(|j| EnvironmentInfo {
            user: j.get("user").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            home_dir: j.get("home_dir").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            cwd: j.get("cwd").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            env_vars: j.get("env_vars")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
        }),
        process_info: model.process_info.as_ref().map(|j| ProcessInfo {
            pid: j.get("pid").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            parent_pid: j.get("parent_pid").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            name: j.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            exe_path: j.get("exe_path").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            start_time: j.get("start_time").and_then(|v| v.as_i64()).unwrap_or(0),
            uptime_seconds: j.get("uptime_seconds").and_then(|v| v.as_i64()).unwrap_or(0),
        }),
        network_info: model.network_info.as_ref().map(|j| {
            let interfaces = j.get("interfaces")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|iface| {
                    Some(NetworkInterface {
                        name: iface.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        mac_address: iface.get("mac_address").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        ipv4_addresses: iface.get("ipv4_addresses")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                            .unwrap_or_default(),
                        ipv6_addresses: iface.get("ipv6_addresses")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                            .unwrap_or_default(),
                        is_up: iface.get("is_up").and_then(|v| v.as_bool()).unwrap_or(false),
                    })
                }).collect())
                .unwrap_or_default();

            let connections = j.get("connections")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|conn| {
                    Some(NetworkConnection {
                        protocol: conn.get("protocol").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        local_addr: conn.get("local_addr").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        remote_addr: conn.get("remote_addr").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        state: conn.get("state").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    })
                }).collect())
                .unwrap_or_default();

            NetworkInfo {
                interfaces,
                connections,
            }
        }),
        resource_info: model.resource_info.as_ref().map(|j| {
            ResourceInfo {
                cpu_info: j.get("cpu_info").map(|cpu| CpuInfo {
                    model: cpu.get("model").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    core_count: cpu.get("core_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                    usage_percent: cpu.get("usage_percent").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                }),
                memory_info: j.get("memory_info").map(|mem| MemoryInfo {
                    total_bytes: mem.get("total_bytes").and_then(|v| v.as_u64()).unwrap_or(0),
                    used_bytes: mem.get("used_bytes").and_then(|v| v.as_u64()).unwrap_or(0),
                    available_bytes: mem.get("available_bytes").and_then(|v| v.as_u64()).unwrap_or(0),
                    usage_percent: mem.get("usage_percent").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                }),
                disk_info: j.get("disks")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|disk| {
                        Some(DiskInfo {
                            mount_point: disk.get("mount_point").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            fs_type: disk.get("fs_type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            total_bytes: disk.get("total_bytes").and_then(|v| v.as_u64()).unwrap_or(0),
                            used_bytes: disk.get("used_bytes").and_then(|v| v.as_u64()).unwrap_or(0),
                            available_bytes: disk.get("available_bytes").and_then(|v| v.as_u64()).unwrap_or(0),
                            usage_percent: disk.get("usage_percent").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                        })
                    }).collect())
                    .unwrap_or_default(),
            }
        }),
        collected_at: model.reported_at.timestamp(),
    }
}

fn health_score_to_proto(model: &crate::storage::entities::health_score::Model) -> HealthScore {
    HealthScore {
        agent_id: model.agent_id.to_string(),
        score: model.overall_score as i32,
        status: if model.overall_score >= 80.0 {
            "healthy".to_string()
        } else if model.overall_score >= 50.0 {
            "degraded".to_string()
        } else {
            "unhealthy".to_string()
        },
        factors: vec![],
        calculated_at: model.scored_at.timestamp(),
    }
}

fn lifecycle_event_to_proto(event: &domain_agent_protocol::lifecycle::LifecycleEvent) -> AgentEvent {
    AgentEvent {
        event_id: event.event_id.to_string(),
        agent_id: event.agent_id.to_string(),
        event_type: event.event_type.to_string(),
        payload: event.reason.clone().unwrap_or_default(),
        timestamp: event.timestamp.timestamp(),
    }
}

#[cfg(test)]
mod tests {
    // Tests require a real Service instance, so we skip inline testing here.
    // Integration tests should be used for testing GrpcServer with actual service dependencies.
}
