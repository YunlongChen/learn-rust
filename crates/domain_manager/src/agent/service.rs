//! Agent 服务层
//! 
//! 提供 Agent 的业务逻辑服务

use std::sync::Arc;
use uuid::Uuid;

use super::model::{Agent, AgentFilter, AgentStatus, Capability};
use super::registry::AgentRegistry;

/// Agent 服务
pub struct AgentService {
    registry: Arc<AgentRegistry>,
}

impl AgentService {
    /// 创建 Agent 服务
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self { registry }
    }
    
    /// 创建 Agent
    pub async fn create_agent(&self, name: String, endpoint: String) -> Agent {
        let agent = Agent::new(name, endpoint);
        let mut agent = agent;
        agent.enabled = true;
        
        self.registry.register(agent.clone()).await;
        agent
    }
    
    /// 获取 Agent
    pub async fn get_agent(&self, agent_id: Uuid) -> Option<Agent> {
        self.registry.get(agent_id).await
    }
    
    /// 获取所有 Agent
    pub async fn get_all_agents(&self) -> Vec<Agent> {
        self.registry.get_all().await
    }
    
    /// 获取在线 Agent
    pub async fn get_online_agents(&self) -> Vec<Agent> {
        self.registry.get_online().await
    }
    
    /// 获取可用 Agent
    pub async fn get_available_agents(&self) -> Vec<Agent> {
        self.registry.get_available().await
    }
    
    /// 根据能力获取 Agent
    pub async fn get_agents_by_capability(&self, capability: Capability) -> Vec<Agent> {
        self.registry.get_by_capability(&capability).await
    }
    
    /// 筛选 Agent
    pub async fn filter_agents(&self, filter: AgentFilter) -> Vec<Agent> {
        self.registry.filter(&filter).await
    }
    
    /// 更新 Agent 状态
    pub async fn update_status(&self, agent_id: Uuid, status: AgentStatus) -> Result<(), String> {
        if self.registry.update_status(agent_id, status).await {
            Ok(())
        } else {
            Err(format!("Agent {} not found", agent_id))
        }
    }
    
    /// Agent 上线
    pub async fn agent_online(&self, agent_id: Uuid) -> Result<(), String> {
        self.update_status(agent_id, AgentStatus::Online).await
    }
    
    /// Agent 离线
    pub async fn agent_offline(&self, agent_id: Uuid) -> Result<(), String> {
        self.update_status(agent_id, AgentStatus::Offline).await
    }
    
    /// Agent 忙碌
    pub async fn agent_busy(&self, agent_id: Uuid) -> Result<(), String> {
        self.update_status(agent_id, AgentStatus::Busy).await
    }
    
    /// 更新心跳
    pub async fn update_heartbeat(&self, agent_id: Uuid) -> Result<(), String> {
        if self.registry.update_heartbeat(agent_id).await {
            Ok(())
        } else {
            Err(format!("Agent {} not found", agent_id))
        }
    }
    
    /// 删除 Agent
    pub async fn delete_agent(&self, agent_id: Uuid) -> Result<Agent, String> {
        self.registry
            .unregister(agent_id)
            .await
            .ok_or_else(|| format!("Agent {} not found", agent_id))
    }
    
    /// 获取 Agent 统计
    pub async fn get_statistics(&self) -> AgentStatistics {
        let agents = self.get_all_agents().await;
        let online = agents.iter().filter(|a| a.status == AgentStatus::Online).count();
        let offline = agents.iter().filter(|a| a.status == AgentStatus::Offline).count();
        let busy = agents.iter().filter(|a| a.status == AgentStatus::Busy).count();
        
        AgentStatistics {
            total: agents.len(),
            online,
            offline,
            busy,
            total_capabilities: Capability::all().len(),
        }
    }
    
    /// 检查 Agent 是否在线
    pub async fn is_agent_online(&self, agent_id: Uuid) -> bool {
        if let Some(agent) = self.registry.get(agent_id).await {
            agent.is_online()
        } else {
            false
        }
    }
    
    /// 检查 Agent 是否有能力
    pub async fn has_capability(&self, agent_id: Uuid, capability: &Capability) -> bool {
        if let Some(agent) = self.registry.get(agent_id).await {
            agent.has_capability(capability)
        } else {
            false
        }
    }
}

/// Agent 统计信息
#[derive(Debug, Clone)]
pub struct AgentStatistics {
    /// 总数
    pub total: usize,
    /// 在线数
    pub online: usize,
    /// 离线数
    pub offline: usize,
    /// 忙碌数
    pub busy: usize,
    /// 能力总数
    pub total_capabilities: usize,
}

impl Capability {
    /// 获取所有能力列表
    pub fn all() -> Vec<Capability> {
        vec![
            Capability::DdnsClient,
            Capability::SslValidator,
            Capability::ShellExecutor,
            Capability::ScriptRunner,
            Capability::FileTransfer,
        ]
    }
    
    /// 从字符串解析能力
    pub fn from_str(s: &str) -> Option<Capability> {
        match s.to_lowercase().as_str() {
            "ddns_client" | "ddns" => Some(Capability::DdnsClient),
            "ssl_validator" | "ssl" | "ssl_validator" => Some(Capability::SslValidator),
            "shell_executor" | "shell" => Some(Capability::ShellExecutor),
            "script_runner" | "script" => Some(Capability::ScriptRunner),
            "file_transfer" | "file" => Some(Capability::FileTransfer),
            _ => None,
        }
    }
}

impl std::fmt::Display for AgentStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Agent Statistics:")?;
        writeln!(f, "  Total: {}", self.total)?;
        writeln!(f, "  Online: {}", self.online)?;
        writeln!(f, "  Offline: {}", self.offline)?;
        writeln!(f, "  Busy: {}", self.busy)?;
        writeln!(f, "  Capabilities: {}", self.total_capabilities)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_agent() {
        let registry = Arc::new(AgentRegistry::new());
        let service = AgentService::new(registry);
        
        let agent = service.create_agent("Test".to_string(), "ws://localhost".to_string()).await;
        assert_eq!(agent.name, "Test");
        assert_eq!(agent.endpoint, "ws://localhost");
        assert!(agent.enabled);
    }

    #[tokio::test]
    async fn test_get_online_agents() {
        let registry = Arc::new(AgentRegistry::new());
        let service = AgentService::new(registry);
        
        let agent1 = service.create_agent("Agent 1".to_string(), "ws://1".to_string()).await;
        service.create_agent("Agent 2".to_string(), "ws://2".to_string()).await;
        
        service.agent_online(agent1.id).await.unwrap();
        
        let online = service.get_online_agents().await;
        assert_eq!(online.len(), 1);
        assert_eq!(online[0].name, "Agent 1");
    }
}
