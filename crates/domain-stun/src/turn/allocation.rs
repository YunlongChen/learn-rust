//! TURN allocation management

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::stun::{StunMessage, StunMessageType, StunAttribute, make_error_response};

#[derive(Debug, Clone)]
pub struct TurnAllocation {
    pub id: Uuid,
    pub client_addr: SocketAddr,
    pub relayed_addr: SocketAddr,
    pub mapped_addr: SocketAddr,
    pub channel: u16,
    pub lifetime: u32,
    pub created_at: std::time::Instant,
    pub last_activity: std::time::Instant,
}

impl TurnAllocation {
    pub fn new(client_addr: SocketAddr, lifetime: u32) -> Self {
        let port = Self::next_relay_port();
        Self {
            id: Uuid::new_v4(),
            client_addr,
            relayed_addr: SocketAddr::from((Ipv4Addr::new(0, 0, 0, 0), port)),
            mapped_addr: client_addr,
            channel: 0x4000 | (port & 0x3FFF),
            lifetime,
            created_at: std::time::Instant::now(),
            last_activity: std::time::Instant::now(),
        }
    }

    fn next_relay_port() -> u16 {
        use std::sync::atomic::{AtomicU16, Ordering};
        static NEXT_PORT: AtomicU16 = AtomicU16::new(49152);
        NEXT_PORT.fetch_add(1, Ordering::Relaxed)
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed().as_secs() > self.lifetime as u64
    }

    pub fn refresh(&mut self, lifetime: u32) {
        self.lifetime = lifetime;
        self.last_activity = std::time::Instant::now();
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AllocationInfo {
    pub id: String,
    pub client_addr: String,
    pub relayed_addr: String,
    pub lifetime: u32,
    pub created_at: String,
}

pub struct TurnHandler {
    allocations: Arc<RwLock<HashMap<Uuid, TurnAllocation>>>,
    max_allocations: usize,
    default_lifetime: u32,
}

impl TurnHandler {
    pub fn new() -> Self {
        Self {
            allocations: Arc::new(RwLock::new(HashMap::new())),
            max_allocations: 1000,
            default_lifetime: 600,
        }
    }

    pub async fn handle_allocate_request(
        &self,
        msg: &StunMessage,
        from: &SocketAddr,
    ) -> Option<Vec<u8>> {
        let lifetime = self.default_lifetime;

        if self.allocations.read().await.len() >= self.max_allocations {
            tracing::warn!("Max allocations reached, rejecting request from {}", from);
            return Some(make_error_response(msg, 486, "Allocation Quota Reached"));
        }

        let allocation = TurnAllocation::new(*from, lifetime);

        let relayed_addr = allocation.relayed_addr;
        let id = allocation.id;

        self.allocations.write().await.insert(id, allocation);

        tracing::info!("Created TURN allocation for {} -> {}", from, relayed_addr);

        let response = self.make_allocate_response(msg, relayed_addr, lifetime);
        Some(response)
    }

    pub async fn handle_refresh_request(
        &self,
        msg: &StunMessage,
        from: &SocketAddr,
    ) -> Option<Vec<u8>> {
        let mut allocations = self.allocations.write().await;

        if let Some(allocation) = allocations.values_mut().find(|a| a.client_addr == *from) {
            allocation.refresh(self.default_lifetime);
            tracing::debug!("Refreshed allocation for {}", from);

            let response = self.make_refresh_response(msg, self.default_lifetime);
            Some(response)
        } else {
            Some(make_error_response(msg, 400, "No Allocation Found"))
        }
    }

    pub async fn handle_channel_bind(
        &self,
        msg: &StunMessage,
        _from: &SocketAddr,
    ) -> Option<Vec<u8>> {
        Some(make_error_response(msg, 400, "Not Implemented"))
    }

    pub async fn get_allocations(&self) -> Vec<AllocationInfo> {
        let allocations = self.allocations.read().await;
        allocations
            .values()
            .map(|a| AllocationInfo {
                id: a.id.to_string(),
                client_addr: a.client_addr.to_string(),
                relayed_addr: a.relayed_addr.to_string(),
                lifetime: a.lifetime,
                created_at: format!("{:?}", a.created_at),
            })
            .collect()
    }

    fn make_allocate_response(
        &self,
        msg: &StunMessage,
        relayed_addr: SocketAddr,
        lifetime: u32,
    ) -> Vec<u8> {
        let mut addr_bytes = vec![0u8; 8];
        addr_bytes[0] = 0x00;
        addr_bytes[1] = 0x01;
        addr_bytes[2..4].copy_from_slice(&relayed_addr.port().to_be_bytes());

        let ip_octets = match relayed_addr.ip() {
            IpAddr::V4(ipv4) => ipv4.octets(),
            IpAddr::V6(_) => [0, 0, 0, 0],
        };
        addr_bytes[4..8].copy_from_slice(&ip_octets);

        let mut lifetime_bytes = vec![0u8; 4];
        lifetime_bytes.copy_from_slice(&lifetime.to_be_bytes());

        let response = StunMessage {
            message_type: StunMessageType::AllocateResponse,
            transaction_id: msg.transaction_id,
            attributes: vec![
                StunAttribute::new(0x0016, addr_bytes),
                StunAttribute::new(0x000D, lifetime_bytes),
            ],
        };

        response.to_bytes()
    }

    fn make_refresh_response(&self, msg: &StunMessage, lifetime: u32) -> Vec<u8> {
        let mut lifetime_bytes = vec![0u8; 4];
        lifetime_bytes.copy_from_slice(&lifetime.to_be_bytes());

        let response = StunMessage {
            message_type: StunMessageType::RefreshResponse,
            transaction_id: msg.transaction_id,
            attributes: vec![
                StunAttribute::new(0x000D, lifetime_bytes),
            ],
        };

        response.to_bytes()
    }
}

impl Default for TurnHandler {
    fn default() -> Self {
        Self::new()
    }
}
