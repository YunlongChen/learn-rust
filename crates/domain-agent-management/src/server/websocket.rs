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

use domain_agent_protocol::diagnostic::SystemInfoReport;
use domain_agent_protocol::lifecycle::{EventSource, LifecycleEvent, LifecycleEventType};

use crate::service::Service;

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

/// Response sent back to agents (protocol format: type + payload)
#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "payload")]
enum ServerMessage {
    #[serde(rename = "RegisterAccepted")]
    RegisterAccepted(RegisterAcceptedPayload),

    #[serde(rename = "RegisterRejected")]
    RegisterRejected(RegisterRejectedPayload),

    #[serde(rename = "Error")]
    Error(ErrorPayload),
}

#[derive(Debug, Serialize)]
struct RegisterAcceptedPayload {
    agent_id: Uuid,
    session_id: String,
    server_time: i64,
    requires_approval: bool,
    message: Option<String>,
}

#[derive(Debug, Serialize)]
struct RegisterRejectedPayload {
    reason: String,
    code: String,
}

#[derive(Debug, Serialize)]
struct ErrorPayload {
    code: String,
    message: String,
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
        info!("Received text message: {:?}", &text);

        // Parse the message type field to route appropriately
        let parse_result: Result<serde_json::Value, _> = serde_json::from_str(text);

        match parse_result {
            Ok(json) => {
                let msg_type = json.get("type").and_then(|v| v.as_str());

                match msg_type {
                    Some("RegisterWithSecret") => {
                        info!("Routing message to RegisterWithSecret handler");
                        if let Some(payload) = json.get("payload") {
                            let msg: RegisterWithSecretPayload = serde_json::from_value(payload.clone())?;
                            info!("RegisterWithSecret payload: agent_name={}, hostname={}, capabilities={:?}",
                                  msg.agent_name, msg.hostname, msg.capabilities);
                            self.handle_register(write, msg).await?;
                        } else {
                            info!("RegisterWithSecret missing payload field");
                            let response = ServerMessage::Error(ErrorPayload {
                                code: "INVALID_MESSAGE".to_string(),
                                message: "Missing 'payload' field in RegisterWithSecret".to_string(),
                            });
                            self.send_response(write, &response).await?;
                        }
                    }
                    Some("SystemInfoReport") => {
                        info!("Routing message to SystemInfoReport handler");
                        if let Some(payload) = json.get("payload") {
                            let msg: SystemInfoReportPayload = serde_json::from_value(payload.clone())?;
                            info!("SystemInfoReport payload: agent_id={}, timestamp={}", msg.agent_id, msg.timestamp);
                            self.handle_system_info(write, msg).await?;
                        } else {
                            info!("SystemInfoReport missing payload field");
                            let response = ServerMessage::Error(ErrorPayload {
                                code: "INVALID_MESSAGE".to_string(),
                                message: "Missing 'payload' field in SystemInfoReport".to_string(),
                            });
                            self.send_response(write, &response).await?;
                        }
                    }
                    Some("Heartbeat") => {
                        info!("Routing message to Heartbeat handler");
                        if let Some(payload) = json.get("payload") {
                            let msg: HeartbeatPayload = serde_json::from_value(payload.clone())?;
                            info!("Heartbeat payload: status={}, timestamp={}, latency_ms={:?}",
                                  msg.status, msg.timestamp, msg.metrics.latency_ms);
                            self.handle_heartbeat(write, msg).await?;
                        } else {
                            info!("Heartbeat missing payload field");
                            let response = ServerMessage::Error(ErrorPayload {
                                code: "INVALID_MESSAGE".to_string(),
                                message: "Missing 'payload' field in Heartbeat".to_string(),
                            });
                            self.send_response(write, &response).await?;
                        }
                    }
                    Some("Unregister") => {
                        info!("Routing message to Unregister handler");
                        if let Some(payload) = json.get("payload") {
                            let msg: UnregisterPayload = serde_json::from_value(payload.clone())?;
                            info!("Unregister payload: reason={:?}", msg.reason);
                            self.handle_unregister(write, msg).await?;
                        } else {
                            info!("Unregister missing payload field");
                            let response = ServerMessage::Error(ErrorPayload {
                                code: "INVALID_MESSAGE".to_string(),
                                message: "Missing 'payload' field in Unregister".to_string(),
                            });
                            self.send_response(write, &response).await?;
                        }
                    }
                    Some(t) => {
                        info!("Unknown message type received: {}", t);
                        let response = ServerMessage::Error(ErrorPayload {
                            code: "UNKNOWN_MESSAGE_TYPE".to_string(),
                            message: format!("Unknown message type: {}", t),
                        });
                        self.send_response(write, &response).await?;
                    }
                    None => {
                        info!("Message missing 'type' field");
                        let response = ServerMessage::Error(ErrorPayload {
                            code: "INVALID_MESSAGE".to_string(),
                            message: "Missing 'type' field in message".to_string(),
                        });
                        self.send_response(write, &response).await?;
                    }
                }
            }
            Err(e) => {
                info!("Failed to parse JSON: {}", e);
                let response = ServerMessage::Error(ErrorPayload {
                    code: "INVALID_JSON".to_string(),
                    message: format!("Invalid JSON: {}", e),
                });
                self.send_response(write, &response).await?;
            }
        }

        Ok(())
    }

    /// Send a response message
    async fn send_response(
        &self,
        write: &mut futures_util::stream::SplitSink<WebSocketStream<TcpStream>, Message>,
        response: &ServerMessage,
    ) -> Result<()> {
        let json = serde_json::to_string(response)?;
        info!("Sending response: {:?}", &json);
        write.send(Message::Text(json)).await?;
        Ok(())
    }

    /// Handle RegisterWithSecret message
    async fn handle_register(
        &self,
        write: &mut futures_util::stream::SplitSink<WebSocketStream<TcpStream>, Message>,
        msg: RegisterWithSecretPayload,
    ) -> Result<()> {
        info!("Processing RegisterWithSecret: agent_name={}, hostname={}", msg.agent_name, msg.hostname);

        // Determine the agent_id to use
        let agent_id = match &msg.agent_id {
            Some(agent_id_str) => {
                if let Ok(agent_id) = Uuid::parse_str(agent_id_str) {
                    match self.service.agent_service.get_agent(agent_id).await {
                        Ok(Some(agent)) => {
                            if agent.approval_state == "denied" {
                                info!("Agent {} was denied, rejecting registration", agent_id);
                                let response = ServerMessage::RegisterRejected(RegisterRejectedPayload {
                                    code: "AGENT_DENIED".to_string(),
                                    reason: "Agent was denied by administrator".to_string(),
                                });
                                self.send_response(write, &response).await?;
                                return Ok(());
                            }
                            // Agent exists with non-denied state, use existing ID
                            info!("Agent {} exists with approval_state={}, proceeding", agent_id, agent.approval_state);
                            agent_id
                        }
                        Ok(None) => {
                            // Agent doesn't exist, use the provided ID for new registration
                            info!("Agent {} not found, will create new record with provided ID", agent_id);
                            agent_id
                        }
                        Err(e) => {
                            // Query failed, generate new ID
                            warn!("Failed to query agent {}: {}, generating new ID", agent_id, e);
                            Uuid::new_v4()
                        }
                    }
                } else {
                    // Invalid UUID format, generate new
                    warn!("Invalid agent_id format: {}, generating new ID", agent_id_str);
                    Uuid::new_v4()
                }
            }
            None => {
                // No agent_id provided, generate new
                info!("No agent_id provided, generating new ID");
                Uuid::new_v4()
            }
        };

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

        info!("Creating agent with id={}, name={}", agent_id, msg.agent_name);

        // Create agent via AgentService
        match self.service.agent_service.create_agent(input).await {
            Ok(_agent_info) => {
                info!("Agent created successfully: id={}, name={}", agent_id, msg.agent_name);

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
                } else {
                    info!("Lifecycle event recorded for agent: id={}", agent_id);
                }

                // Send RegisterAccepted response
                let session_id = Uuid::new_v4().to_string();
                info!("Sending RegisterAccepted: agent_id={}, session_id={}", agent_id, session_id);
                let response = ServerMessage::RegisterAccepted(RegisterAcceptedPayload {
                    agent_id,
                    session_id,
                    server_time: Utc::now().timestamp(),
                    requires_approval: false,
                    message: Some("Registration successful".to_string()),
                });
                self.send_response(write, &response).await?;
            }
            Err(e) => {
                error!("Failed to create agent {}: {}", msg.agent_name, e);
                info!("Sending RegisterRejected for agent: {}", msg.agent_name);
                let response = ServerMessage::RegisterRejected(RegisterRejectedPayload {
                    reason: e.to_string(),
                    code: "REGISTRATION_FAILED".to_string(),
                });
                self.send_response(write, &response).await?;
            }
        }

        Ok(())
    }

    /// Handle SystemInfoReport message
    async fn handle_system_info(
        &self,
        write: &mut futures_util::stream::SplitSink<WebSocketStream<TcpStream>, Message>,
        msg: SystemInfoReportPayload,
    ) -> Result<()> {
        let agent_id = match Uuid::parse_str(&msg.agent_id) {
            Ok(id) => id,
            Err(_) => {
                info!("Invalid agent_id format in SystemInfoReport: {}", msg.agent_id);
                let response = ServerMessage::Error(ErrorPayload {
                    code: "INVALID_AGENT_ID".to_string(),
                    message: "Invalid agent_id format".to_string(),
                });
                self.send_response(write, &response).await?;
                return Ok(());
            }
        };

        info!("Processing SystemInfoReport: agent_id={}, timestamp={}", agent_id, msg.timestamp);

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

        info!("Storing system info for agent_id={}", agent_id);
        match self.service.diagnostic_service.store_system_info(agent_id, report).await {
            Ok(_) => {
                info!("System info stored successfully for agent_id={}", agent_id);
                let response = ServerMessage::RegisterAccepted(RegisterAcceptedPayload {
                    agent_id,
                    session_id: String::new(),
                    server_time: Utc::now().timestamp(),
                    requires_approval: false,
                    message: Some("System info recorded".to_string()),
                });
                self.send_response(write, &response).await?;
            }
            Err(e) => {
                error!("Failed to store system info for agent_id={}: {}", agent_id, e);
                let response = ServerMessage::Error(ErrorPayload {
                    code: "SYSTEM_INFO_FAILED".to_string(),
                    message: e.to_string(),
                });
                self.send_response(write, &response).await?;
            }
        }

        Ok(())
    }

    /// Extract OS info from the data JSON
    fn extract_os_info(&self, data: &serde_json::Value) -> Option<domain_agent_protocol::diagnostic::OsInfo> {
        data.get("os").and_then(|os| {
            serde_json::from_value(os.clone()).ok()
        })
    }

    /// Extract environment info from the data JSON
    fn extract_environment_info(&self, data: &serde_json::Value) -> Option<domain_agent_protocol::diagnostic::EnvironmentInfo> {
        data.get("environment").and_then(|env| {
            serde_json::from_value(env.clone()).ok()
        })
    }

    /// Extract process info from the data JSON
    fn extract_process_info(&self, data: &serde_json::Value) -> Option<domain_agent_protocol::diagnostic::ProcessInfo> {
        data.get("process").and_then(|process| {
            serde_json::from_value(process.clone()).ok()
        })
    }

    /// Extract network info from the data JSON
    fn extract_network_info(&self, data: &serde_json::Value) -> Option<domain_agent_protocol::diagnostic::NetworkInfo> {
        data.get("network").and_then(|network| {
            serde_json::from_value(network.clone()).ok()
        })
    }

    /// Extract resource info from the data JSON
    fn extract_resource_info(&self, data: &serde_json::Value) -> Option<domain_agent_protocol::diagnostic::ResourceInfo> {
        data.get("resources").and_then(|resources| {
            serde_json::from_value(resources.clone()).ok()
        })
    }

    /// Handle Heartbeat message
    async fn handle_heartbeat(
        &self,
        write: &mut futures_util::stream::SplitSink<WebSocketStream<TcpStream>, Message>,
        msg: HeartbeatPayload,
    ) -> Result<()> {
        info!("Processing Heartbeat: status={}, timestamp={}", msg.status, msg.timestamp);

        // Extract metrics for health score calculation
        // Note: Without agent_id, we can't record health score
        let network_metrics = crate::service::health::NetworkHealthMetrics {
            latency_ms: msg.metrics.latency_ms,
            jitter_ms: msg.metrics.jitter_ms,
            packet_loss_percent: msg.metrics.packet_loss_percent,
            bandwidth_kbps: msg.metrics.bandwidth_kbps,
        };

        info!("Heartbeat metrics: latency_ms={:?}, jitter_ms={:?}, packet_loss={:?}, bandwidth={:?}",
              network_metrics.latency_ms, network_metrics.jitter_ms,
              network_metrics.packet_loss_percent, network_metrics.bandwidth_kbps);

        // Note: Without agent_id, we can't update last_seen_at or record health score
        // The agent would need to send its ID in the heartbeat or we need another way to identify it

        // Send heartbeat acknowledgment
        let response = ServerMessage::RegisterAccepted(RegisterAcceptedPayload {
            agent_id: Uuid::nil(),
            session_id: String::new(),
            server_time: Utc::now().timestamp(),
            requires_approval: false,
            message: Some("Heartbeat received".to_string()),
        });
        self.send_response(write, &response).await?;

        Ok(())
    }

    /// Handle Unregister message
    async fn handle_unregister(
        &self,
        write: &mut futures_util::stream::SplitSink<WebSocketStream<TcpStream>, Message>,
        msg: UnregisterPayload,
    ) -> Result<()> {
        info!("Processing Unregister: reason={:?}", msg.reason);

        // Note: We should update agent status to "disconnected" here
        // But we don't have agent_id from the unregister message itself
        // In a real implementation, we'd need to track the agent_id from registration

        info!("Sending Unregister acknowledgment");
        let response = ServerMessage::RegisterAccepted(RegisterAcceptedPayload {
            agent_id: Uuid::nil(),
            session_id: String::new(),
            server_time: Utc::now().timestamp(),
            requires_approval: false,
            message: Some("Unregister acknowledged".to_string()),
        });
        self.send_response(write, &response).await?;

        Ok(())
    }
}

// Message payload types for deserialization (extracted from 'payload' field)

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RegisterWithSecretPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    agent_id: Option<String>,
    agent_name: String,
    agent_key: String,
    capabilities: Vec<String>,
    version: Option<String>,
    hostname: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SystemInfoReportPayload {
    agent_id: String,
    timestamp: String,
    data: SystemInfoData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HeartbeatPayload {
    status: String,
    metrics: HeartbeatMetrics,
    timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UnregisterPayload {
    reason: Option<String>,
}

/// Runs the WebSocket server as a background task
pub async fn run_websocket_server(service: Service, addr: String) {
    let server = WebSocketServer::new(service);
    if let Err(e) = server.start(&addr).await {
        error!("WebSocket server error: {}", e);
    }
}
