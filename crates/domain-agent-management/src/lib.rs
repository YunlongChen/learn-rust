//! Agent Management Service
//!
//! A Rust-based microservice for managing agents in a distributed system.
//!
//! ## Features
//!
//! - **Multi-Protocol Support**: gRPC (port 50051), REST (port 8080), and WebSocket (port 8081)
//! - **Agent Lifecycle Management**: State machine-based lifecycle tracking
//! - **Health Monitoring**: Network health scoring based on latency, jitter, packet loss, and bandwidth
//! - **System Diagnostics**: Collect and query system information from agents
//! - **Approval Workflow**: Pending/Approved/Denied agent registration workflow
//! - **PostgreSQL Storage**: Persistent storage with SeaORM
//!
//! ## Architecture
//!
//! The service follows a layered architecture:
//!
//! - **Transport Layer**: gRPC (tonic), REST (axum), WebSocket (tungstenite)
//! - **Service Layer**: Business logic services (AgentService, LifecycleService, HealthService, DiagnosticService)
//! - **Domain Layer**: State machine for agent lifecycle management
//! - **Storage Layer**: PostgreSQL via SeaORM
//!
//! ## Usage
//!
//! ```rust,ignore
//! use agent_management::config::AppConfig;
//! use agent_management::service::Service;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = AppConfig::load()?;
//!     let service = Service::new(config).await?;
//!     service.run().await
//! }
//! ```
//!
//! ## Documentation
//!
//! - [README](https://github.com/your-org/agent-management/blob/main/README.md) - Overview and quick start
//! - [ARCHITECTURE](ARCHITECTURE.md) - Internal architecture details
//! - [API](API.md) - Detailed API reference
//!
//! ## Crates
//!
//! This crate provides the `agent-management` binary and library.

pub mod config;

pub mod service;
pub mod storage;
pub mod protocol;
pub mod domain;
pub mod server;

pub mod generated;

pub mod web_config;