//! Service module for agent management
//!
//! Contains business logic services including lifecycle event management.

pub mod health;
pub mod lifecycle;

pub use health::{HealthService, NetworkHealthMetrics};
pub use lifecycle::LifecycleService;
