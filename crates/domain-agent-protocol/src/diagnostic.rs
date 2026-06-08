//! Diagnostic and system information data structures.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Operating system information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OsInfo {
    pub os_type: String,
    pub distro: String,
    pub kernel: String,
    pub architecture: String,
    pub hostname: String,
}

/// Environment information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    pub env_vars: HashMap<String, String>,
    pub current_working_dir: String,
    pub user: String,
    pub uid_gid: Option<String>,
}

/// Process information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub command: String,
    pub start_time: Option<DateTime<Utc>>,
    pub uptime_seconds: Option<u64>,
}

/// A network interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub ip: String,
    pub mac: Option<String>,
}

/// A network connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub proto: String,
    pub local: String,
    pub remote: String,
    pub state: String,
}

/// Network information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub interfaces: Vec<NetworkInterface>,
    pub connections: Vec<NetworkConnection>,
}

/// CPU information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub model: String,
    pub cores: usize,
    pub speed_mhz: Option<f64>,
    pub usage_percent: Option<f64>,
}

/// Memory information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: Option<f64>,
}

/// Disk information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub mount_point: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: Option<f64>,
}

/// Resource information combining CPU, memory, and disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub disks: Vec<DiskInfo>,
}

/// Query for system information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfoQuery {
    pub query_id: Uuid,
    pub agent_id: Uuid,
    pub include_env: bool,
    pub include_process: bool,
    pub include_network: bool,
    pub include_resources: bool,
    pub timestamp: DateTime<Utc>,
}

impl SystemInfoQuery {
    /// Creates a new query with default settings (all info requested).
    pub fn new(agent_id: Uuid) -> Self {
        Self {
            query_id: Uuid::new_v4(),
            agent_id,
            include_env: true,
            include_process: true,
            include_network: true,
            include_resources: true,
            timestamp: Utc::now(),
        }
    }

    /// Creates a minimal query with only basic info.
    pub fn minimal(agent_id: Uuid) -> Self {
        Self {
            query_id: Uuid::new_v4(),
            agent_id,
            include_env: false,
            include_process: false,
            include_network: false,
            include_resources: false,
            timestamp: Utc::now(),
        }
    }
}

/// System information report containing diagnostic data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfoReport {
    pub report_id: Uuid,
    pub agent_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub os: Option<OsInfo>,
    pub environment: Option<EnvironmentInfo>,
    pub process: Option<ProcessInfo>,
    pub network: Option<NetworkInfo>,
    pub resources: Option<ResourceInfo>,
}

impl SystemInfoReport {
    /// Creates a new empty report for the given agent.
    pub fn new(agent_id: Uuid) -> Self {
        Self {
            report_id: Uuid::new_v4(),
            agent_id,
            timestamp: Utc::now(),
            os: None,
            environment: None,
            process: None,
            network: None,
            resources: None,
        }
    }

    /// Sets the OS information.
    pub fn with_os(mut self, os: OsInfo) -> Self {
        self.os = Some(os);
        self
    }

    /// Sets the environment information.
    pub fn with_environment(mut self, env: EnvironmentInfo) -> Self {
        self.environment = Some(env);
        self
    }

    /// Sets the process information.
    pub fn with_process(mut self, process: ProcessInfo) -> Self {
        self.process = Some(process);
        self
    }

    /// Sets the network information.
    pub fn with_network(mut self, network: NetworkInfo) -> Self {
        self.network = Some(network);
        self
    }

    /// Sets the resource information.
    pub fn with_resources(mut self, resources: ResourceInfo) -> Self {
        self.resources = Some(resources);
        self
    }
}

/// Response containing system information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfoResponse {
    pub query_id: Uuid,
    pub report: SystemInfoReport,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl SystemInfoResponse {
    /// Creates a successful response.
    pub fn success(query_id: Uuid, report: SystemInfoReport) -> Self {
        Self {
            query_id,
            report,
            success: true,
            error: None,
        }
    }

    /// Creates an error response.
    pub fn error(query_id: Uuid, error: impl Into<String>) -> Self {
        Self {
            query_id,
            report: SystemInfoReport::new(Uuid::nil()),
            success: false,
            error: Some(error.into()),
        }
    }
}

/// Health metrics for an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub agent_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub uptime_seconds: u64,
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub network_connected: bool,
    pub last_event: Option<LifecycleEventType>,
}

/// Health score for an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthScore {
    pub agent_id: Uuid,
    pub score: f64,
    pub grade: HealthGrade,
    pub factors: Vec<HealthFactor>,
    pub timestamp: DateTime<Utc>,
}

/// Health grade levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthGrade {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

impl HealthGrade {
    /// Determines the grade based on a numeric score (0-100).
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s >= 90.0 => HealthGrade::Excellent,
            s if s >= 75.0 => HealthGrade::Good,
            s if s >= 60.0 => HealthGrade::Fair,
            s if s >= 40.0 => HealthGrade::Poor,
            _ => HealthGrade::Critical,
        }
    }
}

/// Individual factor contributing to health score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthFactor {
    pub name: String,
    pub weight: f64,
    pub value: f64,
    pub contribution: f64,
}

/// Re-export lifecycle event type for use in health metrics.
pub use crate::lifecycle::LifecycleEventType;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_info_query_new() {
        let agent_id = Uuid::new_v4();
        let query = SystemInfoQuery::new(agent_id);

        assert_eq!(query.agent_id, agent_id);
        assert!(query.include_env);
        assert!(query.include_process);
        assert!(query.include_network);
        assert!(query.include_resources);
    }

    #[test]
    fn test_system_info_query_minimal() {
        let agent_id = Uuid::new_v4();
        let query = SystemInfoQuery::minimal(agent_id);

        assert_eq!(query.agent_id, agent_id);
        assert!(!query.include_env);
        assert!(!query.include_process);
        assert!(!query.include_network);
        assert!(!query.include_resources);
    }

    #[test]
    fn test_system_info_report_builder() {
        let agent_id = Uuid::new_v4();
        let os = OsInfo {
            os_type: "Linux".to_string(),
            distro: "Ubuntu".to_string(),
            kernel: "5.4.0".to_string(),
            architecture: "x86_64".to_string(),
            hostname: "test-host".to_string(),
        };

        let report = SystemInfoReport::new(agent_id)
            .with_os(os.clone());

        assert_eq!(report.agent_id, agent_id);
        assert!(report.environment.is_none());
        assert_eq!(report.os, Some(os));
    }

    #[test]
    fn test_system_info_response_success() {
        let query_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();
        let report = SystemInfoReport::new(agent_id);
        let response = SystemInfoResponse::success(query_id, report);

        assert!(response.success);
        assert!(response.error.is_none());
    }

    #[test]
    fn test_system_info_response_error() {
        let query_id = Uuid::new_v4();
        let response = SystemInfoResponse::error(query_id, "Connection failed");

        assert!(!response.success);
        assert_eq!(response.error, Some("Connection failed".to_string()));
    }

    #[test]
    fn test_health_grade_from_score() {
        assert_eq!(HealthGrade::from_score(95.0), HealthGrade::Excellent);
        assert_eq!(HealthGrade::from_score(80.0), HealthGrade::Good);
        assert_eq!(HealthGrade::from_score(65.0), HealthGrade::Fair);
        assert_eq!(HealthGrade::from_score(50.0), HealthGrade::Poor);
        assert_eq!(HealthGrade::from_score(25.0), HealthGrade::Critical);
    }

    #[test]
    fn test_health_metrics_creation() {
        let agent_id = Uuid::new_v4();
        let metrics = HealthMetrics {
            agent_id,
            timestamp: Utc::now(),
            uptime_seconds: 3600,
            cpu_usage_percent: 45.5,
            memory_usage_percent: 62.3,
            network_connected: true,
            last_event: Some(LifecycleEventType::AgentConnected),
        };

        assert_eq!(metrics.agent_id, agent_id);
        assert_eq!(metrics.uptime_seconds, 3600);
        assert!(metrics.network_connected);
    }

    #[test]
    fn test_health_score_creation() {
        let agent_id = Uuid::new_v4();
        let factors = vec![
            HealthFactor {
                name: "uptime".to_string(),
                weight: 0.3,
                value: 95.0,
                contribution: 28.5,
            },
            HealthFactor {
                name: "cpu".to_string(),
                weight: 0.3,
                value: 80.0,
                contribution: 24.0,
            },
        ];
        let score = HealthScore {
            agent_id,
            score: 85.0,
            grade: HealthGrade::Good,
            factors,
            timestamp: Utc::now(),
        };

        assert_eq!(score.score, 85.0);
        assert_eq!(score.grade, HealthGrade::Good);
        assert_eq!(score.factors.len(), 2);
    }

    #[test]
    fn test_network_interface_serialization() {
        let interface = NetworkInterface {
            name: "eth0".to_string(),
            ip: "192.168.1.100".to_string(),
            mac: Some("00:11:22:33:44:55".to_string()),
        };

        let json = serde_json::to_string(&interface).unwrap();
        let deserialized: NetworkInterface = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "eth0");
        assert_eq!(deserialized.ip, "192.168.1.100");
        assert_eq!(deserialized.mac, Some("00:11:22:33:44:55".to_string()));
    }

    #[test]
    fn test_memory_info_serialization() {
        let memory = MemoryInfo {
            total_bytes: 16_000_000_000,
            used_bytes: 8_000_000_000,
            available_bytes: 8_000_000_000,
            usage_percent: Some(50.0),
        };

        let json = serde_json::to_string(&memory).unwrap();
        let deserialized: MemoryInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_bytes, 16_000_000_000);
        assert_eq!(deserialized.used_bytes, 8_000_000_000);
        assert_eq!(deserialized.usage_percent, Some(50.0));
    }

    #[test]
    fn test_resource_info_serialization() {
        let resource = ResourceInfo {
            cpu: CpuInfo {
                model: "Intel i7".to_string(),
                cores: 8,
                speed_mhz: Some(3200.0),
                usage_percent: Some(25.0),
            },
            memory: MemoryInfo {
                total_bytes: 16_000_000_000,
                used_bytes: 4_000_000_000,
                available_bytes: 12_000_000_000,
                usage_percent: Some(25.0),
            },
            disks: vec![
                DiskInfo {
                    mount_point: "/".to_string(),
                    total_bytes: 500_000_000_000,
                    used_bytes: 250_000_000_000,
                    available_bytes: 250_000_000_000,
                    usage_percent: Some(50.0),
                },
            ],
        };

        let json = serde_json::to_string(&resource).unwrap();
        let deserialized: ResourceInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.cpu.cores, 8);
        assert_eq!(deserialized.memory.total_bytes, 16_000_000_000);
        assert_eq!(deserialized.disks.len(), 1);
    }
}
