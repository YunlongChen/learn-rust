//! Agent management client wrapper for domain-manager.
//!
//! This module provides a convenient wrapper around the agent-management-client crate
//! for interacting with the agent management service.

use anyhow::Result;
use agent_management_client::client::agent_management;
use agent_management_client::{AgentManagementClient as InnerClient, Config};

/// Wrapper client for the agent-management service.
///
/// This provides a convenient interface for domain-manager to communicate
/// with the agent management service.
#[derive(Debug)]
pub struct AgentManagementClient {
    inner: InnerClient,
}

impl AgentManagementClient {
    /// Create a new client connected to the specified endpoint.
    pub async fn new(endpoint: impl Into<String>) -> Result<Self> {
        let config = Config::new(endpoint);
        let inner = InnerClient::connect(config).await?;
        Ok(Self { inner })
    }

    /// Get an agent by its ID.
    pub async fn get_agent(&mut self, agent_id: &str) -> Result<agent_management::Agent> {
        self.inner.get_agent(agent_id).await
    }

    /// List agents with optional filtering.
    pub async fn list_agents(
        &mut self,
        status_filter: Option<&str>,
        approval_status_filter: Option<&str>,
        page_size: i32,
    ) -> Result<agent_management::ListAgentsResponse> {
        self.inner.list_agents(status_filter, approval_status_filter, page_size).await
    }

    /// Approve a pending agent.
    pub async fn approve_agent(
        &mut self,
        agent_id: &str,
        approved_by: Option<&str>,
        notes: Option<&str>,
    ) -> Result<agent_management::Agent> {
        self.inner.approve_agent(agent_id, approved_by, notes).await
    }

    /// Deny a pending agent.
    pub async fn deny_agent(
        &mut self,
        agent_id: &str,
        reason: &str,
        denied_by: Option<&str>,
    ) -> Result<agent_management::Agent> {
        self.inner.deny_agent(agent_id, reason, denied_by).await
    }
}
