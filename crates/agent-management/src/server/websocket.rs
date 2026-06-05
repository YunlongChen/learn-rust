//! WebSocket server implementation for agent connections
//!
//! Accepts WebSocket connections from agents and handles:
//! - RegisterWithSecret: Agent registration with secret key
//! - SystemInfoReport: System diagnostic information
//! - Heartbeat: Agent heartbeat with health metrics

use anyhow::Result;
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use tracing::{error, info, warn};
use uuid::Uuid;

use agent_protocol::diagnostic::SystemInfoReport;
use agent_protocol::lifecycle::{EventSource, LifecycleEvent, LifecycleEventType};

use crate::service::Service;

/// WebSocket message types received from agents
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[allow(dead_code)]
enum AgentMessage {
    #[serde(rename = "RegisterWithSecret")]
    RegisterWithSecret {
        agent_name: String,
        agent_key: String,
        capabilities: Vec<String>,
        version: Option<String>,
        hostname: String,
    },

    #[serde(rename = "SystemInfoReport")]
    SystemInfoReport {
        agent_id: String,
        timestamp: String,
        data: SystemInfoData,
    },

    #[serde(rename = "Heartbeat")]
    Heartbeat {
        status: String,
        metrics: HeartbeatMetrics,
        timestamp: i64,
    },
}

/// System information data nested in SystemInfoReport
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SystemInfoData {
    #[serde(flatten)]
    extra: serde_json::Value,
}

/// Heartbeat metrics from agent
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HeartbeatMetrics {
    latency_ms: Option<f64>,
    jitter_ms: Option<f64>,
    packet_loss_percent: Option<f64>,
    bandwidth_kbps: Option<f64>,
}

/// Response sent back to agents
#[derive(Debug, Serialize)]
struct ResponseMessage {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    agent_id: Option<String>,
}

impl ResponseMessage {
    fn success(msg: impl Into<String>) -> Self {
        Self {
            success: true,
            message: msg.into(),
            agent_id: None,
        }
    }

    fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            message: msg.into(),
            agent_id: None,
        }
    }

    fn with_agent_id(mut self, agent_id: Uuid) -> Self {
        self.agent_id = Some(agent_id.to_string());
        self
    }
}

/// WebSocket server for handling agent connections
#[derive(Debug, Clone)]
pub struct WebSocketServer {
    service: Service,
}

impl WebSocketServer {
    /// Creates a new WebSocket server instance
    pub fn new(service: Service) -> Self {
        Self { service }
    }

    /// Starts the WebSocket server, listening for connections
    pub async fn start(&self, addr: &str) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        info!("WebSocket server listening on {}", addr);

        let server = Arc::new(self.clone());

        loop {
            match listener.accept().await {
                Ok((stream, peer_addr)) => {
                    info!("New WebSocket connection from {}", peer_addr);
                    let server = server.clone();
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_connection(stream).await {
                            error!("Error handling connection from {}: {}", peer_addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    /// Handles a single WebSocket connection
    async fn handle_connection(&self, stream: TcpStream) -> Result<()> {
        let ws_stream = tokio_tungstenite::accept_async(stream).await?;
        let (mut write, mut read) = ws_stream.split();

        while let Some(msg_result) = read.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    if let Err(e) = self.process_message(&mut write, &text).await {
                        error!("Error processing message: {}", e);
                    }
                }
                Ok(Message::Binary(data)) => {
                    if let Ok(text) = String::from_utf8(data) {
                        if let Err(e) = self.process_message(&mut write, &text).await {
                            error!("Error processing binary message: {}", e);
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection closed");
                    break;
                }
                Ok(Message::Ping(data)) => {
                    if write.send(Message::Pong(data)).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    warn!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Process a single message from an agent
    async fn process_message(
        &self,
        write: &mut futures_util::stream::SplitSink<WebSocketStream<TcpStream>, Message>,
        text: &str,
    ) -> Result<()> {
        // Parse the message type field to route appropriately
        let parse_result: Result<serde_json::Value, _> = serde_json::from_str(text);

        match parse_result {
            Ok(json) => {
                let msg_type = json.get("type").and_then(|v| v.as_str());

                match msg_type {
                    Some("RegisterWithSecret") => {
                        let msg: RegisterWithSecretMsg = serde_json::from_str(text)?;
                        self.handle_register(write, msg).await?;
                    }
                    Some("SystemInfoReport") => {
                        let msg: SystemInfoReportMsg = serde_json::from_str(text)?;
                        self.handle_system_info(write, msg).await?;
                    }
                    Some("Heartbeat") => {
                        let msg: HeartbeatMsg = serde_json::from_str(text)?;
                        self.handle_heartbeat(write, msg).await?;
                    }
                    Some(t) => {
                        let response = ResponseMessage::error(format!("Unknown message type: {}", t));
                        self.send_response(write, &response).await?;
                    }
                    None => {
                        let response = ResponseMessage::error("Missing 'type' field in message");
                        self.send_response(write, &response).await?;
                    }
                }
            }
            Err(e) => {
                let response = ResponseMessage::error(format!("Invalid JSON: {}", e));
                self.send_response(write, &response).await?;
            }
        }

        Ok(())
    }

    /// Send a response message
    async fn send_response(
        &self,
        write: &mut futures_util::stream::SplitSink<WebSocketStream<TcpStream>, Message>,
        response: &ResponseMessage,
    ) -> Result<()> {
        let json = serde_json::to_string(response)?;
        write.send(Message::Text(json)).await?;
        Ok(())
    }

    /// Handle RegisterWithSecret message
    async fn handle_register(
        &self,
        write: &mut futures_util::stream::SplitSink<WebSocketStream<TcpStream>, Message>,
        msg: RegisterWithSecretMsg,
    ) -> Result<()> {
        info!("Processing RegisterWithSecret for agent: {}", msg.agent_name);

        // Generate a new agent ID
        let agent_id = Uuid::new_v4();
        let now = Utc::now();

        // Create agent input
        let input = crate::service::agent::CreateAgentInput {
            id: agent_id,
            name: msg.agent_name.clone(),
            endpoint: format!("ws://{}", msg.hostname),
            status: "pending".to_string(),
            approval_state: "pending".to_string(),
            capabilities: serde_json::json!(msg.capabilities).into(),
            cert_fingerprint: None,
            auth_method: "secret".to_string(),
            version: msg.version,
            registered_at: Some(now),
            last_seen_at: Some(now),
        };

        // Create agent via AgentService
        match self.service.agent_service.create_agent(input).await {
            Ok(_agent_info) => {
                // Record lifecycle event
                let event = LifecycleEvent::new(
                    agent_id,
                    LifecycleEventType::AgentRegistering,
                    EventSource::Agent,
                )
                .with_reason("Agent registration via WebSocket")
                .with_metadata(serde_json::json!({
                    "agent_name": msg.agent_name,
                    "hostname": msg.hostname,
                }));

                if let Err(e) = self.service.lifecycle_service.record_event(&event).await {
                    error!("Failed to record lifecycle event: {}", e);
                }

                // Send success response with agent_id
                let response = ResponseMessage::success("Registration successful")
                    .with_agent_id(agent_id);
                self.send_response(write, &response).await?;
            }
            Err(e) => {
                error!("Failed to create agent: {}", e);
                let response = ResponseMessage::error(format!("Registration failed: {}", e));
                self.send_response(write, &response).await?;
            }
        }

        Ok(())
    }

    /// Handle SystemInfoReport message
    async fn handle_system_info(
        &self,
        write: &mut futures_util::stream::SplitSink<WebSocketStream<TcpStream>, Message>,
        msg: SystemInfoReportMsg,
    ) -> Result<()> {
        let agent_id = match Uuid::parse_str(&msg.agent_id) {
            Ok(id) => id,
            Err(_) => {
                let response = ResponseMessage::error("Invalid agent_id format");
                self.send_response(write, &response).await?;
                return Ok(());
            }
        };

        info!("Processing SystemInfoReport for agent: {}", agent_id);

        // Parse timestamp
        let timestamp = DateTime::parse_from_rfc3339(&msg.timestamp)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        // Convert the data to SystemInfoReport format
        let report = SystemInfoReport {
            report_id: Uuid::new_v4(),
            agent_id,
            timestamp,
            os: self.extract_os_info(&msg.data.extra),
            environment: self.extract_environment_info(&msg.data.extra),
            process: self.extract_process_info(&msg.data.extra),
            network: self.extract_network_info(&msg.data.extra),
            resources: self.extract_resource_info(&msg.data.extra),
        };

        match self.service.diagnostic_service.store_system_info(agent_id, report).await {
            Ok(_) => {
                let response = ResponseMessage::success("System info recorded");
                self.send_response(write, &response).await?;
            }
            Err(e) => {
                error!("Failed to store system info: {}", e);
                let response = ResponseMessage::error(format!("Failed to store system info: {}", e));
                self.send_response(write, &response).await?;
            }
        }

        Ok(())
    }

    /// Extract OS info from the data JSON
    fn extract_os_info(&self, data: &serde_json::Value) -> Option<agent_protocol::diagnostic::OsInfo> {
        data.get("os").and_then(|os| {
            serde_json::from_value(os.clone()).ok()
        })
    }

    /// Extract environment info from the data JSON
    fn extract_environment_info(&self, data: &serde_json::Value) -> Option<agent_protocol::diagnostic::EnvironmentInfo> {
        data.get("environment").and_then(|env| {
            serde_json::from_value(env.clone()).ok()
        })
    }

    /// Extract process info from the data JSON
    fn extract_process_info(&self, data: &serde_json::Value) -> Option<agent_protocol::diagnostic::ProcessInfo> {
        data.get("process").and_then(|process| {
            serde_json::from_value(process.clone()).ok()
        })
    }

    /// Extract network info from the data JSON
    fn extract_network_info(&self, data: &serde_json::Value) -> Option<agent_protocol::diagnostic::NetworkInfo> {
        data.get("network").and_then(|network| {
            serde_json::from_value(network.clone()).ok()
        })
    }

    /// Extract resource info from the data JSON
    fn extract_resource_info(&self, data: &serde_json::Value) -> Option<agent_protocol::diagnostic::ResourceInfo> {
        data.get("resources").and_then(|resources| {
            serde_json::from_value(resources.clone()).ok()
        })
    }

    /// Handle Heartbeat message
    async fn handle_heartbeat(
        &self,
        write: &mut futures_util::stream::SplitSink<WebSocketStream<TcpStream>, Message>,
        msg: HeartbeatMsg,
    ) -> Result<()> {
        info!("Processing Heartbeat with status: {}", msg.status);

        // Get agent_id from heartbeat - need to find agent first
        // For now, we'll skip the update if we don't have an agent_id
        // The heartbeat contains status but not agent_id directly
        // We may need to look this up or have the agent include it

        // Extract metrics for health score calculation
        // Note: Without agent_id, we can't record health score
        let _network_metrics = crate::service::health::NetworkHealthMetrics {
            latency_ms: msg.metrics.latency_ms,
            jitter_ms: msg.metrics.jitter_ms,
            packet_loss_percent: msg.metrics.packet_loss_percent,
            bandwidth_kbps: msg.metrics.bandwidth_kbps,
        };

        // Note: Without agent_id, we can't update last_seen_at or record health score
        // The agent would need to send its ID in the heartbeat or we need another way to identify it

        let response = ResponseMessage::success("Heartbeat received");
        self.send_response(write, &response).await?;

        Ok(())
    }
}

// Message type wrappers for deserialization

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RegisterWithSecretMsg {
    #[serde(rename = "type")]
    msg_type: String,
    agent_name: String,
    agent_key: String,
    capabilities: Vec<String>,
    version: Option<String>,
    hostname: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SystemInfoReportMsg {
    #[serde(rename = "type")]
    msg_type: String,
    agent_id: String,
    timestamp: String,
    data: SystemInfoData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HeartbeatMsg {
    #[serde(rename = "type")]
    msg_type: String,
    status: String,
    metrics: HeartbeatMetrics,
    timestamp: i64,
}

/// Runs the WebSocket server as a background task
pub async fn run_websocket_server(service: Service, addr: String) {
    let server = WebSocketServer::new(service);
    if let Err(e) = server.start(&addr).await {
        error!("WebSocket server error: {}", e);
    }
}
