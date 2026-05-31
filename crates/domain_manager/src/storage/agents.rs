//! Agent 数据访问层
//!
//! 提供 Agent 的数据库操作

use crate::storage::account::Relation;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue, DeleteResult};
use serde::{Deserialize, Serialize};

use crate::agent::model::{Agent, AgentApprovalState, AgentStatus};
use crate::storage::entities::agents::{ActiveModel, Column, Entity, Model};

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
            version: None,
            hostname: None,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::model::Capability;
    use crate::storage::database::init_memory_database;
    use crate::tests::test_utils::init_test_env;

    fn create_test_agent() -> Agent {
        Agent::new("Test Agent".to_string(), "ws://localhost:8080".to_string())
    }

    #[tokio::test]
    async fn it_works() {
        init_test_env();
        let connection = init_memory_database().await.expect("初始化数据库失败");

        let agent = create_test_agent();
        let created = create_agent(&connection, agent.clone())
            .await
            .expect("创建 Agent 失败");

        assert_eq!(created.name, "Test Agent");
        assert_eq!(created.endpoint, "ws://localhost:8080");
        assert_eq!(created.status, AgentStatus::Offline);
        assert_eq!(created.approval_state, AgentApprovalState::Pending);

        let all_agents = find_all_agents(&connection).await.expect("查询所有 Agent 失败");
        assert_eq!(all_agents.len(), 1);

        let found = find_agent_by_id(&connection, created.id)
            .await
            .expect("根据 ID 查询 Agent 失败");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test Agent");
    }

    #[tokio::test]
    async fn test_find_online_agents() {
        init_test_env();
        let connection = init_memory_database().await.expect("初始化数据库失败");

        let agent = create_test_agent();
        create_agent(&connection, agent).await.expect("创建 Agent 失败");

        let online_agents = find_online_agents(&connection)
            .await
            .expect("查询在线 Agent 失败");
        assert_eq!(online_agents.len(), 0);
    }

    #[tokio::test]
    async fn test_find_enabled_agents() {
        init_test_env();
        let connection = init_memory_database().await.expect("初始化数据库失败");

        let agent = create_test_agent();
        let created = create_agent(&connection, agent).await.expect("创建 Agent 失败");

        let enabled_agents = find_enabled_agents(&connection)
            .await
            .expect("查询启用 Agent 失败");
        assert_eq!(enabled_agents.len(), 1);
        assert_eq!(enabled_agents[0].id, created.id);
    }

    #[tokio::test]
    async fn test_update_agent() {
        init_test_env();
        let connection = init_memory_database().await.expect("初始化数据库失败");

        let mut agent = create_test_agent();
        let created = create_agent(&connection, agent.clone())
            .await
            .expect("创建 Agent 失败");

        let mut updated_agent = created.clone();
        updated_agent.name = "Updated Agent".to_string();
        updated_agent.status = AgentStatus::Online;

        update_agent(&connection, updated_agent)
            .await
            .expect("更新 Agent 失败");

        let found = find_agent_by_id(&connection, created.id)
            .await
            .expect("根据 ID 查询 Agent 失败");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Updated Agent");
    }

    #[tokio::test]
    async fn test_delete_agent() {
        init_test_env();
        let connection = init_memory_database().await.expect("初始化数据库失败");

        let agent = create_test_agent();
        let created = create_agent(&connection, agent).await.expect("创建 Agent 失败");

        delete_agent(&connection, created.id)
            .await
            .expect("删除 Agent 失败");

        let all_agents = find_all_agents(&connection).await.expect("查询所有 Agent 失败");
        assert_eq!(all_agents.len(), 0);
    }

    #[tokio::test]
    async fn test_agent_with_capabilities() {
        init_test_env();
        let connection = init_memory_database().await.expect("初始化数据库失败");

        let mut agent = create_test_agent();
        agent.capabilities.push(Capability::ShellExecutor);
        agent.capabilities.push(Capability::DdnsClient);

        let created = create_agent(&connection, agent).await.expect("创建 Agent 失败");

        assert!(created.has_capability(&Capability::ShellExecutor));
        assert!(created.has_capability(&Capability::DdnsClient));
        assert!(!created.has_capability(&Capability::SslValidator));
    }

    #[tokio::test]
    async fn test_agent_is_available() {
        let mut agent = create_test_agent();
        assert!(!agent.is_available());

        agent.status = AgentStatus::Online;
        assert!(agent.is_available());

        agent.status = AgentStatus::Busy;
        assert!(agent.is_available());

        agent.status = AgentStatus::Offline;
        assert!(!agent.is_available());
    }
}
