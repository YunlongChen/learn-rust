pub mod client;

pub use client::{AgentManagementClient, Config};

// Re-export generated types from agent-management for convenience
pub use domain_agent_management::generated::agent_management as proto;
