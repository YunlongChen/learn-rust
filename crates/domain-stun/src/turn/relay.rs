//! TURN data relay

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct RelayHandler {
    peers: Arc<RwLock<HashMap<SocketAddr, SocketAddr>>>,
}

impl RelayHandler {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn relay_data(
        &self,
        from: SocketAddr,
        to: SocketAddr,
        data: &[u8],
    ) -> Result<(), String> {
        tracing::debug!("Relaying {} bytes from {} to {}", data.len(), from, to);
        Ok(())
    }

    pub async fn add_peer(&self, relayed: SocketAddr, peer: SocketAddr) {
        self.peers.write().await.insert(relayed, peer);
    }

    pub async fn remove_peer(&self, relayed: &SocketAddr) {
        self.peers.write().await.remove(relayed);
    }
}

impl Default for RelayHandler {
    fn default() -> Self {
        Self::new()
    }
}
