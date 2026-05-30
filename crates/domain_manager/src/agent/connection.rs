//! Agent WebSocket 连接管理
//!
//! 处理 Agent 的 WebSocket 连接和消息发送

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::model::{Agent, AgentStatus};
use super::protocol::{AgentMessage, Session};
use super::registry::AgentRegistry;

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
}

impl AgentConnection {
    pub fn new(agent_id: Uuid, session_id: String) -> Self {
        Self {
            agent_id,
            session_id,
            ws_stream: None,
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
}

/// Agent Hub - 管理所有 Agent 连接
pub struct AgentHub {
    registry: Arc<AgentRegistry>,
    connections: Arc<RwLock<HashMap<Uuid, Arc<RwLock<AgentConnection>>>>>,
    listener: Option<TcpListener>,
    listen_addr: String,
}

impl AgentHub {
    /// 创建新的 AgentHub
    pub fn new(registry: Arc<AgentRegistry>, listen_addr: &str) -> Self {
        Self {
            registry,
            connections: Arc::new(RwLock::new(HashMap::new())),
            listener: None,
            listen_addr: listen_addr.to_string(),
        }
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

                    info!("收到来自 {} 的连接", addr);

                    tokio::spawn(async move {
                        if let Err(e) = handle_new_connection(registry, connections, stream, addr).await {
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
            AgentMessage::RegisterWithSecret {
                agent_name,
                agent_key,
                capabilities,
                version,
                hostname,
            } => {
                info!(
                    "收到 Agent 注册请求: name={}, key={}, capabilities={:?}",
                    agent_name, agent_key, capabilities
                );

                // 验证密钥并获取agent_id
                // TODO: 从数据库验证密钥
                let agent_id = Uuid::new_v4(); // 临时生成

                // 发送注册接受消息
                let _ = broadcast_msg(&agent_conn, &AgentMessage::RegisterAccepted {
                    agent_id,
                    session_id: Uuid::new_v4().to_string(),
                    server_time: chrono::Utc::now().timestamp(),
                    requires_approval: false, // TODO: 根据配置决定是否需要审批
                    message: Some("注册已接受".to_string()),
                }).await;

                // 更新连接信息
                {
                    let mut conn = agent_conn.write().await;
                    conn.agent_id = agent_id;
                }

                info!("Agent {} 注册已接受，等待心跳", agent_id);
            }
            AgentMessage::Register {
                agent_id: _,
                name: _,
                endpoint: _,
                capabilities: _,
                version: _,
            } => {
                // 旧版注册消息，已废弃，发送拒绝
                let _ = broadcast_msg(&agent_conn, &AgentMessage::RegisterRejected {
                    reason: "Unsupported registration format".to_string(),
                    code: "OLD_PROTOCOL".to_string(),
                }).await;
                break;
            }
            AgentMessage::Heartbeat { status, metrics, timestamp } => {
                let agent_id = agent_conn.read().await.agent_id;
                if agent_id == Uuid::nil() {
                    warn!("收到未注册 Agent 的心跳");
                    continue;
                }

                debug!("收到 Agent {} 心跳: {:?}, 指标: {:?}", agent_id, status, metrics);
                registry.update_heartbeat(agent_id).await;
                registry.update_status(agent_id, status).await;

                let _ = broadcast_msg(&agent_conn, &AgentMessage::HeartbeatAck {
                    server_time: chrono::Utc::now().timestamp(),
                }).await;
            }
            AgentMessage::DdnsUpdateResult { domain, success, old_ip, new_ip, error } => {
                let agent_id = agent_conn.read().await.agent_id;
                info!("DDNS 更新结果 from {}: {} - success={}, old={:?}, new={:?}", agent_id, domain, success, old_ip, new_ip);
            }
            AgentMessage::SslChallengeResponse { domain, success, key_authorization, error } => {
                let agent_id = agent_conn.read().await.agent_id;
                info!("SSL 挑战响应 from {}: {} - success={}", agent_id, domain, success);
            }
            AgentMessage::TaskResult { task_id, success, output, error, exit_code, duration_ms } => {
                let agent_id = agent_conn.read().await.agent_id;
                info!("任务结果 from {}: task_id={}, success={}, exit_code={:?}", agent_id, task_id, success, exit_code);
            }
            _ => {
                debug!("收到未处理的消息类型: {:?}", msg);
            }
        }
    }

    let agent_id = agent_conn.read().await.agent_id;
    if agent_id != Uuid::nil() {
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

    #[tokio::test]
    async fn test_agent_connection_new() {
        let agent_id = Uuid::new_v4();
        let session_id = "test-session".to_string();

        let conn = AgentConnection::new(agent_id, session_id.clone());

        assert_eq!(conn.agent_id, agent_id);
        assert_eq!(conn.session_id, session_id);
        assert!(conn.ws_stream.is_none());
    }
}
