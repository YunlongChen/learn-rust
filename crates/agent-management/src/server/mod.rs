//! Server module for agent management
//!
//! Contains gRPC and REST server implementations.

pub mod grpc;
pub mod rest;

pub use grpc::{create_grpc_server, GrpcServer};
pub use rest::{create_rest_server, RestConfig, AppState};
