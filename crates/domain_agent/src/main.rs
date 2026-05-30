//! Domain Manager Agent
//!
//! A lightweight agent that connects to Domain Manager Hub

mod client;
mod config;
mod crypto;

use tracing::info;

use crate::client::AgentClient;
use crate::config::AgentConfig;

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

    // Create and connect the agent client
    let mut client = AgentClient::new(config);

    match client.connect().await {
        Ok(_) => {
            info!("Connected to Hub, starting heartbeat loop");
            client.run_heartbeat_loop().await?;
        }
        Err(e) => {
            tracing::error!("Failed to connect to Hub: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
