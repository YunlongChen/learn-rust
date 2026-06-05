use anyhow::Result;
use tonic::transport::Channel;

// Re-export generated types
pub mod agent_management {
    tonic::include_proto!("agent_management");
}

use agent_management::{
    agent_management_service_client::AgentManagementServiceClient,
    ApproveRequest, DenyRequest, GetAgentRequest, ListAgentsRequest,
};

/// Configuration for connecting to the agent-management gRPC service
#[derive(Debug, Clone)]
pub struct Config {
    /// gRPC endpoint address (e.g., "http://localhost:50051")
    pub endpoint: String,
}

impl Config {
    /// Create a new config with the given endpoint
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }
}

/// Client for the agent-management gRPC service
#[derive(Debug)]
pub struct AgentManagementClient {
    inner: AgentManagementServiceClient<Channel>,
}

impl AgentManagementClient {
    /// Connect to the agent-management service
    pub async fn connect(config: Config) -> Result<Self> {
        let inner = AgentManagementServiceClient::connect(config.endpoint).await?;
        Ok(Self { inner })
    }

    /// Get an agent by ID
    pub async fn get_agent(&mut self, agent_id: &str) -> Result<agent_management::Agent> {
        let request = GetAgentRequest {
            agent_id: agent_id.to_string(),
        };
        let response = self.inner.get_agent(request).await?;
        Ok(response.into_inner())
    }

    /// List agents with optional filtering
    pub async fn list_agents(
        &mut self,
        status_filter: Option<&str>,
        approval_status_filter: Option<&str>,
        page_size: i32,
    ) -> Result<agent_management::ListAgentsResponse> {
        let request = ListAgentsRequest {
            status_filter: status_filter.map(String::from),
            approval_status_filter: approval_status_filter.map(String::from),
            page_size,
            page_token: None,
        };
        let response = self.inner.list_agents(request).await?;
        Ok(response.into_inner())
    }

    /// Approve a pending agent
    pub async fn approve_agent(
        &mut self,
        agent_id: &str,
        approved_by: Option<&str>,
        notes: Option<&str>,
    ) -> Result<agent_management::Agent> {
        let request = ApproveRequest {
            agent_id: agent_id.to_string(),
            approved_by: approved_by.map(String::from),
            notes: notes.map(String::from),
        };
        let response = self.inner.approve_agent(request).await?;
        Ok(response.into_inner())
    }

    /// Deny a pending agent
    pub async fn deny_agent(
        &mut self,
        agent_id: &str,
        reason: &str,
        denied_by: Option<&str>,
    ) -> Result<agent_management::Agent> {
        let request = DenyRequest {
            agent_id: agent_id.to_string(),
            reason: reason.to_string(),
            denied_by: denied_by.map(String::from),
        };
        let response = self.inner.deny_agent(request).await?;
        Ok(response.into_inner())
    }
}
