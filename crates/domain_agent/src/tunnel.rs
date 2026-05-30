//! Reverse tunnel support for Domain Agent
//!
//! This module allows the Agent to listen on a port and forward connections
//! to the Hub, enabling the Hub to connect to services behind the Agent's NAT.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::client::AgentClient;

/// Tunnel message types for communication with Hub
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum TunnelMessage {
    /// Agent requests a reverse tunnel
    TunnelRequest {
        tunnel_id: Uuid,
        bind_port: u16,
        tunnel_type: TunnelType,
    },

    /// Hub responds to tunnel request
    TunnelResponse {
        tunnel_id: Uuid,
        success: bool,
        public_port: Option<u16>,
        error: Option<String>,
    },

    /// Data forwarded through tunnel (Hub -> Agent -> Service)
    TunnelData {
        tunnel_id: Uuid,
        data: Vec<u8>,
    },

    /// Data forwarded through tunnel (Service -> Agent -> Hub)
    TunnelDataForward {
        tunnel_id: Uuid,
        data: Vec<u8>,
    },

    /// Close tunnel
    TunnelClose {
        tunnel_id: Uuid,
        reason: Option<String>,
    },
}

/// Tunnel type
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TunnelType {
    /// TCP direct connection
    Tcp,
    /// HTTP proxy
    Http,
    /// WebSocket
    WebSocket,
}

/// A single tunnel connection (Agent side)
struct TunnelConnection {
    id: Uuid,
    listener_port: u16,
    client_stream: Option<TcpStream>,
    tunnel_type: TunnelType,
}

/// Reverse tunnel manager
pub struct TunnelManager {
    /// Active tunnels
    tunnels: Arc<RwLock<HashMap<Uuid, TunnelConnection>>>,
    /// Shutdown signal
    shutdown_tx: Arc<RwLock<Option<broadcast::Sender<()>>>>,
}

impl TunnelManager {
    /// Create a new tunnel manager
    pub fn new() -> Self {
        Self {
            tunnels: Arc::new(RwLock::new(HashMap::new())),
            shutdown_tx: Arc::new(RwLock::new(None)),
        }
    }

    /// Start a reverse tunnel listener
    pub async fn start_tunnel(
        &self,
        agent: &AgentClient,
        port: u16,
        tunnel_type: TunnelType,
    ) -> Result<Uuid, String> {
        let tunnel_id = Uuid::new_v4();

        info!("Starting reverse tunnel {} on port {}", tunnel_id, port);

        // Create TCP listener
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
            .await
            .map_err(|e| format!("Failed to bind port {}: {}", port, e))?;

        info!("Reverse tunnel listener started on port {}", port);

        // Register tunnel
        let tunnel = TunnelConnection {
            id: tunnel_id,
            listener_port: port,
            client_stream: None,
            tunnel_type,
        };

        {
            let mut tunnels = self.tunnels.write().await;
            tunnels.insert(tunnel_id, tunnel);
        }

        // Send tunnel request to Hub
        let request = TunnelMessage::TunnelRequest {
            tunnel_id,
            bind_port: port,
            tunnel_type,
        };

        let json = serde_json::to_string(&request)
            .map_err(|e| format!("Failed to serialize tunnel request: {}", e))?;

        // Note: This requires agent to have a method to send messages
        // For now, we spawn a task to handle connections
        let tunnels = self.tunnels.clone();
        let shutdown_tx = self.shutdown_tx.clone();

        // Spawn task to accept connections and forward them
        tokio::spawn(async move {
            Self::accept_and_forward(listener, tunnel_id, tunnels, shutdown_tx).await;
        });

        Ok(tunnel_id)
    }

    /// Accept connections and forward data
    async fn accept_and_forward(
        listener: TcpListener,
        tunnel_id: Uuid,
        tunnels: Arc<RwLock<HashMap<Uuid, TunnelConnection>>>,
        _shutdown_tx: Arc<RwLock<Option<broadcast::Sender<()>>>>,
    ) {
        info!("Tunnel {}: accepting connections", tunnel_id);

        loop {
            match listener.accept().await {
                Ok((client_stream, addr)) => {
                    info!("Tunnel {}: received connection from {}", tunnel_id, addr);

                    // Handle the connection
                    // In a full implementation, this would:
                    // 1. Send TunnelData message to Hub with initial data
                    // 2. Bridge data between client_stream and WebSocket
                    let tunnel_id_clone = tunnel_id;
                    let client_addr = addr;

                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_tunnel_connection(
                            client_stream,
                            tunnel_id_clone,
                            client_addr,
                        )
                        .await
                        {
                            warn!(
                                "Tunnel {}: connection handling error: {}",
                                tunnel_id_clone, e
                            );
                        }
                    });
                }
                Err(e) => {
                    error!("Tunnel {}: accept error: {}", tunnel_id, e);
                    break;
                }
            }
        }

        // Clean up tunnel
        let mut tunnels = tunnels.write().await;
        tunnels.remove(&tunnel_id);
        info!("Tunnel {}: closed", tunnel_id);
    }

    /// Handle a single tunnel connection
    async fn handle_tunnel_connection(
        mut client_stream: TcpStream,
        tunnel_id: Uuid,
        _client_addr: std::net::SocketAddr,
    ) -> Result<(), String> {
        // Buffer for copying data
        let mut buf = vec![0u8; 8192];

        // For now, just read and log the data
        // In a full implementation, this would:
        // 1. Read from client_stream
        // 2. Send TunnelDataForward to Hub via WebSocket
        // 3. Read TunnelData from Hub
        // 4. Write to client_stream

        loop {
            use tokio::io::AsyncReadExt;

            match client_stream.read(&mut buf).await {
                Ok(0) => {
                    debug!("Tunnel {}: client closed connection", tunnel_id);
                    break;
                }
                Ok(n) => {
                    debug!(
                        "Tunnel {}: received {} bytes from client",
                        tunnel_id, n
                    );
                    // In full implementation: send to Hub
                    let _data = &buf[..n];

                    // Echo back for testing (remove in production)
                    // use tokio::io::AsyncWriteExt;
                    // client_stream.write_all(&buf[..n]).await?;
                }
                Err(e) => {
                    error!("Tunnel {}: read error: {}", tunnel_id, e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Stop a tunnel
    pub async fn stop_tunnel(&self, tunnel_id: Uuid) -> Result<(), String> {
        let mut tunnels = self.tunnels.write().await;

        if tunnels.remove(&tunnel_id).is_some() {
            info!("Tunnel {}: stopped", tunnel_id);
            Ok(())
        } else {
            Err(format!("Tunnel {} not found", tunnel_id))
        }
    }

    /// Stop all tunnels
    pub async fn stop_all(&self) {
        let mut tunnels = self.tunnels.write().await;
        for (id, _) in tunnels.drain() {
            info!("Tunnel {}: stopped", id);
        }
    }

    /// Get active tunnel count
    pub async fn tunnel_count(&self) -> usize {
        let tunnels = self.tunnels.read().await;
        tunnels.len()
    }
}

impl Default for TunnelManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tunnel_message_serialization() {
        let msg = TunnelMessage::TunnelRequest {
            tunnel_id: Uuid::new_v4(),
            bind_port: 8081,
            tunnel_type: TunnelType::Tcp,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"TunnelRequest\""));
        assert!(json.contains("\"bind_port\":8081"));
    }

    #[tokio::test]
    async fn test_tunnel_manager() {
        let manager = TunnelManager::new();
        assert_eq!(manager.tunnel_count().await, 0);
    }
}
