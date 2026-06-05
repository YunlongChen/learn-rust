//! Agent Management Service
//!
//! This service hosts gRPC and REST APIs for agent management,
//! accepts WebSocket connections from agents, and stores data in PostgreSQL.

pub mod config;

pub mod service;
pub mod storage;
pub mod protocol;
pub mod domain;
pub mod server;

pub mod generated;
