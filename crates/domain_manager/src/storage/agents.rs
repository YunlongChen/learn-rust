//! Agent 数据访问层
//!
//! 提供 Agent 的数据库操作

use crate::storage::account::Relation;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue, DeleteResult};
use serde::{Deserialize, Serialize};

use crate::agent::model::{Agent, AgentApprovalState, AgentStatus};

/// Agent ActiveModel
#[derive(Clone, Debug, PartialEq, Eq, Hash, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "agents")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    #[sea_orm(column_name = "name")]
    pub name: String,
    #[sea_orm(nullable)]
    pub description: Option<String>,
    #[sea_orm(column_name = "endpoint")]
    pub endpoint: String,
    #[sea_orm(nullable, column_name = "auth_key")]
    pub auth_key: Option<String>,
    #[sea_orm(nullable, column_type = "Json")]
    pub capabilities: Option<Json>,
    #[sea_orm(column_name = "status")]
    pub status: String,
    #[sea_orm(nullable, column_type = "Json")]
    pub tags: Option<Json>,
    #[sea_orm(nullable, column_type = "Json")]
    pub system_info: Option<Json>,
    #[sea_orm(nullable, column_type = "Json")]
    pub connection_info: Option<Json>,
    #[sea_orm(nullable)]
    pub last_heartbeat: Option<chrono::DateTime<chrono::Utc>>,
    #[sea_orm(nullable, column_name = "enabled")]
    pub enabled: bool,
    #[sea_orm(column_name = "approval_state")]
    pub approval_state: String,
    #[sea_orm(nullable, column_name = "agent_key_hash")]
    pub agent_key_hash: Option<String>,
    #[sea_orm(nullable)]
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    #[sea_orm(nullable, column_name = "approved_by")]
    pub approved_by: Option<String>,
    #[sea_orm(nullable)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[sea_orm(nullable)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl ActiveModelBehavior for ActiveModel {}

/// Agent 到 Model 的转换
impl From<Model> for Agent {
    fn from(model: Model) -> Self {
        let status = match model.status.as_str() {
            "online" => AgentStatus::Online,
            "offline" => AgentStatus::Offline,
            "busy" => AgentStatus::Busy,
            "maintenance" => AgentStatus::Maintenance,
            _ => AgentStatus::Offline,
        };

        let approval_state = match model.approval_state.as_str() {
            "pending" => AgentApprovalState::Pending,
            "approved" => AgentApprovalState::Approved,
            "denied" => AgentApprovalState::Denied,
            _ => AgentApprovalState::Pending,
        };

        let capabilities: Vec<crate::agent::model::Capability> = model.capabilities
            .and_then(|json| serde_json::from_value(json).ok())
            .unwrap_or_default();

        let tags: Vec<String> = model.tags
            .and_then(|json| serde_json::from_value(json).ok())
            .unwrap_or_default();

        Agent {
            id: model.id,
            name: model.name,
            description: model.description,
            endpoint: model.endpoint,
            auth_key: model.auth_key,
            capabilities,
            status,
            tags,
            system_info: None,
            connection_info: None,
            last_heartbeat: model.last_heartbeat,
            enabled: model.enabled,
            approval_state,
            agent_key_hash: model.agent_key_hash,
            approved_at: model.approved_at,
            approved_by: model.approved_by,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

/// Agent 到 ActiveModel 的转换
impl From<Agent> for ActiveModel {
    fn from(agent: Agent) -> Self {
        ActiveModel {
            id: ActiveValue::Set(agent.id),
            name: ActiveValue::Set(agent.name),
            description: ActiveValue::Set(agent.description),
            endpoint: ActiveValue::Set(agent.endpoint),
            auth_key: ActiveValue::Set(agent.auth_key),
            capabilities: ActiveValue::Set(serde_json::to_value(&agent.capabilities).ok()),
            status: ActiveValue::Set(agent.status.to_string()),
            tags: ActiveValue::Set(serde_json::to_value(&agent.tags).ok()),
            system_info: ActiveValue::Set(None),
            connection_info: ActiveValue::Set(None),
            last_heartbeat: ActiveValue::Set(agent.last_heartbeat),
            enabled: ActiveValue::Set(agent.enabled),
            approval_state: ActiveValue::Set(agent.approval_state.to_string()),
            agent_key_hash: ActiveValue::Set(agent.agent_key_hash),
            approved_at: ActiveValue::Set(agent.approved_at),
            approved_by: ActiveValue::Set(agent.approved_by),
            created_at: ActiveValue::Set(agent.created_at),
            updated_at: ActiveValue::Set(agent.updated_at),
        }
    }
}

/// Repository 函数
pub async fn find_agent_by_id(db: &DbConn, id: Uuid) -> Result<Option<Agent>, DbErr> {
    let agent = Entity::find_by_id(id).one(db).await?;
    Ok(agent.map(|m| m.into()))
}

pub async fn find_all_agents(db: &DbConn) -> Result<Vec<Agent>, DbErr> {
    let agents = Entity::find().all(db).await?;
    Ok(agents.into_iter().map(|m| m.into()).collect())
}

pub async fn find_online_agents(db: &DbConn) -> Result<Vec<Agent>, DbErr> {
    let agents = Entity::find()
        .filter(Column::Status.eq("online"))
        .all(db)
        .await?;
    Ok(agents.into_iter().map(|m| m.into()).collect())
}

pub async fn find_enabled_agents(db: &DbConn) -> Result<Vec<Agent>, DbErr> {
    let agents = Entity::find()
        .filter(Column::Enabled.eq(true))
        .all(db)
        .await?;
    Ok(agents.into_iter().map(|m| m.into()).collect())
}

pub async fn create_agent(db: &DbConn, agent: Agent) -> Result<Agent, DbErr> {
    let active_model: ActiveModel = agent.into();
    let result = active_model.insert(db).await?;
    Ok(result.into())
}

pub async fn update_agent(db: &DbConn, agent: Agent) -> Result<(), DbErr> {
    let active_model: ActiveModel = agent.clone().into();
    active_model.update(db).await?;
    Ok(())
}

pub async fn delete_agent(db: &DbConn, id: Uuid) -> Result<DeleteResult, DbErr> {
    Entity::delete_by_id(id).exec(db).await
}
