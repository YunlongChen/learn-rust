//! Domain module for agent management
//!
//! Contains core business logic including the lifecycle state machine.

pub mod state_machine;

pub use state_machine::{AgentLifecycleState, LifecycleStateMachine};