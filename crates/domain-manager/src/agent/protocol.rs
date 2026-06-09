//! Agent 通信协议
//! 
//! 定义 Agent 与 Hub 之间的 WebSocket 消息格式

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::model::{AgentMetrics, AgentStatus, Capability};

/// 消息类型标签
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum AgentMessage {
    // ==================== 注册相关 ====================

    /// Agent 使用密钥注册（新版，推荐）
    #[serde(rename = "RegisterWithSecret")]
    RegisterWithSecret {
        #[serde(skip_serializing_if = "Option::is_none")]
        agent_id: Option<String>,
        agent_name: String,
        agent_key: String,
        capabilities: Vec<Capability>,
        version: Option<String>,
        hostname: Option<String>,
    },

    /// Hub 接受注册请求
    #[serde(rename = "RegisterAccepted")]
    RegisterAccepted {
        agent_id: Uuid,
        session_id: String,
        server_time: i64,
        requires_approval: bool,
        message: Option<String>,
    },

    /// Hub 拒绝注册
    #[serde(rename = "RegisterRejected")]
    RegisterRejected {
        reason: String,
        code: String,
    },

    /// Hub 批准 Agent（需要审批时）
    #[serde(rename = "ApprovalGranted")]
    ApprovalGranted {
        server_time: i64,
        message: Option<String>,
    },

    /// Hub 拒绝批准 Agent
    #[serde(rename = "ApprovalDenied")]
    ApprovalDenied {
        reason: String,
    },

    /// Agent 主动断开连接
    #[serde(rename = "Unregister")]
    Unregister {
        reason: Option<String>,
    },

    // ==================== 旧版兼容（已废弃）====================

    /// Agent 注册请求（旧版，仅用于兼容）
    #[serde(rename = "Register")]
    Register {
        agent_id: Uuid,
        name: String,
        endpoint: String,
        capabilities: Vec<Capability>,
        version: Option<String>,
    },

    /// 注册确认响应（旧版，仅用于兼容）
    #[serde(rename = "RegisterAck")]
    RegisterAck {
        session_id: String,
        server_time: i64,
        message: Option<String>,
    },

    // ==================== 心跳相关 ====================
    
    // ==================== 心跳相关 ====================
    
    /// 心跳请求（Agent -> Hub）
    Heartbeat {
        status: AgentStatus,
        metrics: AgentMetrics,
        timestamp: i64,
    },
    
    /// 心跳响应（Hub -> Agent）
    HeartbeatAck {
        server_time: i64,
    },
    
    // ==================== 任务相关 ====================
    
    /// 任务分配（Hub -> Agent）
    TaskAssigned {
        task_id: Uuid,
        task_type: TaskType,
        params: serde_json::Value,
        timeout_seconds: u32,
    },
    
    /// 任务结果（Agent -> Hub）
    TaskResult {
        task_id: Uuid,
        success: bool,
        output: String,
        error: Option<String>,
        exit_code: Option<i32>,
        duration_ms: u64,
    },
    
    /// 任务取消（Hub -> Agent）
    TaskCancelled {
        task_id: Uuid,
        reason: String,
    },
    
    // ==================== DDNS 相关 ====================
    
    /// DDNS 更新请求（Hub -> Agent）
    DdnsUpdateRequest {
        domain: String,
        record_type: String,
    },
    
    /// DDNS 更新结果（Agent -> Hub）
    DdnsUpdateResult {
        domain: String,
        success: bool,
        old_ip: Option<String>,
        new_ip: Option<String>,
        error: Option<String>,
    },
    
    // ==================== SSL 相关 ====================
    
    /// SSL 挑战请求（Hub -> Agent）
    SslChallengeRequest {
        domain: String,
        challenge_type: SslChallengeType,
        token: String,
    },
    
    /// SSL 挑战响应（Agent -> Hub）
    SslChallengeResponse {
        domain: String,
        success: bool,
        key_authorization: Option<String>,
        error: Option<String>,
    },
    
    // ==================== 系统相关 ====================
    
    /// 错误消息
    Error {
        code: String,
        message: String,
        details: Option<serde_json::Value>,
    },
    
    /// Ping 消息
    Ping {
        timestamp: i64,
    },
    
    /// Pong 响应
    Pong {
        timestamp: i64,
    },

    // ==================== 隧道相关 ====================

    /// Agent 请求反向隧道 (Agent -> Hub)
    TunnelRequest {
        tunnel_id: Uuid,
        bind_port: u16,
        tunnel_type: TunnelType,
    },

    /// Hub 响应隧道请求 (Hub -> Agent)
    TunnelResponse {
        tunnel_id: Uuid,
        success: bool,
        public_port: Option<u16>,
        error: Option<String>,
    },

    /// Hub 通过隧道转发数据 (Hub -> Agent)
    TunnelData {
        tunnel_id: Uuid,
        data: Vec<u8>,
    },

    /// Agent 透传数据 (Agent -> Hub)
    TunnelDataForward {
        tunnel_id: Uuid,
        data: Vec<u8>,
    },

    /// 关闭隧道
    TunnelClose {
        tunnel_id: Uuid,
        reason: Option<String>,
    },

    // ==================== P2P/ICE 相关 ====================

    /// Agent 请求与另一个 Agent 建立 P2P 连接
    P2pConnectRequest {
        request_id: Uuid,
        target_agent_id: Uuid,
    },

    /// Hub 转发 P2P 连接请求给目标 Agent
    P2pConnectOffer {
        request_id: Uuid,
        source_agent_id: Uuid,
        sdp_offer: String,
    },

    /// 目标 Agent 返回 SDP Answer
    P2pAnswer {
        request_id: Uuid,
        target_agent_id: Uuid,
        sdp_answer: String,
    },

    /// ICE 候选地址交换
    P2pIceCandidate {
        request_id: Uuid,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<u16>,
    },

    /// P2P 连接已建立
    P2pConnected {
        request_id: Uuid,
    },

    /// P2P 连接失败
    P2pFailed {
        request_id: Uuid,
        reason: String,
    },
}

/// 隧道类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TunnelType {
    /// TCP 直连
    Tcp,
    /// HTTP 代理
    Http,
    /// WebSocket
    WebSocket,
}

/// 任务类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum TaskType {
    /// Shell 命令执行
    Shell {
        command: String,
        working_dir: Option<String>,
        env_vars: Option<std::collections::HashMap<String, String>>,
    },
    
    /// 脚本执行
    Script {
        script_id: Uuid,
        args: Vec<String>,
    },
    
    /// 文件上传
    FileUpload {
        remote_path: String,
        content_base64: String,
        mode: Option<u32>,
    },
    
    /// 文件下载
    FileDownload {
        remote_path: String,
    },
    
    /// 文件删除
    FileDelete {
        remote_path: String,
    },
    
    /// 获取公网 IP
    GetPublicIp,
    
    /// SSL HTTP-01 挑战
    SslHttpChallenge {
        token: String,
        key_authorization: String,
        web_root: String,
    },
}

/// SSL 挑战类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SslChallengeType {
    /// HTTP-01 挑战
    Http01,
    /// DNS-01 挑战
    Dns01,
}

/// 会话信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// 会话 ID
    pub id: String,
    /// Agent ID
    pub agent_id: Uuid,
    /// 创建时间戳
    pub created_at: i64,
    /// 最后活跃时间戳
    pub last_activity: i64,
    /// 连接是否有效
    pub active: bool,
}

impl Session {
    /// 创建新会话
    pub fn new(agent_id: Uuid) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            agent_id,
            created_at: now,
            last_activity: now,
            active: true,
        }
    }
    
    /// 更新活跃时间
    pub fn touch(&mut self) {
        self.last_activity = chrono::Utc::now().timestamp();
    }
    
    /// 标记为非活跃
    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_message_serialization() {
        let msg = AgentMessage::Register {
            agent_id: Uuid::new_v4(),
            name: "Test Agent".to_string(),
            endpoint: "ws://localhost:8080".to_string(),
            capabilities: vec![Capability::ShellExecutor],
            version: Some("0.1.0".to_string()),
        };
        
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"Register\""));
        assert!(json.contains("\"name\":\"Test Agent\""));
    }

    #[test]
    fn test_task_type_serialization() {
        let task = TaskType::Shell {
            command: "ls -la".to_string(),
            working_dir: Some("/tmp".to_string()),
            env_vars: None,
        };
        
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("\"type\":\"Shell\""));
        assert!(json.contains("\"command\":\"ls -la\""));
    }
}
