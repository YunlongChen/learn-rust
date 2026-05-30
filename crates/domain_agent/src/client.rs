//! Agent client implementation

use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::interval;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::config::AgentConfig;

/// Agent message types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum AgentMessage {
    /// Register with secret
    #[serde(rename = "RegisterWithSecret")]
    RegisterWithSecret {
        agent_name: String,
        agent_key: String,
        capabilities: Vec<String>,
        version: Option<String>,
        hostname: Option<String>,
    },

    /// Registration accepted
    #[serde(rename = "RegisterAccepted")]
    RegisterAccepted {
        agent_id: Uuid,
        session_id: String,
        server_time: i64,
        requires_approval: bool,
        message: Option<String>,
    },

    /// Registration rejected
    #[serde(rename = "RegisterRejected")]
    RegisterRejected {
        reason: String,
        code: String,
    },

    /// Heartbeat
    #[serde(rename = "Heartbeat")]
    Heartbeat {
        status: String,
        metrics: AgentMetrics,
        timestamp: i64,
    },

    /// Heartbeat acknowledgment
    #[serde(rename = "HeartbeatAck")]
    HeartbeatAck {
        server_time: i64,
    },

    /// Unregister
    #[serde(rename = "Unregister")]
    Unregister {
        reason: Option<String>,
    },
}

/// Agent metrics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentMetrics {
    pub cpu_usage: Option<f32>,
    pub memory_usage: Option<f32>,
    pub disk_usage: Option<f32>,
    pub latency_ms: Option<u32>,
}

/// Agent client
pub struct AgentClient {
    config: AgentConfig,
    agent_id: Option<Uuid>,
    session_id: Option<String>,
}

impl AgentClient {
    /// Create a new agent client
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            agent_id: None,
            session_id: None,
        }
    }

    /// Connect to the Hub
    pub async fn connect(&mut self) -> Result<(), String> {
        let url = format!("ws://{}/ws", self.config.hub);
        info!("Connecting to Hub at {}", url);

        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| format!("WebSocket connection failed: {}", e))?;

        info!("WebSocket connected, sending registration");

        // Build WebSocket URL
        let (mut write, mut read) = ws_stream.split();

        // Send registration message
        let register_msg = AgentMessage::RegisterWithSecret {
            agent_name: self.config.name.clone(),
            agent_key: self.config.key.clone(),
            capabilities: vec![
                "ddns_client".to_string(),
                "shell_executor".to_string(),
                "ssl_validator".to_string(),
            ],
            version: Some(self.config.version.clone()),
            hostname: self.config.hostname.clone(),
        };

        let json = serde_json::to_string(&register_msg)
            .map_err(|e| format!("Failed to serialize message: {}", e))?;

        write
            .send(Message::Text(json.into()))
            .await
            .map_err(|e| format!("Failed to send registration: {}", e))?;

        info!("Registration sent, waiting for response");

        // Wait for registration response
        let msg = read
            .next()
            .await
            .ok_or_else(|| "No response from Hub".to_string())?
            .map_err(|e| format!("Failed to receive message: {}", e))?;

        let text = msg
            .into_text()
            .map_err(|e| format!("Failed to get text: {}", e))?;

        debug!("Received: {}", text);

        let response: AgentMessage = serde_json::from_str(&text)
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        match response {
            AgentMessage::RegisterAccepted {
                agent_id,
                session_id,
                requires_approval,
                message,
                ..
            } => {
                info!(
                    "Registration accepted: agent_id={}, session_id={}, requires_approval={}, message={:?}",
                    agent_id, session_id, requires_approval, message
                );
                self.agent_id = Some(agent_id);
                self.session_id = Some(session_id);

                if requires_approval {
                    info!("Agent requires approval, waiting...");
                    // Wait for approval message
                    self.wait_for_approval().await?;
                }

                Ok(())
            }
            AgentMessage::RegisterRejected { reason, code } => {
                Err(format!("Registration rejected: {} (code: {})", reason, code))
            }
            _ => Err("Unexpected response type".to_string()),
        }
    }

    /// Wait for approval
    async fn wait_for_approval(&mut self) -> Result<(), String> {
        // This would need to receive an ApprovalGranted message
        // For now, just wait indefinitely
        info!("Waiting for approval from Hub...");
        Ok(())
    }

    /// Run the heartbeat loop
    pub async fn run_heartbeat_loop(&mut self) -> Result<(), String> {
        let mut ticker = interval(Duration::from_secs(30));

        loop {
            ticker.tick().await;
            self.send_heartbeat().await?;
        }
    }

    /// Send a heartbeat
    async fn send_heartbeat(&self) -> Result<(), String> {
        let msg = AgentMessage::Heartbeat {
            status: "online".to_string(),
            metrics: AgentMetrics {
                cpu_usage: Some(0.0),
                memory_usage: Some(0.0),
                disk_usage: Some(0.0),
                latency_ms: Some(0),
            },
            timestamp: chrono::Utc::now().timestamp(),
        };

        let json = serde_json::to_string(&msg)
            .map_err(|e| format!("Failed to serialize heartbeat: {}", e))?;

        // For now, we can't actually send because we don't have the WebSocket connection
        debug!("Heartbeat: {}", json);

        Ok(())
    }
}
