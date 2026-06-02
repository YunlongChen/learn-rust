//! Agent 数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Agent 能力定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// DDNS 客户端能力
    DdnsClient,
    /// SSL 证书验证能力
    SslValidator,
    /// Shell 命令执行能力
    ShellExecutor,
    /// 脚本运行能力
    ScriptRunner,
    /// 文件传输能力
    FileTransfer,
    /// 隧道客户端能力
    TunnelClient,
    /// P2P 节点能力
    P2pNode,
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Capability::DdnsClient => write!(f, "DDNS客户端"),
            Capability::SslValidator => write!(f, "SSL验证器"),
            Capability::ShellExecutor => write!(f, "Shell执行"),
            Capability::ScriptRunner => write!(f, "脚本运行"),
            Capability::FileTransfer => write!(f, "文件传输"),
            Capability::TunnelClient => write!(f, "隧道客户端"),
            Capability::P2pNode => write!(f, "P2P节点"),
        }
    }
}

/// Agent 状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    /// 在线
    Online,
    /// 离线
    Offline,
    /// 忙碌（正在执行任务）
    Busy,
    /// 维护中
    Maintenance,
}

impl Default for AgentStatus {
    fn default() -> Self {
        AgentStatus::Offline
    }
}

/// Agent 审批状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentApprovalState {
    /// 等待批准
    Pending,
    /// 已批准
    Approved,
    /// 被拒绝
    Denied,
}

impl Default for AgentApprovalState {
    fn default() -> Self {
        AgentApprovalState::Pending
    }
}

impl std::fmt::Display for AgentApprovalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentApprovalState::Pending => write!(f, "pending"),
            AgentApprovalState::Approved => write!(f, "approved"),
            AgentApprovalState::Denied => write!(f, "denied"),
        }
    }
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentStatus::Online => write!(f, "在线"),
            AgentStatus::Offline => write!(f, "离线"),
            AgentStatus::Busy => write!(f, "忙碌"),
            AgentStatus::Maintenance => write!(f, "维护中"),
        }
    }
}

/// Agent 系统信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemInfo {
    /// CPU 信息
    pub cpu: Option<String>,
    /// 内存信息
    pub memory: Option<String>,
    /// 操作系统
    pub os: Option<String>,
    /// Agent 版本
    pub version: Option<String>,
    /// Rust 运行时版本
    pub rust_version: Option<String>,
}

/// Agent 指标信息（心跳上报）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentMetrics {
    /// CPU 使用率
    pub cpu_usage: Option<f32>,
    /// 内存使用率
    pub memory_usage: Option<f32>,
    /// 磁盘使用率
    pub disk_usage: Option<f32>,
    /// 网络延迟（毫秒）
    pub latency_ms: Option<u32>,
    /// 上行带宽（bps）
    pub upload_speed: Option<u64>,
    /// 下行带宽（bps）
    pub download_speed: Option<u64>,
}

/// Agent 连接信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConnectionInfo {
    /// IP 地址
    pub ip: Option<String>,
    /// 端口
    pub port: Option<u16>,
    /// 连接建立时间
    pub connected_at: Option<DateTime<Utc>>,
    /// 最后活跃时间
    pub last_activity: Option<DateTime<Utc>>,
}

/// Agent 模型（用于数据库）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// 唯一标识
    pub id: Uuid,
    /// 名称
    pub name: String,
    /// 描述
    pub description: Option<String>,
    /// 连接地址（WebSocket 地址）
    pub endpoint: String,
    /// 认证密钥（已废弃，使用 agent_key_hash 代替）
    pub auth_key: Option<String>,
    /// 能力列表
    pub capabilities: Vec<Capability>,
    /// 状态
    pub status: AgentStatus,
    /// 标签
    pub tags: Vec<String>,
    /// 系统信息
    pub system_info: Option<SystemInfo>,
    /// 连接信息
    pub connection_info: Option<ConnectionInfo>,
    /// 最后心跳时间
    pub last_heartbeat: Option<sea_orm::prelude::DateTime>,
    /// 是否启用
    pub enabled: bool,
    /// 审批状态
    pub approval_state: AgentApprovalState,
    /// 密钥哈希（SHA-256）
    pub agent_key_hash: Option<String>,
    /// 批准时间
    pub approved_at: Option<sea_orm::prelude::DateTime>,
    /// 批准人
    pub approved_by: Option<String>,
    /// Agent 版本
    pub version: Option<String>,
    /// 主机名
    pub hostname: Option<String>,
    /// 创建时间
    pub created_at: Option<sea_orm::prelude::DateTime>,
    /// 更新时间
    pub updated_at: Option<sea_orm::prelude::DateTime>,
}

impl Agent {
    /// 创建新的 Agent
    pub fn new(name: String, endpoint: String) -> Self {
        let now = Utc::now().naive_utc();
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            endpoint,
            auth_key: None,
            capabilities: Vec::new(),
            status: AgentStatus::Offline,
            tags: Vec::new(),
            system_info: None,
            connection_info: None,
            last_heartbeat: None,
            enabled: true,
            approval_state: AgentApprovalState::Pending,
            agent_key_hash: None,
            approved_at: None,
            approved_by: None,
            version: None,
            hostname: None,
            created_at: Some(now),
            updated_at: Some(now),
        }
    }

    /// 检查是否支持指定能力
    pub fn has_capability(&self, capability: &Capability) -> bool {
        self.capabilities.contains(capability)
    }

    /// 检查是否在线
    pub fn is_online(&self) -> bool {
        self.status == AgentStatus::Online
    }

    /// 检查是否可用（在线且未忙碌）
    pub fn is_available(&self) -> bool {
        matches!(self.status, AgentStatus::Online | AgentStatus::Busy)
    }

    /// 更新心跳时间
    pub fn update_heartbeat(&mut self) {
        let now = Utc::now().naive_utc();
        self.last_heartbeat = Some(now);
        self.updated_at = Some(now);
    }

    /// 更新状态
    pub fn update_status(&mut self, status: AgentStatus) {
        self.status = status;
        self.updated_at = Some(Utc::now().naive_utc());
    }
}

/// Agent 过滤条件
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentFilter {
    /// 状态过滤
    pub status: Option<AgentStatus>,
    /// 能力过滤
    pub capability: Option<Capability>,
    /// 名称搜索
    pub search: Option<String>,
    /// 标签过滤
    pub tags: Option<Vec<String>>,
}

/// Agent 排序字段
#[derive(Debug, Clone, Default)]
pub enum AgentSortField {
    #[default]
    Name,
    Status,
    LastHeartbeat,
    CreatedAt,
}

impl std::fmt::Display for AgentSortField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentSortField::Name => write!(f, "name"),
            AgentSortField::Status => write!(f, "status"),
            AgentSortField::LastHeartbeat => write!(f, "last_heartbeat"),
            AgentSortField::CreatedAt => write!(f, "created_at"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = Agent::new("Test Agent".to_string(), "ws://localhost:8080".to_string());

        assert!(!agent.id.is_nil());
        assert_eq!(agent.name, "Test Agent");
        assert_eq!(agent.endpoint, "ws://localhost:8080");
        assert_eq!(agent.status, AgentStatus::Offline);
        assert!(agent.is_online() == false);
    }

    #[test]
    fn test_agent_capabilities() {
        let mut agent = Agent::new("Test".to_string(), "ws://localhost".to_string());
        agent.capabilities.push(Capability::ShellExecutor);

        assert!(agent.has_capability(&Capability::ShellExecutor));
        assert!(!agent.has_capability(&Capability::DdnsClient));
    }
}
