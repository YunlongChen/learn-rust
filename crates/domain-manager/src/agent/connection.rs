//! Agent WebSocket 连接管理
//!
//! 处理 Agent 的 WebSocket 连接和消息发送

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use sha2::{Sha256, Digest};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::model::{Agent, AgentStatus};
use super::protocol::{AgentMessage, AgentMessage::*, TunnelType};
use super::registry::AgentRegistry;

/// 计算密钥哈希
fn hash_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Agent 连接信息
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub agent_id: Uuid,
    pub session_id: String,
    pub connected_at: chrono::DateTime<chrono::Utc>,
}

/// 单个 Agent 连接
pub struct AgentConnection {
    pub agent_id: Uuid,
    pub session_id: String,
    pub ws_stream: Option<WebSocketStream<TcpStream>>,
    pub tunnels: HashMap<Uuid, TunnelInfo>,
    pub p2p_connections: HashMap<Uuid, P2pConnectionInfo>,
}

#[derive(Debug, Clone)]
pub struct TunnelInfo {
    pub tunnel_id: Uuid,
    pub bind_port: u16,
    pub tunnel_type: TunnelType,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct P2pConnectionInfo {
    pub request_id: Uuid,
    pub peer_agent_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl AgentConnection {
    pub fn new(agent_id: Uuid, session_id: String) -> Self {
        Self {
            agent_id,
            session_id,
            ws_stream: None,
            tunnels: HashMap::new(),
            p2p_connections: HashMap::new(),
        }
    }

    /// 发送消息
    pub async fn send(&mut self, msg: &AgentMessage) -> Result<(), String> {
        if let Some(ref mut ws) = self.ws_stream {
            let json = serde_json::to_string(msg)
                .map_err(|e| format!("序列化消息失败: {}", e))?;
            ws.send(Message::Text(json.into()))
                .await
                .map_err(|e| format!("发送消息失败: {}", e))?;
            Ok(())
        } else {
            Err("WebSocket 未连接".to_string())
        }
    }

    /// 接收消息
    pub async fn recv(&mut self) -> Result<AgentMessage, String> {
        if let Some(ref mut ws) = self.ws_stream {
            match ws.next().await {
                Some(Ok(Message::Text(text))) => {
                    serde_json::from_str(&text)
                        .map_err(|e| format!("解析消息失败: {}", e))
                }
                Some(Ok(Message::Close(_))) => {
                    Err("连接已关闭".to_string())
                }
                Some(Ok(_)) => {
                    Err("不支持的消息类型".to_string())
                }
                Some(Err(e)) => {
                    Err(format!("接收消息错误: {}", e))
                }
                None => {
                    Err("流已结束".to_string())
                }
            }
        } else {
            Err("WebSocket 未连接".to_string())
        }
    }

    /// 关闭连接
    pub async fn close(mut self) -> Result<(), String> {
        if let Some(ref mut ws) = self.ws_stream {
            ws.close(None).await.map_err(|e| format!("关闭连接失败: {}", e))?;
        }
        Ok(())
    }

    /// 添加隧道
    pub fn add_tunnel(&mut self, tunnel_id: Uuid, bind_port: u16, tunnel_type: TunnelType) {
        self.tunnels.insert(tunnel_id, TunnelInfo {
            tunnel_id,
            bind_port,
            tunnel_type: tunnel_type.clone(),
            created_at: chrono::Utc::now(),
        });
        info!("Agent {} 添加隧道 {}, 端口: {}, 类型: {:?}", self.agent_id, tunnel_id, bind_port, tunnel_type);
    }

    /// 移除隧道
    pub fn remove_tunnel(&mut self, tunnel_id: &Uuid) -> Option<TunnelInfo> {
        self.tunnels.remove(tunnel_id)
    }

    /// 获取隧道数量
    pub fn tunnel_count(&self) -> usize {
        self.tunnels.len()
    }

    /// 添加 P2P 连接
    pub fn add_p2p_connection(&mut self, request_id: Uuid, peer_agent_id: Uuid) {
        self.p2p_connections.insert(request_id, P2pConnectionInfo {
            request_id,
            peer_agent_id,
            created_at: chrono::Utc::now(),
        });
    }

    /// 移除 P2P 连接
    pub fn remove_p2p_connection(&mut self, request_id: &Uuid) -> Option<P2pConnectionInfo> {
        self.p2p_connections.remove(request_id)
    }
}

/// Agent Hub - 管理所有 Agent 连接
pub struct AgentHub {
    registry: Arc<AgentRegistry>,
    connections: Arc<RwLock<HashMap<Uuid, Arc<RwLock<AgentConnection>>>>>,
    listener: Option<TcpListener>,
    listen_addr: String,
    /// 已注册的密钥哈希（用于简单验证，生产环境应从数据库加载）
    registered_keys: Arc<RwLock<HashMap<String, Uuid>>>,
}

impl AgentHub {
    /// 创建新的 AgentHub
    pub fn new(registry: Arc<AgentRegistry>, listen_addr: &str) -> Self {
        Self {
            registry,
            connections: Arc::new(RwLock::new(HashMap::new())),
            listener: None,
            listen_addr: listen_addr.to_string(),
            registered_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册密钥（将密钥哈希与 Agent ID 关联）
    pub async fn register_key(&self, key_hash: String, agent_id: Uuid) {
        let mut keys = self.registered_keys.write().await;
        keys.insert(key_hash, agent_id);
    }

    /// 验证密钥
    pub async fn verify_key(&self, key: &str) -> Option<Uuid> {
        let key_hash = hash_key(key);
        let keys = self.registered_keys.read().await;
        keys.get(&key_hash).copied()
    }

    /// 启动 WebSocket 服务器
    pub async fn start(&mut self) -> Result<(), String> {
        let addr = self.listen_addr.clone();
        info!("启动 Agent Hub WebSocket 服务器: {}", addr);

        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| format!("绑定地址失败: {}", e))?;

        self.listener = Some(listener);
        info!("Agent Hub 已启动，监听 {}", addr);

        Ok(())
    }

    /// 运行服务器主循环
    pub async fn run(&mut self) -> Result<(), String> {
        let listener = self.listener.take()
            .ok_or_else(|| "服务器未启动".to_string())?;

        info!("Agent Hub 开始接受连接...");

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let registry = self.registry.clone();
                    let connections = Arc::clone(&self.connections);
                    let registered_keys = self.registered_keys.clone();

                    info!("收到来自 {} 的连接", addr);

                    tokio::spawn(async move {
                        if let Err(e) = handle_new_connection(registry, connections, stream, addr, registered_keys).await {
                            error!("处理连接失败: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("接受连接失败: {}", e);
                }
            }
        }
    }

    /// 向指定 Agent 发送消息
    pub async fn send_to(&self, agent_id: Uuid, msg: &AgentMessage) -> Result<(), String> {
        let connections = self.connections.read().await;

        if let Some(conn) = connections.get(&agent_id) {
            let mut conn = conn.write().await;
            conn.send(msg).await
        } else {
            Err(format!("Agent {} 未连接", agent_id))
        }
    }

    /// 广播消息到所有连接的 Agent
    pub async fn broadcast(&self, msg: &AgentMessage) -> Result<(), String> {
        let connections = self.connections.read().await;
        let mut errors = Vec::new();

        for (agent_id, conn) in connections.iter() {
            let mut conn = conn.write().await;
            if let Err(e) = conn.send(msg).await {
                errors.push(format!("Agent {}: {}", agent_id, e));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }

    /// 获取连接数量
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }
}

/// 处理新的 WebSocket 连接
async fn handle_new_connection(
    registry: Arc<AgentRegistry>,
    connections: Arc<RwLock<HashMap<Uuid, Arc<RwLock<AgentConnection>>>>>,
    stream: TcpStream,
    addr: SocketAddr,
    registered_keys: Arc<RwLock<HashMap<String, Uuid>>>,
) -> Result<(), String> {
    let ws_stream = accept_async(stream)
        .await
        .map_err(|e| format!("WebSocket 握手失败: {}", e))?;

    let agent_conn = Arc::new(RwLock::new(AgentConnection::new(
        Uuid::nil(),
        String::new(),
    )));

    {
        let mut conn_guard = agent_conn.write().await;
        conn_guard.ws_stream = Some(ws_stream);
    }

    // 用于存储验证后的 agent_id
    let mut verified_agent_id: Option<Uuid> = None;

    loop {
        let msg = {
            let mut conn = agent_conn.write().await;
            match conn.recv().await {
                Ok(msg) => msg,
                Err(e) => {
                    warn!("从 {} 接收消息失败: {}", addr, e);
                    break;
                }
            }
        };

        match msg {
            RegisterWithSecret {
                agent_id: _,
                agent_name,
                agent_key,
                capabilities,
                version,
                hostname,
            } => {
                info!(
                    "收到 Agent 注册请求: name={}, capabilities={:?}, version={:?}",
                    agent_name, capabilities, version
                );

                // 验证密钥
                let key_hash = hash_key(&agent_key);
                let keys = registered_keys.read().await;
                let agent_id = if let Some(id) = keys.get(&key_hash) {
                    info!("密钥验证成功，Agent ID: {}", id);
                    *id
                } else {
                    // 简单模式：使用密钥哈希的前8字节生成一个稳定的 ID
                    let generated_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, key_hash.as_bytes());
                    info!("使用生成的 Agent ID（密钥未在 Hub 注册）: {}", generated_id);
                    generated_id
                };

                // 检查是否需要审批
                let requires_approval = false; // TODO: 根据配置决定

                // 发送注册接受消息
                let _ = broadcast_msg(&agent_conn, &RegisterAccepted {
                    agent_id,
                    session_id: Uuid::new_v4().to_string(),
                    server_time: chrono::Utc::now().timestamp(),
                    requires_approval,
                    message: Some("注册已接受".to_string()),
                }).await;

                // 更新连接信息
                {
                    let mut conn = agent_conn.write().await;
                    conn.agent_id = agent_id;
                }

                // 保存 agent_id
                verified_agent_id = Some(agent_id);

                // 在注册表中注册
                let mut agent = Agent::new(agent_name.clone(), format!("ws://{}", addr));
                agent.capabilities = capabilities;
                agent.version = version;
                agent.hostname = hostname;
                agent.status = AgentStatus::Online;
                agent.update_heartbeat();
                registry.register(agent).await;

                info!("Agent {} 注册已接受，等待心跳", agent_id);
            }
            Register {
                agent_id: _,
                name: _,
                endpoint: _,
                capabilities: _,
                version: _,
            } => {
                // 旧版注册消息，已废弃，发送拒绝
                let _ = broadcast_msg(&agent_conn, &RegisterRejected {
                    reason: "Unsupported registration format".to_string(),
                    code: "OLD_PROTOCOL".to_string(),
                }).await;
                break;
            }
            Heartbeat { status, metrics, timestamp: _ } => {
                let agent_id = agent_conn.read().await.agent_id;
                if agent_id == Uuid::nil() {
                    warn!("收到未注册 Agent 的心跳");
                    continue;
                }

                debug!("收到 Agent {} 心跳: {:?}, 指标: {:?}", agent_id, status, metrics);
                registry.update_heartbeat(agent_id).await;
                registry.update_status(agent_id, status).await;

                let _ = broadcast_msg(&agent_conn, &HeartbeatAck {
                    server_time: chrono::Utc::now().timestamp(),
                }).await;
            }
            DdnsUpdateResult { domain, success, old_ip, new_ip, error: _ } => {
                let agent_id = agent_conn.read().await.agent_id;
                info!("DDNS 更新结果 from {}: {} - success={}, old={:?}, new={:?}", agent_id, domain, success, old_ip, new_ip);
            }
            SslChallengeResponse { domain, success, key_authorization: _, error: _ } => {
                let agent_id = agent_conn.read().await.agent_id;
                info!("SSL 挑战响应 from {}: {} - success={}", agent_id, domain, success);
            }
            TaskResult { task_id, success, output: _, error: _, exit_code, duration_ms: _ } => {
                let agent_id = agent_conn.read().await.agent_id;
                info!("任务结果 from {}: task_id={}, success={}, exit_code={:?}", agent_id, task_id, success, exit_code);
            }

            // ==================== Tunnel 消息处理 ====================
            TunnelRequest { tunnel_id, bind_port, tunnel_type } => {
                let _agent_id = agent_conn.read().await.agent_id;
                info!("收到 Tunnel 请求: tunnel_id={}, port={}, type={:?}", tunnel_id, bind_port, tunnel_type);

                // 添加隧道
                {
                    let mut conn = agent_conn.write().await;
                    conn.add_tunnel(tunnel_id, bind_port, tunnel_type.clone());
                }

                // 发送成功响应
                let _ = broadcast_msg(&agent_conn, &TunnelResponse {
                    tunnel_id,
                    success: true,
                    public_port: Some(bind_port), // TODO: 实际应分配公网端口
                    error: None,
                }).await;

                info!("Tunnel {} 已创建，端口 {}", tunnel_id, bind_port);
            }
            TunnelData { tunnel_id, data } => {
                debug!("收到 Tunnel {} 数据，字节数: {}", tunnel_id, data.len());
                // TODO: 实际应将数据转发到目标地址
            }
            TunnelDataForward { tunnel_id, data } => {
                debug!("Tunnel {} 转发数据，字节数: {}", tunnel_id, data.len());
                // TODO: 实际应将数据转发到 Hub
            }
            TunnelClose { tunnel_id, reason } => {
                info!("关闭 Tunnel {}: {:?}", tunnel_id, reason);
                {
                    let mut conn = agent_conn.write().await;
                    conn.remove_tunnel(&tunnel_id);
                }
            }

            // ==================== P2P 消息处理 ====================
            P2pConnectRequest { request_id, target_agent_id } => {
                let agent_id = agent_conn.read().await.agent_id;
                info!("P2P 连接请求: request_id={}, from={}, target={}", request_id, agent_id, target_agent_id);
                // TODO: 查找目标 Agent 并转发请求
            }
            P2pConnectOffer { request_id, source_agent_id, sdp_offer } => {
                info!("P2P Offer: request_id={}, from={}, sdp_len={}", request_id, source_agent_id, sdp_offer.len());
                // TODO: 转发 Offer 给目标 Agent
            }
            P2pAnswer { request_id, target_agent_id, sdp_answer } => {
                info!("P2P Answer: request_id={}, to={}, sdp_len={}", request_id, target_agent_id, sdp_answer.len());
                // TODO: 转发 Answer 给源 Agent
            }
            P2pIceCandidate { request_id, candidate, sdp_mid, sdp_m_line_index } => {
                debug!("P2P ICE Candidate: request_id={}, candidate={}, mid={:?}, index={:?}", request_id, candidate, sdp_mid, sdp_m_line_index);
                // TODO: 转发 ICE Candidate 给对等方
            }
            P2pConnected { request_id } => {
                let agent_id = agent_conn.read().await.agent_id;
                info!("P2P 连接已建立: request_id={}, agent={}", request_id, agent_id);
            }
            P2pFailed { request_id, reason } => {
                let agent_id = agent_conn.read().await.agent_id;
                warn!("P2P 连接失败: request_id={}, agent={}, reason={}", request_id, agent_id, reason);
                {
                    let mut conn = agent_conn.write().await;
                    conn.remove_p2p_connection(&request_id);
                }
            }

            // ==================== 其他消息 ====================
            Unregister { reason } => {
                info!("Agent 主动断开: {:?}", reason);
                break;
            }
            Error { code, message, details } => {
                let agent_id = agent_conn.read().await.agent_id;
                error!("Agent {} 错误: {} - {} (details: {:?})", agent_id, code, message, details);
            }
            Ping { timestamp } => {
                debug!("Ping from {}", addr);
                let _ = broadcast_msg(&agent_conn, &Pong { timestamp }).await;
            }
            Pong { timestamp: _ } => {
                debug!("Pong from {}", addr);
            }
            // Hub 发送的消息不应该从 Agent 收到，但为了 exhaustive matching 添加
            RegisterAccepted { .. } |
            RegisterRejected { .. } |
            RegisterAck { .. } |
            ApprovalGranted { .. } |
            ApprovalDenied { .. } |
            TaskAssigned { .. } |
            DdnsUpdateRequest { .. } |
            SslChallengeRequest { .. } |
            TaskCancelled { .. } |
            HeartbeatAck { .. } |
            TunnelResponse { .. } => {
                warn!("收到 Hub 消息但这是 Hub 角色，忽略: {:?}", msg);
            }
        }
    }

    // 清理连接
    if let Some(agent_id) = verified_agent_id {
        connections.write().await.remove(&agent_id);
        let _ = registry.unregister(agent_id).await;
    }

    info!("与 {} 的连接已关闭", addr);
    Ok(())
}

/// 向连接发送消息
async fn broadcast_msg(
    conn: &Arc<RwLock<AgentConnection>>,
    msg: &AgentMessage,
) -> Result<(), String> {
    let mut conn = conn.write().await;
    conn.send(msg).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_hash() {
        let key = "test-secret-key";
        let hash1 = hash_key(key);
        let hash2 = hash_key(key);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 produces 64 hex chars
    }

    #[tokio::test]
    async fn test_agent_connection_new() {
        let agent_id = Uuid::new_v4();
        let session_id = "test-session".to_string();

        let conn = AgentConnection::new(agent_id, session_id.clone());

        assert_eq!(conn.agent_id, agent_id);
        assert_eq!(conn.session_id, session_id);
        assert!(conn.ws_stream.is_none());
        assert_eq!(conn.tunnel_count(), 0);
    }

    #[tokio::test]
    async fn test_tunnel_management() {
        let agent_id = Uuid::new_v4();
        let mut conn = AgentConnection::new(agent_id, "test".to_string());

        let tunnel_id = Uuid::new_v4();
        conn.add_tunnel(tunnel_id, 8080, TunnelType::Tcp);

        assert_eq!(conn.tunnel_count(), 1);
        assert!(conn.remove_tunnel(&tunnel_id).is_some());
        assert_eq!(conn.tunnel_count(), 0);
    }
}
