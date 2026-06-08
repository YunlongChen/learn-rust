//! Agent Protocol - Shared protocol definitions for lifecycle events and diagnostic data.
//!
//! This crate provides shared types used by both agent-management service and domain-agent.

pub mod diagnostic;
pub mod lifecycle;

pub use diagnostic::*;
pub use lifecycle::*;
