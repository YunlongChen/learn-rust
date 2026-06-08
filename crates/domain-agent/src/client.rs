//! Agent client implementation with robust network handling

use futures_util::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio::time::interval;
use tokio_tungstenite::{client_async, connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use domain_agent_protocol::{SystemInfoQuery, SystemInfoReport, SystemInfoResponse};
use crate::config::{AgentConfig, ProxyConfig};
use crate::diagnostic::collect_system_info;

/// Agent connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentState {
    Disconnected,
    Connecting,
    Connected,
    Registered,
    PendingApproval,
    Reconnecting,
    ShuttingDown,
    Closed,
}

/// Type alias for WebSocket stream with default TLS
type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WsSink = futures_util::stream::SplitSink<WsStream, Message>;
type WsRead = futures_util::stream::SplitStream<WsStream>;

/// Agent client with persistent connection
pub struct AgentClient {
    config: AgentConfig,
    state: Arc<AtomicU32>,
    agent_id: Arc<RwLock<Option<Uuid>>>,
    session_id: Arc<RwLock<Option<String>>>,

    // WebSocket resources
    ws_write: Arc<Mutex<Option<WsSink>>>,
    ws_read: Arc<RwLock<Option<WsRead>>>,

    // Reconnection
    reconnect: Arc<ReconnectionManager>,

    // Shutdown signal
    shutdown_tx: Arc<RwLock<Option<broadcast::Sender<()>>>>,
}

impl AgentClient {
    /// Create a new agent client
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            state: Arc::new(AtomicU32::new(AgentState::Disconnected as u32)),
            agent_id: Arc::new(RwLock::new(None)),
            session_id: Arc::new(RwLock::new(None)),
            ws_write: Arc::new(Mutex::new(None)),
            ws_read: Arc::new(RwLock::new(None)),
            reconnect: Arc::new(ReconnectionManager::new()),
            shutdown_tx: Arc::new(RwLock::new(None)),
        }
    }

    /// Get current state
    pub fn get_state(&self) -> AgentState {
        AgentState::from(self.state.load(Ordering::SeqCst))
    }

    /// Set state
    fn set_state(&self, state: AgentState) {
        self.state.store(state as u32, Ordering::SeqCst);
    }

    /// Connect to the Hub
    pub async fn connect(&mut self) -> Result<(), String> {
        self.set_state(AgentState::Connecting);

        let url = format!("ws://{}/ws", self.config.hub);
        info!("Connecting to Hub at {} (proxy: {:?})", url, self.config.proxy);

        // Create WebSocket stream
        let ws_stream = self.create_ws_stream(&url).await?;

        info!("WebSocket connected");

        // Split and save the stream
        let (write, read) = ws_stream.split();

        {
            let mut write_guard = self.ws_write.lock().await;
            *write_guard = Some(write);
        }
        {
            let mut read_guard = self.ws_read.write().await;
            *read_guard = Some(read);
        }

        self.set_state(AgentState::Connected);
        info!("WebSocket stream saved for later use");

        // Send registration
        self.send_registration().await?;

        // Wait for registration response
        self.wait_for_registration_response().await?;

        Ok(())
    }

    /// Create WebSocket stream, with proxy support
    async fn create_ws_stream(&self, url: &str) -> Result<WsStream, String> {
        match &self.config.proxy {
            ProxyConfig::None => {
                // Direct connection
                let (ws_stream, _) = connect_async(url)
                    .await
                    .map_err(|e| format!("WebSocket connection failed: {}", e))?;
                Ok(ws_stream)
            }
            ProxyConfig::Socks5 { host, port, auth } => {
                // SOCKS5 proxy
                let target = url.trim_start_matches("ws://").trim_start_matches("wss://");
                let (target_host, target_port) = self.parse_ws_url(target)?;
                info!("Connecting via SOCKS5 proxy {}:{} to {}:{}", host, port, target_host, target_port);

                let proxy_stream = crate::proxy::connect_via_socks5(
                    host, *port, &target_host, target_port, auth.as_ref()
                ).await?;

                // Wrap in MaybeTlsStream::Plain
                let stream = MaybeTlsStream::Plain(proxy_stream.stream);

                // Upgrade to WebSocket
                let ws_stream = client_async(url, stream)
                    .await
                    .map_err(|e| format!("WebSocket upgrade failed: {}", e))?
                    .0;
                Ok(ws_stream)
            }
            ProxyConfig::Http { host, port, auth } => {
                // HTTP CONNECT proxy
                let target = url.trim_start_matches("ws://").trim_start_matches("wss://");
                let (target_host, target_port) = self.parse_ws_url(target)?;
                info!("Connecting via HTTP proxy {}:{} to {}:{}", host, port, target_host, target_port);

                let proxy_stream = crate::proxy::connect_via_http_proxy(
                    host, *port, &target_host, target_port, auth.as_ref()
                ).await?;

                // Wrap in MaybeTlsStream::Plain
                let stream = MaybeTlsStream::Plain(proxy_stream.stream);

                // Upgrade to WebSocket
                let ws_stream = client_async(url, stream)
                    .await
                    .map_err(|e| format!("WebSocket upgrade failed: {}", e))?
                    .0;
                Ok(ws_stream)
            }
        }
    }

    /// Parse WebSocket URL into host and port
    fn parse_ws_url(&self, url: &str) -> Result<(String, u16), String> {
        let url = url.split_once('/').map(|(h, _)| h).unwrap_or(url);
        let (host, port_str) = if url.contains(':') {
            let parts: Vec<&str> = url.split(':').collect();
            (parts[0].to_string(), parts[1])
        } else {
            (url.to_string(), "80")
        };
        let port: u16 = port_str.parse()
            .map_err(|_| format!("Invalid port in URL: {}", url))?;
        Ok((host, port))
    }

    /// Send registration message
    async fn send_registration(&self) -> Result<(), String> {
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

        self.send_message(&json).await?;
        info!("Registration sent");

        Ok(())
    }

    /// Wait for registration response
    async fn wait_for_registration_response(&mut self) -> Result<(), String> {
        let msg = self.receive_message().await?;

        let response: AgentMessage = serde_json::from_str(&msg)
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
                *self.agent_id.write().await = Some(agent_id);
                *self.session_id.write().await = Some(session_id);

                if requires_approval {
                    self.set_state(AgentState::PendingApproval);
                    info!("Agent requires approval, waiting...");
                    self.wait_for_approval().await?;
                } else {
                    self.set_state(AgentState::Registered);
                }

                self.reconnect.reset();
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
        loop {
            tokio::select! {
                msg = self.receive_message() => {
                    match msg {
                        Ok(msg) => {
                            let response: AgentMessage = serde_json::from_str(&msg)
                                .map_err(|e| format!("Failed to parse: {}", e))?;

                            match response {
                                AgentMessage::ApprovalGranted { message, .. } => {
                                    info!("Approval granted: {:?}", message);
                                    self.set_state(AgentState::Registered);
                                    return Ok(());
                                }
                                AgentMessage::ApprovalDenied { reason } => {
                                    return Err(format!("Approval denied: {}", reason));
                                }
                                _ => continue,
                            }
                        }
                        Err(e) => return Err(e),
                    }
                }
                _ = tokio::time::sleep(Duration::from_secs(60)) => {
                    return Err("Approval timeout".to_string());
                }
            }
        }
    }

    /// Send a text message
    async fn send_message(&self, msg: &str) -> Result<(), String> {
        let mut write_guard = self.ws_write.lock().await;
        let write = write_guard.as_mut()
            .ok_or_else(|| "WebSocket not connected".to_string())?;

        write.send(Message::Text(msg.into()))
            .await
            .map_err(|e| format!("Failed to send message: {}", e))?;

        Ok(())
    }

    /// Receive a text message
    async fn receive_message(&self) -> Result<String, String> {
        let mut read_guard = self.ws_read.write().await;
        let read = read_guard.as_mut()
            .ok_or_else(|| "WebSocket not connected".to_string())?;

        match read.next().await {
            Some(Ok(msg)) => {
                let text = msg.into_text()
                    .map_err(|e| format!("Failed to get text: {}", e))?;
                debug!("Received: {}", text);
                Ok(text)
            }
            Some(Err(e)) => Err(format!("WebSocket error: {}", e)),
            None => Err("WebSocket stream ended".to_string()),
        }
    }

    /// Run the main loop: heartbeat and message handling
    pub async fn run(&mut self) -> Result<(), String> {
        info!("Starting main loop");

        let mut heartbeat_ticker = interval(Duration::from_secs(30));
        let mut consecutive_failures = 0u32;

        loop {
            tokio::select! {
                // Heartbeat
                _ = heartbeat_ticker.tick() => {
                    if self.get_state() == AgentState::Registered {
                        match self.send_heartbeat().await {
                            Ok(_) => {
                                consecutive_failures = 0;
                            }
                            Err(e) => {
                                warn!("Heartbeat failed: {}", e);
                                consecutive_failures += 1;
                                if consecutive_failures >= 3 {
                                    error!("Too many heartbeat failures, reconnecting...");
                                    self.handle_disconnect().await?;
                                }
                            }
                        }
                    }
                }

                // Incoming messages
                msg = self.receive_message() => {
                    match msg {
                        Ok(msg) => {
                            if let Err(e) = self.handle_message(&msg).await {
                                warn!("Message handling error: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("Message receive error: {}", e);
                            self.handle_disconnect().await?;
                            break;
                        }
                    }
                }

                // Shutdown signal
                _ = self.wait_for_shutdown() => {
                    info!("Shutdown signal received");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle incoming message
    async fn handle_message(&self, msg: &str) -> Result<(), String> {
        let response: AgentMessage = serde_json::from_str(msg)
            .map_err(|e| format!("Failed to parse message: {}", e))?;

        match response {
            AgentMessage::HeartbeatAck { server_time } => {
                debug!("Heartbeat acknowledged, server time: {}", server_time);
            }
            AgentMessage::ApprovalGranted { message, .. } => {
                info!("Approval granted while in Registered state: {:?}", message);
            }
            AgentMessage::Unregister { reason } => {
                info!("Hub requested unregister: {:?}", reason);
                return Err("Hub requested unregister".to_string());
            }
            AgentMessage::SystemInfoQuery { query } => {
                info!("Received SystemInfoQuery: {:?}", query.query_id);
                // Respond with system info report
                if let Err(e) = self.send_system_info_report().await {
                    warn!("Failed to send system info report: {}", e);
                }
            }
            _ => {
                debug!("Unhandled message type: {:?}", response);
            }
        }

        Ok(())
    }

    /// Send heartbeat using saved write half
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

        self.send_message(&json).await?;
        debug!("Heartbeat sent");

        Ok(())
    }

    /// Send system info report
    pub async fn send_system_info_report(&self) -> Result<(), String> {
        let agent_id = *self.agent_id.read().await;
        let agent_id = agent_id.ok_or_else(|| "Agent not registered".to_string())?;

        let report = collect_system_info(agent_id);

        let msg = AgentMessage::SystemInfoReport { report };

        let json = serde_json::to_string(&msg)
            .map_err(|e| format!("Failed to serialize system info report: {}", e))?;

        self.send_message(&json).await?;
        info!("System info report sent");

        Ok(())
    }

    /// Handle disconnection with reconnection
    async fn handle_disconnect(&mut self) -> Result<(), String> {
        self.set_state(AgentState::Reconnecting);
        self.clear_connection().await;

        while self.reconnect.should_retry() {
            let delay = self.reconnect.get_next_delay();
            let retry_count = self.reconnect.retry_count();
            info!("Reconnecting in {:?} (attempt {})", delay, retry_count + 1);

            tokio::time::sleep(delay).await;

            match self.connect().await {
                Ok(_) => {
                    info!("Reconnection successful");
                    return Ok(());
                }
                Err(e) => {
                    error!("Reconnection failed: {}", e);
                    self.reconnect.exponential_backoff();
                    self.reconnect.record_retry();
                    self.clear_connection().await;
                }
            }
        }

        Err("Max retries exceeded".to_string())
    }

    /// Clear connection resources
    async fn clear_connection(&self) {
        {
            let mut write_guard = self.ws_write.lock().await;
            *write_guard = None;
        }
        {
            let mut read_guard = self.ws_read.write().await;
            *read_guard = None;
        }
        *self.agent_id.write().await = None;
        *self.session_id.write().await = None;
    }

    /// Disconnect gracefully
    pub async fn disconnect(&mut self) -> Result<(), String> {
        self.set_state(AgentState::ShuttingDown);

        // Send unregister message
        let msg = AgentMessage::Unregister { reason: Some("Client initiated".to_string()) };
        let json = serde_json::to_string(&msg)
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        if let Err(e) = self.send_message(&json).await {
            warn!("Failed to send unregister: {}", e);
        }

        self.clear_connection().await;
        self.set_state(AgentState::Closed);
        info!("Disconnected gracefully");

        Ok(())
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        let state = self.get_state();
        matches!(state, AgentState::Connected | AgentState::Registered | AgentState::PendingApproval)
    }

    /// Get agent ID
    pub async fn agent_id(&self) -> Option<Uuid> {
        *self.agent_id.read().await
    }

    /// Get session ID
    pub async fn session_id(&self) -> Option<String> {
        self.session_id.read().await.clone()
    }

    /// Wait for shutdown signal
    async fn wait_for_shutdown(&self) {
        let tx_guard = self.shutdown_tx.read().await;
        if let Some(tx) = tx_guard.as_ref() {
            let mut rx = tx.subscribe();
            let _ = rx.recv().await;
        }
    }
}

impl AgentState {
    fn from(v: u32) -> Self {
        match v {
            0 => AgentState::Disconnected,
            1 => AgentState::Connecting,
            2 => AgentState::Connected,
            3 => AgentState::Registered,
            4 => AgentState::PendingApproval,
            5 => AgentState::Reconnecting,
            6 => AgentState::ShuttingDown,
            7 => AgentState::Closed,
            _ => AgentState::Disconnected,
        }
    }
}

impl Drop for AgentClient {
    fn drop(&mut self) {
        let state = self.get_state();
        if state != AgentState::Closed && state != AgentState::Disconnected {
            warn!("AgentClient dropped without proper disconnect");
        }
    }
}

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

    /// Approval granted (when requires_approval=true)
    #[serde(rename = "ApprovalGranted")]
    ApprovalGranted {
        server_time: i64,
        message: Option<String>,
    },

    /// Approval denied
    #[serde(rename = "ApprovalDenied")]
    ApprovalDenied {
        reason: String,
    },

    /// System information query from hub
    #[serde(rename = "SystemInfoQuery")]
    SystemInfoQuery {
        #[serde(flatten)]
        query: SystemInfoQuery,
    },

    /// System information report from agent
    #[serde(rename = "SystemInfoReport")]
    SystemInfoReport {
        #[serde(flatten)]
        report: SystemInfoReport,
    },

    /// System information response
    #[serde(rename = "SystemInfoResponse")]
    SystemInfoResponse {
        #[serde(flatten)]
        response: SystemInfoResponse,
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

/// Reconnection manager with exponential backoff
pub struct ReconnectionManager {
    base_delay_ms: u64,
    max_delay_ms: u64,
    current_delay_ms: AtomicU64,
    max_retries: u32,
    retry_count: AtomicU32,
    jitter: f64,
}

impl ReconnectionManager {
    pub fn new() -> Self {
        Self {
            base_delay_ms: 1000,
            max_delay_ms: 300_000,
            current_delay_ms: AtomicU64::new(1000),
            max_retries: 0, // 0 = infinite
            retry_count: AtomicU32::new(0),
            jitter: 0.1,
        }
    }

    /// Get next delay with exponential backoff and jitter
    pub fn get_next_delay(&self) -> Duration {
        let base = self.current_delay_ms.load(Ordering::SeqCst);
        let delay = base.min(self.max_delay_ms);

        // Add jitter: +/- jitter%
        let jitter_range = (delay as f64 * self.jitter) as u64;
        let jittered = if rand::random::<bool>() {
            delay.saturating_sub(jitter_range)
        } else {
            delay.saturating_add(jitter_range)
        };

        Duration::from_millis(jittered)
    }

    /// Apply exponential backoff
    pub fn exponential_backoff(&self) {
        let current = self.current_delay_ms.load(Ordering::SeqCst);
        let new_delay = (current * 2).min(self.max_delay_ms);
        self.current_delay_ms.store(new_delay, Ordering::SeqCst);
    }

    /// Reset delay after successful connection
    pub fn reset(&self) {
        self.current_delay_ms.store(self.base_delay_ms, Ordering::SeqCst);
        self.retry_count.store(0, Ordering::SeqCst);
    }

    /// Check if should continue retrying
    pub fn should_retry(&self) -> bool {
        let retries = self.retry_count.load(Ordering::SeqCst);
        self.max_retries == 0 || retries < self.max_retries
    }

    /// Record a retry attempt
    pub fn record_retry(&self) {
        self.retry_count.fetch_add(1, Ordering::SeqCst);
    }

    /// Get current retry count
    pub fn retry_count(&self) -> u32 {
        self.retry_count.load(Ordering::SeqCst)
    }
}

impl Default for ReconnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reconnection_manager_basic() {
        let reconnect = ReconnectionManager::new();

        // Initial delay should be base
        let delay1 = reconnect.get_next_delay();
        assert!(delay1.as_millis() >= 900 && delay1.as_millis() <= 1100);

        // After backoff, delay should double
        reconnect.exponential_backoff();
        let delay2 = reconnect.get_next_delay();
        assert!(delay2.as_millis() >= 1800 && delay2.as_millis() <= 2200);
    }

    #[test]
    fn test_reconnection_manager_reset() {
        let reconnect = ReconnectionManager::new();

        reconnect.exponential_backoff();
        reconnect.exponential_backoff();
        reconnect.record_retry();
        reconnect.record_retry();

        reconnect.reset();

        let delay = reconnect.get_next_delay();
        assert!(delay.as_millis() >= 900 && delay.as_millis() <= 1100);
        assert_eq!(reconnect.retry_count(), 0);
    }

    #[test]
    fn test_agent_state_from() {
        assert_eq!(AgentState::from(0), AgentState::Disconnected);
        assert_eq!(AgentState::from(3), AgentState::Registered);
        assert_eq!(AgentState::from(99), AgentState::Disconnected);
    }
}
