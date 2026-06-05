//! Domain Manager Agent
//!
//! A lightweight agent that connects to Domain Manager Hub
//!
//! Features:
//! - WebSocket connection to Hub with auto-reconnect
//! - SOCKS5 and HTTP proxy support
//! - Reverse tunnel for inbound connections
//! - P2P connectivity between agents

mod client;
mod config;
mod crypto;
mod diagnostic;
mod proxy;
mod p2p;
mod protocol;
mod tunnel;

use tracing::{error, info, warn};

use crate::client::AgentClient;
use crate::config::AgentConfig;
use crate::p2p::P2pManager;
use crate::tunnel::{TunnelManager, TunnelType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Parse command line arguments
    let config = AgentConfig::from_args();

    info!(
        "Starting Domain Agent: name={}, hub={}",
        config.name, config.hub
    );

    // Create managers
    let tunnel_manager = TunnelManager::new();
    let p2p_manager = P2pManager::new(config.p2p_port);

    // Initialize P2P (discover external address, NAT type)
    if let Err(e) = p2p_manager.initialize().await {
        warn!("P2P initialization failed (continuing anyway): {}", e);
    } else if let Some(addr) = p2p_manager.get_external_address().await {
        info!("P2P external address: {}:{}", addr.ip, addr.port);
    }

    // Create agent client
    let mut client = AgentClient::new(config.clone());

    // Connect to Hub
    match client.connect().await {
        Ok(_) => {
            info!("Connected to Hub successfully");
        }
        Err(e) => {
            error!("Failed to connect to Hub: {}", e);
            std::process::exit(1);
        }
    }

    // Start reverse tunnel if configured
    if config.tunnel_port > 0 {
        match tunnel_manager
            .start_tunnel(&client, config.tunnel_port, TunnelType::Tcp)
            .await
        {
            Ok(tunnel_id) => {
                info!(
                    "Reverse tunnel {} started on port {}",
                    tunnel_id, config.tunnel_port
                );
            }
            Err(e) => {
                error!("Failed to start tunnel: {}", e);
            }
        }
    }

    // Run main loop (heartbeat + message handling)
    // This will block until disconnect or error
    if let Err(e) = client.run().await {
        error!("Agent run error: {}", e);
    }

    // Cleanup
    tunnel_manager.stop_all().await;
    p2p_manager.close_all().await;

    // Graceful disconnect
    let _ = client.disconnect().await;

    info!("Agent shutdown complete");
    Ok(())
}
