//! Agent 注册表
//!
//! 管理所有已连接的 Agent，包括在线状态、心跳等

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::model::{Agent, AgentFilter, AgentStatus, Capability};
use super::protocol::Session;

/// Agent 注册表
pub struct AgentRegistry {
    /// Agent 列表
    agents: RwLock<HashMap<Uuid, Agent>>,
    /// 在线会话
    sessions: RwLock<HashMap<Uuid, Session>>,
    /// WebSocket 连接（可选）
    connections: RwLock<HashMap<Uuid, Arc<dyn AgentConnection>>>,
}

impl AgentRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        Self {
            agents: RwLock::new(HashMap::new()),
            sessions: RwLock::new(HashMap::new()),
            connections: RwLock::new(HashMap::new()),
        }
    }

    /// 注册 Agent
    pub async fn register(&self, mut agent: Agent) -> Option<Session> {
        let mut agents = self.agents.write().await;
        let mut sessions = self.sessions.write().await;

        // 检查是否已存在
        if agents.contains_key(&agent.id) {
            return None;
        }

        // 创建会话
        let session = Session::new(agent.id);
        let session_clone = session.clone();

        // 插入 Agent
        agent.update_status(AgentStatus::Online);
        agents.insert(agent.id, agent);

        // 插入会话
        sessions.insert(session_clone.agent_id, session_clone);

        Some(session)
    }

    /// 注销 Agent
    pub async fn unregister(&self, agent_id: Uuid) -> Option<Agent> {
        let mut agents = self.agents.write().await;
        let mut sessions = self.sessions.write().await;
        let mut connections = self.connections.write().await;

        connections.remove(&agent_id);
        sessions.remove(&agent_id);
        agents.remove(&agent_id)
    }

    /// 获取 Agent
    pub async fn get(&self, agent_id: Uuid) -> Option<Agent> {
        let agents = self.agents.read().await;
        agents.get(&agent_id).cloned()
    }

    /// 获取所有 Agent
    pub async fn get_all(&self) -> Vec<Agent> {
        let agents = self.agents.read().await;
        agents.values().cloned().collect()
    }

    /// 获取在线 Agent
    pub async fn get_online(&self) -> Vec<Agent> {
        let agents = self.agents.read().await;
        agents
            .values()
            .filter(|a| a.status == AgentStatus::Online)
            .cloned()
            .collect()
    }

    /// 获取可用 Agent（在线且未忙碌）
    pub async fn get_available(&self) -> Vec<Agent> {
        let agents = self.agents.read().await;
        agents
            .values()
            .filter(|a| matches!(a.status, AgentStatus::Online))
            .cloned()
            .collect()
    }

    /// 根据能力筛选 Agent
    pub async fn get_by_capability(&self, capability: &Capability) -> Vec<Agent> {
        let agents = self.agents.read().await;
        agents
            .values()
            .filter(|a| a.has_capability(capability))
            .cloned()
            .collect()
    }

    /// 筛选 Agent
    pub async fn filter(&self, filter: &AgentFilter) -> Vec<Agent> {
        let agents = self.agents.read().await;

        agents
            .values()
            .filter(|agent| {
                // 状态过滤
                if let Some(status) = &filter.status {
                    if agent.status != *status {
                        return false;
                    }
                }

                // 能力过滤
                if let Some(capability) = &filter.capability {
                    if !agent.has_capability(capability) {
                        return false;
                    }
                }

                // 名称搜索
                if let Some(search) = &filter.search {
                    if !agent.name.to_lowercase().contains(&search.to_lowercase()) {
                        return false;
                    }
                }

                // 标签过滤
                if let Some(tags) = &filter.tags {
                    if !tags.iter().any(|tag| agent.tags.contains(tag)) {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect()
    }

    /// 更新 Agent 状态
    pub async fn update_status(&self, agent_id: Uuid, status: AgentStatus) -> bool {
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(&agent_id) {
            agent.update_status(status);
            true
        } else {
            false
        }
    }

    /// 更新心跳
    pub async fn update_heartbeat(&self, agent_id: Uuid) -> bool {
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(&agent_id) {
            agent.update_heartbeat();
            true
        } else {
            false
        }
    }

    /// 获取会话
    pub async fn get_session(&self, agent_id: Uuid) -> Option<Session> {
        let sessions = self.sessions.read().await;
        sessions.get(&agent_id).cloned()
    }

    /// 更新会话活跃时间
    pub async fn touch_session(&self, agent_id: Uuid) -> bool {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&agent_id) {
            session.touch();
            true
        } else {
            false
        }
    }

    /// 设置连接
    pub async fn set_connection(&self, agent_id: Uuid, connection: Arc<dyn AgentConnection>) {
        let mut connections = self.connections.write().await;
        connections.insert(agent_id, connection);
    }

    /// 获取连接
    pub async fn get_connection(&self, agent_id: Uuid) -> Option<Arc<dyn AgentConnection>> {
        let connections = self.connections.read().await;
        connections.get(&agent_id).cloned()
    }

    /// 获取连接数量
    pub async fn connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }

    /// 获取在线 Agent 数量
    pub async fn online_count(&self) -> usize {
        let agents = self.agents.read().await;
        agents
            .values()
            .filter(|a| a.status == AgentStatus::Online)
            .count()
    }
}

/// Agent 连接接口
pub trait AgentConnection: Send + Sync + 'static {
    /// 发送消息
    fn send(&self, message: &str) -> Box<dyn std::future::Future<Output = Result<(), String>> + Send>;

    /// 关闭连接
    fn close(&self) -> Box<dyn std::future::Future<Output = Result<(), String>> + Send>;

    /// 检查是否已关闭
    fn is_closed(&self) -> bool;
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_agent() {
        let registry = AgentRegistry::new();
        let agent = Agent::new("Test Agent".to_string(), "ws://localhost:8080".to_string());

        let result = registry.register(agent.clone()).await;
        assert!(result.is_some());

        let retrieved = registry.get(agent.id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Agent");
    }

    #[tokio::test]
    async fn test_get_online_agents() {
        let registry = AgentRegistry::new();

        let mut agent1 = Agent::new("Agent 1".to_string(), "ws://localhost:8081".to_string());
        agent1.update_status(AgentStatus::Online);

        let agent2 = Agent::new("Agent 2".to_string(), "ws://localhost:8082".to_string());

        registry.register(agent1).await;
        registry.register(agent2).await;

        let online = registry.get_online().await;
        assert_eq!(online.len(), 1);
        assert_eq!(online[0].name, "Agent 1");
    }

    #[tokio::test]
    async fn test_filter_by_capability() {
        let registry = AgentRegistry::new();

        let mut agent1 = Agent::new("Agent 1".to_string(), "ws://localhost:8081".to_string());
        agent1.capabilities.push(Capability::ShellExecutor);

        let mut agent2 = Agent::new("Agent 2".to_string(), "ws://localhost:8082".to_string());
        agent2.capabilities.push(Capability::DdnsClient);

        registry.register(agent1).await;
        registry.register(agent2).await;

        let shell_agents = registry.get_by_capability(&Capability::ShellExecutor).await;
        assert_eq!(shell_agents.len(), 1);
        assert_eq!(shell_agents[0].name, "Agent 1");
    }
}
