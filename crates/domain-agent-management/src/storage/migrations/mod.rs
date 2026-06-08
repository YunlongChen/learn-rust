//! Storage migrations
//!
//! Database migrations for the agent management storage layer.

pub use sea_orm_migration::prelude::*;

pub mod m20250604_000001_create_agents_table;
pub mod m20250604_000002_create_lifecycle_events_table;
pub mod m20250604_000003_create_system_info_table;
pub mod m20250604_000004_create_health_scores_table;

use m20250604_000001_create_agents_table::Migration as CreateAgentsTable;
use m20250604_000002_create_lifecycle_events_table::Migration as CreateLifecycleEventsTable;
use m20250604_000003_create_system_info_table::Migration as CreateSystemInfoTable;
use m20250604_000004_create_health_scores_table::Migration as CreateHealthScoresTable;

/// All migrations to be run
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(CreateAgentsTable),
            Box::new(CreateLifecycleEventsTable),
            Box::new(CreateSystemInfoTable),
            Box::new(CreateHealthScoresTable),
        ]
    }
}
