//! P2P connection support for Domain Agent
//!
//! This module handles NAT traversal and peer-to-peer connections between agents.
//! Uses a combination of STUN-like techniques and Hub relay for signaling.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// P2P message types for signaling
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum P2pMessage {
    /// Request P2P connection with another agent
    P2pConnectRequest {
        request_id: Uuid,
        target_agent_id: Uuid,
    },

    /// Offer for P2P connection (contains SDP)
    P2pConnectOffer {
        request_id: Uuid,
        source_agent_id: Uuid,
        sdp_offer: String,
    },

    /// Answer for P2P connection (contains SDP)
    P2pAnswer {
        request_id: Uuid,
        target_agent_id: Uuid,
        sdp_answer: String,
    },

    /// ICE candidate for NAT traversal
    P2pIceCandidate {
        request_id: Uuid,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<u16>,
    },

    /// P2P connection established
    P2pConnected {
        request_id: Uuid,
    },

    /// P2P connection failed
    P2pFailed {
        request_id: Uuid,
        reason: String,
    },
}

/// P2P connection state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum P2pState {
    /// Idle - no connection attempt
    Idle,
    /// Connecting - initiating connection
    Connecting,
    /// Received offer - waiting for answer
    ReceivedOffer,
    /// Sent offer - waiting for answer
    SentOffer,
    /// Connected - P2P connection established
    Connected,
    /// Failed - connection failed
    Failed(String),
}

/// A P2P connection
pub struct P2pConnection {
    pub id: Uuid,
    pub peer_agent_id: Uuid,
    pub state: P2pState,
    pub direct_socket: Option<TcpStream>,
    pub created_at: std::time::Instant,
}

/// NAT type detection result
#[derive(Debug, Clone)]
pub enum NatType {
    /// Full cone NAT - any external host can send to us
    FullCone,
    /// Restricted cone NAT - only to where we sent
    RestrictedCone,
    /// Port restricted cone NAT - more restrictive
    PortRestrictedCone,
    /// Symmetric NAT - each destination gets different port
    Symmetric,
    /// No NAT - direct connection possible
    NoNat,
}

/// External address info
#[derive(Debug, Clone)]
pub struct ExternalAddress {
    pub ip: String,
    pub port: u16,
}

/// P2P connection manager
pub struct P2pManager {
    /// Active P2P connections
    connections: Arc<RwLock<HashMap<Uuid, P2pConnection>>>,
    /// External address (discovered via STUN-like query)
    external_addr: Arc<RwLock<Option<ExternalAddress>>>,
    /// NAT type
    nat_type: Arc<RwLock<Option<NatType>>>,
    /// Local listening port for P2P
    listen_port: u16,
    /// Shutdown signal
    shutdown_tx: Arc<RwLock<Option<broadcast::Sender<()>>>>,
}

impl P2pManager {
    /// Create a new P2P manager
    pub fn new(listen_port: u16) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            external_addr: Arc::new(RwLock::new(None)),
            nat_type: Arc::new(RwLock::new(None)),
            listen_port,
            shutdown_tx: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize P2P manager - discover external address and NAT type
    pub async fn initialize(&self) -> Result<(), String> {
        info!("Initializing P2P manager on port {}", self.listen_port);

        // Try to discover external address
        match self.discover_external_address().await {
            Ok(addr) => {
                info!("Discovered external address: {}:{}", addr.ip, addr.port);
                *self.external_addr.write().await = Some(addr);
            }
            Err(e) => {
                warn!("Failed to discover external address: {}", e);
            }
        }

        // Try to detect NAT type
        match self.detect_nat_type().await {
            Ok(nat) => {
                info!("Detected NAT type: {:?}", nat);
                *self.nat_type.write().await = Some(nat);
            }
            Err(e) => {
                warn!("Failed to detect NAT type: {}", e);
            }
        }

        Ok(())
    }

    /// Discover external address using a simple STUN-like query
    async fn discover_external_address(&self) -> Result<ExternalAddress, String> {
        // Note: In production, use a real STUN server
        // For now, we use a simple UDP socket to get local address
        // and assume it's the external address (works for no-NAT scenarios)

        let socket = UdpSocket::bind(format!("0.0.0.0:{}", self.listen_port))
            .await
            .map_err(|e| format!("Failed to bind UDP socket: {}", e))?;

        // Connect to a public server to discover our external address
        // Using a simple echo/whoami service
        let (ip, port) = match socket.peer_addr() {
            Ok(peer) => (peer.ip().to_string(), peer.port()),
            Err(_) => {
                // Fallback: use local address
                let local = socket.local_addr()
                    .map_err(|e| format!("Failed to get local addr: {}", e))?;
                (local.ip().to_string(), local.port())
            }
        };

        Ok(ExternalAddress { ip, port })
    }

    /// Detect NAT type using simple tests
    async fn detect_nat_type(&self) -> Result<NatType, String> {
        // Simplified NAT detection
        // In production, use multiple STUN servers and perform proper tests

        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| format!("Failed to create UDP socket: {}", e))?;

        // Try to connect to itself - if it works, no NAT
        if socket.connect(socket.local_addr().unwrap()).await.is_ok() {
            return Ok(NatType::NoNat);
        }

        // Simplified: assume symmetric NAT
        Ok(NatType::Symmetric)
    }

    /// Request a P2P connection with another agent
    pub async fn request_connection(
        &self,
        target_agent_id: Uuid,
    ) -> Result<Uuid, String> {
        let request_id = Uuid::new_v4();

        info!(
            "Requesting P2P connection with agent {} (request_id: {})",
            target_agent_id, request_id
        );

        let connection = P2pConnection {
            id: request_id,
            peer_agent_id: target_agent_id,
            state: P2pState::Connecting,
            direct_socket: None,
            created_at: std::time::Instant::now(),
        };

        let mut connections = self.connections.write().await;
        connections.insert(request_id, connection);

        Ok(request_id)
    }

    /// Handle incoming P2P offer
    pub async fn handle_offer(
        &self,
        request_id: Uuid,
        source_agent_id: Uuid,
        sdp_offer: String,
    ) -> Result<(), String> {
        debug!(
            "Received P2P offer from {} for request {}",
            source_agent_id, request_id
        );

        let mut connections = self.connections.write().await;

        if let Some(conn) = connections.get_mut(&request_id) {
            conn.state = P2pState::ReceivedOffer;
            conn.peer_agent_id = source_agent_id;
            debug!("P2P connection {} state updated to ReceivedOffer", request_id);
        } else {
            // New incoming connection
            let connection = P2pConnection {
                id: request_id,
                peer_agent_id: source_agent_id,
                state: P2pState::ReceivedOffer,
                direct_socket: None,
                created_at: std::time::Instant::now(),
            };
            connections.insert(request_id, connection);
        }

        Ok(())
    }

    /// Handle P2P answer
    pub async fn handle_answer(
        &self,
        request_id: Uuid,
        _sdp_answer: String,
    ) -> Result<(), String> {
        debug!("Received P2P answer for request {}", request_id);

        let mut connections = self.connections.write().await;

        if let Some(conn) = connections.get_mut(&request_id) {
            conn.state = P2pState::Connected;
            info!("P2P connection {} established!", request_id);
            Ok(())
        } else {
            Err(format!("Unknown P2P request: {}", request_id))
        }
    }

    /// Handle ICE candidate
    pub async fn handle_ice_candidate(
        &self,
        request_id: Uuid,
        candidate: String,
    ) -> Result<(), String> {
        debug!(
            "Received ICE candidate for request {}: {}",
            request_id, candidate
        );

        // In a full implementation, try to connect using the candidate
        let mut connections = self.connections.write().await;

        if let Some(conn) = connections.get_mut(&request_id) {
            // Try to establish direct connection using candidate
            if let Err(e) = self.try_direct_connection(request_id, &candidate).await {
                debug!(
                    "Failed to connect using ICE candidate for {}: {}",
                    request_id, e
                );
            }
            Ok(())
        } else {
            Err(format!("Unknown P2P request: {}", request_id))
        }
    }

    /// Try to establish direct connection using ICE candidate
    async fn try_direct_connection(
        &self,
        request_id: Uuid,
        candidate: &str,
    ) -> Result<(), String> {
        // Parse candidate (simplified: "IP:PORT" format)
        // Full ICE candidates are more complex (priority, foundation, component, etc.)
        let parts: Vec<&str> = candidate.split(':').collect();
        if parts.len() != 2 {
            return Err("Invalid candidate format".to_string());
        }

        let ip = parts[0].to_string();
        let port: u16 = parts[1]
            .parse()
            .map_err(|_| "Invalid port in candidate")?;

        info!(
            "Attempting direct connection to {}:{} for P2P {}",
            ip, port, request_id
        );

        let addr = format!("{}:{}", ip, port);
        match TcpStream::connect(&addr).await {
            Ok(stream) => {
                info!("Direct connection established for P2P {}", request_id);
                let mut connections = self.connections.write().await;
                if let Some(conn) = connections.get_mut(&request_id) {
                    conn.direct_socket = Some(stream);
                    conn.state = P2pState::Connected;
                }
                Ok(())
            }
            Err(e) => Err(format!("Connection failed: {}", e)),
        }
    }

    /// Notify connection failure
    pub async fn notify_failed(&self, request_id: Uuid, reason: String) {
        error!("P2P connection {} failed: {}", request_id, reason);
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(&request_id) {
            conn.state = P2pState::Failed(reason.clone());
        }
    }

    /// Get connection state
    pub async fn get_connection_state(&self, request_id: Uuid) -> Option<P2pState> {
        let connections = self.connections.read().await;
        connections.get(&request_id).map(|c| c.state.clone())
    }

    /// Get external address
    pub async fn get_external_address(&self) -> Option<ExternalAddress> {
        self.external_addr.read().await.clone()
    }

    /// Get NAT type
    pub async fn get_nat_type(&self) -> Option<NatType> {
        self.nat_type.read().await.clone()
    }

    /// Close a P2P connection
    pub async fn close_connection(&self, request_id: Uuid) -> Result<(), String> {
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.remove(&request_id) {
            if let Some(stream) = conn.direct_socket {
                drop(stream);
            }
            info!("P2P connection {} closed", request_id);
            Ok(())
        } else {
            Err(format!("Connection {} not found", request_id))
        }
    }

    /// Close all connections
    pub async fn close_all(&self) {
        let mut connections = self.connections.write().await;
        for (id, conn) in connections.drain() {
            if let Some(stream) = conn.direct_socket {
                drop(stream);
            }
            info!("P2P connection {} closed", id);
        }
    }

    /// Get active connection count
    pub async fn connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p2p_message_serialization() {
        let msg = P2pMessage::P2pConnectRequest {
            request_id: Uuid::new_v4(),
            target_agent_id: Uuid::new_v4(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"P2pConnectRequest\""));
    }

    #[tokio::test]
    async fn test_p2p_manager() {
        let manager = P2pManager::new(0);
        assert_eq!(manager.connection_count().await, 0);
    }
}
