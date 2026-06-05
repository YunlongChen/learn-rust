//! Storage entities
//!
//! Sea-orm entity models for agent management.

pub mod agent;
pub mod health_score;
pub mod lifecycle_event;
pub mod system_info;

// Re-export the Entity types from each module
pub use agent::Entity as AgentEntity;
pub use health_score::Entity as HealthScoreEntity;
pub use lifecycle_event::Entity as LifecycleEventEntity;
pub use system_info::Entity as SystemInfoEntity;
