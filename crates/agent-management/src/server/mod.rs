//! Server module for agent management
//!
//! Contains gRPC, REST, and WebSocket server implementations.

pub mod grpc;
pub mod rest;
pub mod websocket;

pub use grpc::{create_grpc_server, GrpcServer};
pub use rest::{create_rest_server, RestConfig, AppState};
pub use websocket::WebSocketServer;
