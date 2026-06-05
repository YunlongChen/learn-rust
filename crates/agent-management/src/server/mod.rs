//! Server module for agent management
//!
//! Contains gRPC server implementation.

pub mod grpc;

pub use grpc::{create_grpc_server, GrpcServer};
