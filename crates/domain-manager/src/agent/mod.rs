//! Agent 模块
//! 
//! 提供分布式 Agent 管理功能，包括：
//! - Agent 注册与心跳
//! - 任务分发
//! - WebSocket 通信协议

pub mod model;
pub mod protocol;
pub mod registry;
pub mod service;
pub mod connection;
pub mod heartbeat;

