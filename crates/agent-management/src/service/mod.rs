//! Service module for agent management
//!
//! Contains business logic services including lifecycle event management,
//! health scoring, agent management, and diagnostics.

pub mod agent;
pub mod diagnostic;
pub mod health;
pub mod lifecycle;

pub use agent::{AgentService, AgentInfo};
pub use diagnostic::DiagnosticService;
pub use health::{HealthService, NetworkHealthMetrics};
pub use lifecycle::LifecycleService;

use crate::config::AppConfig;
use crate::server::grpc::create_grpc_server;
use crate::server::rest::{create_rest_server, AppState, RestConfig};
use crate::storage::Database;
use anyhow::Result;
use tonic::transport::Server as TonicServer;
use tracing::{info, error};

/// Unified service containing all agent management services
#[derive(Clone, Debug)]
pub struct Service {
    pub agent_service: AgentService,
    pub lifecycle_service: LifecycleService,
    pub health_service: HealthService,
    pub diagnostic_service: DiagnosticService,
    pub database: Database,
    pub config: AppConfig,
}

impl Service {
    /// Creates a new Service instance with all subsystems initialized
    pub async fn new(config: AppConfig) -> Result<Self> {
        info!("Initializing Agent Management Service...");

        // Initialize database
        let database = Database::new(&config.database.url).await?;
        database.run_migrations().await?;
        info!("Database initialized and migrations completed");

        // Initialize services
        let agent_service = AgentService::new(database.clone());
        let lifecycle_service = LifecycleService::new(database.clone());
        let health_service = HealthService::new(database.get_conn().clone());
        let diagnostic_service = DiagnosticService::new(database.clone());

        info!("All services initialized successfully");

        Ok(Self {
            agent_service,
            lifecycle_service,
            health_service,
            diagnostic_service,
            database,
            config,
        })
    }

    /// Runs all servers (gRPC, REST, WebSocket) concurrently
    pub async fn run(self) -> Result<()> {
        let config = self.config.clone();

        // Build gRPC address
        let grpc_addr = format!("{}:{}", config.grpc.host, config.grpc.port);
        let grpc_addr_parse = grpc_addr.parse()?;
        info!("gRPC server listening on {}", grpc_addr);

        // Build REST address
        let rest_addr = format!("{}:{}", config.rest.host, config.rest.port);
        info!("REST server listening on {}", rest_addr);

        // WebSocket port (same host as server)
        let ws_port = config.server.ws_port;
        info!("WebSocket server listening on port {}", ws_port);

        // Create REST app state
        let app_state = AppState { service: self.clone() };

        // Spawn gRPC server
        let grpc_handle = tokio::spawn(async move {
            let grpc_server = create_grpc_server(&grpc_addr, self.clone()).await
                .expect("Failed to create gRPC server");

            TonicServer::builder()
                .add_service(grpc_server)
                .serve(grpc_addr_parse)
                .await
                .expect("gRPC server failed");
        });

        // Spawn REST server
        let rest_handle = tokio::spawn(async move {
            let rest_config = RestConfig {
                addr: rest_addr.clone(),
            };

            let app = create_rest_server(rest_config, app_state)
                .await
                .expect("Failed to create REST server");

            let listener = tokio::net::TcpListener::bind(&rest_addr)
                .await
                .expect("Failed to bind REST server");

            axum::serve(listener, app)
                .await
                .expect("REST server failed");
        });

        // Spawn WebSocket server (stubbed for now)
        let ws_handle = tokio::spawn(async move {
            let ws_addr = format!("{}:{}", config.server.host, ws_port);
            info!("WebSocket server stub listening on {}", ws_addr);

            // Stub: just log and keep running
            tokio::signal::ctrl_c().await.ok();
            info!("WebSocket server shutting down");
        });

        // Wait for all servers
        info!("All servers started, waiting for shutdown signal...");

        // Wait for any server to fail or for ctrl-c
        tokio::select! {
            result = grpc_handle => {
                if let Err(e) = result {
                    error!("gRPC server task panicked: {}", e);
                }
            }
            result = rest_handle => {
                if let Err(e) = result {
                    error!("REST server task panicked: {}", e);
                }
            }
            result = ws_handle => {
                if let Err(e) = result {
                    error!("WebSocket server task panicked: {}", e);
                }
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Shutdown signal received");
            }
        }

        info!("Agent Management Service shutting down");
        Ok(())
    }
}