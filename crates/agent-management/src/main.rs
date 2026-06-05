//! Agent Management Service Entry Point

use agent_management::config::AppConfig;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("Agent Management Service starting...");

    // Load configuration
    let config = AppConfig::load().unwrap_or_else(|e| {
        tracing::warn!("Failed to load config, using defaults: {}", e);
        AppConfig::default()
    });

    info!(
        "Configuration loaded: server={}:{}, grpc={}:{}, rest={}:{}",
        config.server.host,
        config.server.port,
        config.grpc.host,
        config.grpc.port,
        config.rest.host,
        config.rest.port
    );

    // TODO: Initialize Service with config
    // This will be implemented in Task 11
    info!("Service initialization placeholder - full wiring in Task 11");

    // Placeholder to keep the service running
    info!("Agent Management Service initialized successfully");
    info!("Waiting for Task 11 to wire up full service functionality...");

    // For now, just block forever
    tokio::signal::ctrl_c().await?;

    Ok(())
}
