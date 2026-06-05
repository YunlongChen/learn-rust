//! Agent service for agent management
//!
//! This module provides the AgentService for managing agents.

use chrono::{DateTime, Utc};
use sea_orm::ActiveModelTrait;
use sea_orm::entity::prelude::*;
use sea_orm::{Set, QueryOrder, EntityTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::storage::Database;
use crate::storage::entities::agent::{Entity as AgentEntity, ActiveModel, Model};

/// Input for creating a new agent.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateAgentInput {
    pub id: Uuid,
    pub name: String,
    pub endpoint: String,
    pub status: String,
    pub approval_state: String,
    pub capabilities: Json,
    pub cert_fingerprint: Option<String>,
    pub auth_method: String,
    pub version: Option<String>,
    pub registered_at: Option<DateTime<Utc>>,
    pub last_seen_at: Option<DateTime<Utc>>,
}

/// Input for updating an existing agent.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateAgentInput {
    pub name: Option<String>,
    pub endpoint: Option<String>,
    pub status: Option<String>,
    pub approval_state: Option<String>,
    pub capabilities: Option<Json>,
    pub cert_fingerprint: Option<String>,
    pub auth_method: Option<String>,
    pub version: Option<String>,
    pub registered_at: Option<DateTime<Utc>>,
    pub last_seen_at: Option<DateTime<Utc>>,
}

/// Filters for querying agents.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AgentFilters {
    pub status: Option<String>,
    pub approval_state: Option<String>,
}

/// Basic agent information returned by get_agent and list_agents.
#[derive(Debug, Clone, Serialize)]
pub struct AgentInfo {
    pub id: Uuid,
    pub name: String,
    pub endpoint: String,
    pub status: String,
    pub approval_state: String,
    pub capabilities: Json,
    pub cert_fingerprint: Option<String>,
    pub auth_method: String,
    pub version: Option<String>,
    pub registered_at: Option<DateTime<Utc>>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Model> for AgentInfo {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            endpoint: model.endpoint,
            status: model.status,
            approval_state: model.approval_state,
            capabilities: model.capabilities,
            cert_fingerprint: model.cert_fingerprint,
            auth_method: model.auth_method,
            version: model.version,
            registered_at: model.registered_at,
            last_seen_at: model.last_seen_at,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

/// Service for managing agents.
///
/// This service provides methods to create, read, update, and delete agents.
#[derive(Clone)]
pub struct AgentService {
    db: Database,
}

impl AgentService {
    /// Creates a new AgentService with the given database.
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Create a new agent.
    pub async fn create_agent(
        &self,
        input: CreateAgentInput,
    ) -> Result<AgentInfo, sea_orm::DbErr> {
        let now = Utc::now();
        let active_model = ActiveModel {
            id: Set(input.id),
            name: Set(input.name),
            endpoint: Set(input.endpoint),
            status: Set(input.status),
            approval_state: Set(input.approval_state),
            capabilities: Set(input.capabilities),
            cert_fingerprint: Set(input.cert_fingerprint),
            auth_method: Set(input.auth_method),
            version: Set(input.version),
            registered_at: Set(input.registered_at),
            last_seen_at: Set(input.last_seen_at),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = active_model.insert(self.db.get_conn()).await?;
        Ok(result.into())
    }

    /// Get an agent by ID.
    pub async fn get_agent(
        &self,
        agent_id: Uuid,
    ) -> Result<Option<AgentInfo>, sea_orm::DbErr> {
        let agent = AgentEntity::find_by_id(agent_id)
            .one(self.db.get_conn())
            .await?;
        Ok(agent.map(|a| a.into()))
    }

    /// List agents with optional filters.
    pub async fn list_agents(
        &self,
        filters: AgentFilters,
    ) -> Result<Vec<AgentInfo>, sea_orm::DbErr> {
        use crate::storage::entities::agent::Column;

        let mut query = AgentEntity::find();

        if let Some(status) = &filters.status {
            query = query.filter(Column::Status.eq(status));
        }

        if let Some(approval_state) = &filters.approval_state {
            query = query.filter(Column::ApprovalState.eq(approval_state));
        }

        let agents = query
            .order_by_desc(Column::CreatedAt)
            .all(self.db.get_conn())
            .await?;

        Ok(agents.into_iter().map(|a| a.into()).collect())
    }

    /// Update an agent by ID.
    pub async fn update_agent(
        &self,
        agent_id: Uuid,
        input: UpdateAgentInput,
    ) -> Result<Option<AgentInfo>, sea_orm::DbErr> {
        let agent = AgentEntity::find_by_id(agent_id)
            .one(self.db.get_conn())
            .await?;

        match agent {
            Some(agent) => {
                let mut active_model: ActiveModel = agent.into();
                let now = Utc::now();

                if let Some(name) = input.name {
                    active_model.name = Set(name);
                }
                if let Some(endpoint) = input.endpoint {
                    active_model.endpoint = Set(endpoint);
                }
                if let Some(status) = input.status {
                    active_model.status = Set(status);
                }
                if let Some(approval_state) = input.approval_state {
                    active_model.approval_state = Set(approval_state);
                }
                if let Some(capabilities) = input.capabilities {
                    active_model.capabilities = Set(capabilities);
                }
                if let Some(cert_fingerprint) = input.cert_fingerprint {
                    active_model.cert_fingerprint = Set(Some(cert_fingerprint));
                }
                if let Some(auth_method) = input.auth_method {
                    active_model.auth_method = Set(auth_method);
                }
                if let Some(version) = input.version {
                    active_model.version = Set(Some(version));
                }
                if let Some(registered_at) = input.registered_at {
                    active_model.registered_at = Set(Some(registered_at));
                }
                if let Some(last_seen_at) = input.last_seen_at {
                    active_model.last_seen_at = Set(Some(last_seen_at));
                }

                active_model.updated_at = Set(now);

                let result = active_model.update(self.db.get_conn()).await?;
                Ok(Some(result.into()))
            }
            None => Ok(None),
        }
    }

    /// Delete an agent by ID.
    pub async fn delete_agent(
        &self,
        agent_id: Uuid,
    ) -> Result<bool, sea_orm::DbErr> {
        let agent = AgentEntity::find_by_id(agent_id)
            .one(self.db.get_conn())
            .await?;

        match agent {
            Some(agent) => {
                let active_model: ActiveModel = agent.into();
                active_model.delete(self.db.get_conn()).await?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    /// Approve an agent.
    pub async fn approve_agent(
        &self,
        agent_id: Uuid,
        _approved_by: String,
    ) -> Result<Option<AgentInfo>, sea_orm::DbErr> {
        self.update_agent(agent_id, UpdateAgentInput {
            approval_state: Some("approved".to_string()),
            ..Default::default()
        }).await
    }

    /// Deny an agent.
    pub async fn deny_agent(
        &self,
        agent_id: Uuid,
        _reason: String,
        _denied_by: String,
    ) -> Result<Option<AgentInfo>, sea_orm::DbErr> {
        self.update_agent(agent_id, UpdateAgentInput {
            approval_state: Some("denied".to_string()),
            ..Default::default()
        }).await
    }
}

impl Default for AgentService {
    fn default() -> Self {
        panic!("AgentService::default() is not supported, use AgentService::new(db)")
    }
}
