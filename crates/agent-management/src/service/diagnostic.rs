//! Diagnostic service for system information collection
//!
//! This module provides the DiagnosticService for collecting system diagnostics.

use sea_orm::ActiveModelTrait;
use sea_orm::entity::prelude::*;
use sea_orm::{Set, QueryOrder, EntityTrait};
use uuid::Uuid;

use agent_protocol::diagnostic::SystemInfoReport;
use crate::storage::Database;
use crate::storage::entities::system_info::{Entity as SystemInfoEntity, ActiveModel, Model};

/// Service for collecting diagnostic information from agents.
///
/// This service provides methods to query system information, resource usage,
/// and other diagnostic data from managed agents.
#[derive(Clone)]
pub struct DiagnosticService {
    db: Database,
}

impl DiagnosticService {
    /// Creates a new DiagnosticService with the given database.
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Store system info report for an agent.
    ///
    /// Stores all info types (os, environment, process, network, resources) in a single record.
    pub async fn store_system_info(
        &self,
        agent_id: Uuid,
        report: SystemInfoReport,
    ) -> Result<Model, sea_orm::DbErr> {
        let os_json = report.os.map(|o| serde_json::to_value(o).unwrap_or_default());
        let env_json = report.environment.map(|e| serde_json::to_value(e).unwrap_or_default());
        let process_json = report.process.map(|p| serde_json::to_value(p).unwrap_or_default());
        let network_json = report.network.map(|n| serde_json::to_value(n).unwrap_or_default());
        let resource_json = report.resources.map(|r| serde_json::to_value(r).unwrap_or_default());

        let active_model = ActiveModel {
            id: Set(report.report_id),
            agent_id: Set(agent_id),
            reported_at: Set(report.timestamp),
            info_type: Set("system_info".to_string()),
            os_info: Set(os_json),
            environment_info: Set(env_json),
            process_info: Set(process_json),
            network_info: Set(network_json),
            resource_info: Set(resource_json),
        };

        let result = active_model.insert(self.db.get_conn()).await?;
        Ok(result)
    }

    /// Get the latest system info for an agent.
    ///
    /// Returns the most recent system info record ordered by reported_at descending.
    pub async fn get_system_info(&self, agent_id: Uuid) -> Result<Option<Model>, sea_orm::DbErr> {
        use crate::storage::entities::system_info::Column;

        let result = SystemInfoEntity::find()
            .filter(Column::AgentId.eq(agent_id))
            .order_by_desc(Column::ReportedAt)
            .one(self.db.get_conn())
            .await?;

        Ok(result)
    }
}

impl Default for DiagnosticService {
    fn default() -> Self {
        panic!("DiagnosticService::default() is not supported, use DiagnosticService::new(db)")
    }
}
