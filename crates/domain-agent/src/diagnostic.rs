//! System diagnostic collection for domain-agent.

use agent_protocol::diagnostic::{
    CpuInfo, DiskInfo, EnvironmentInfo, MemoryInfo, NetworkInfo,
    NetworkInterface, OsInfo, ProcessInfo, ResourceInfo, SystemInfoReport,
};
use std::collections::HashMap;
use std::env;
use sysinfo::Disks;
use uuid::Uuid;

/// Collect operating system information.
pub fn collect_os_info() -> OsInfo {
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    OsInfo {
        os_type: env::consts::OS.to_string(),
        distro: String::new(), // Not easily available on Windows
        kernel: String::new(), // Not easily available on Windows
        architecture: env::consts::ARCH.to_string(),
        hostname,
    }
}

/// Collect environment information.
pub fn collect_environment_info() -> EnvironmentInfo {
    let env_vars: HashMap<String, String> = env::vars().collect();
    let current_working_dir = env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let user = env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());

    EnvironmentInfo {
        env_vars,
        current_working_dir,
        user,
        uid_gid: None,
    }
}

/// Collect process information.
pub fn collect_process_info() -> ProcessInfo {
    ProcessInfo {
        pid: std::process::id(),
        parent_pid: None,
        command: env::args().next().unwrap_or_else(|| "domain-agent".to_string()),
        start_time: None,
        uptime_seconds: None,
    }
}

/// Collect network information (stubbed for Windows).
pub fn collect_network_info() -> NetworkInfo {
    // Network info collection is stubbed - requires platform-specific implementation
    NetworkInfo {
        interfaces: vec![NetworkInterface {
            name: "unknown".to_string(),
            ip: "0.0.0.0".to_string(),
            mac: None,
        }],
        connections: vec![],
    }
}

/// Collect resource information using sysinfo crate.
pub fn collect_resource_info() -> ResourceInfo {
    let mut sys = sysinfo::System::new_all();

    // Refresh all data
    sys.refresh_all();

    // Get CPU info
    let cpus = sys.cpus();
    let cpu = if let Some(first_cpu) = cpus.first() {
        CpuInfo {
            model: first_cpu.brand().to_string(),
            cores: cpus.len(),
            speed_mhz: Some(first_cpu.frequency() as f64),
            usage_percent: Some(sys.global_cpu_usage() as f64),
        }
    } else {
        CpuInfo {
            model: "unknown".to_string(),
            cores: 0,
            speed_mhz: None,
            usage_percent: None,
        }
    };

    // Get memory info
    let memory = MemoryInfo {
        total_bytes: sys.total_memory(),
        used_bytes: sys.used_memory(),
        available_bytes: sys.available_memory(),
        usage_percent: Some((sys.used_memory() as f64 / sys.total_memory() as f64 * 100.0) as f64),
    };

    // Get disk info
    let disks_list = Disks::new_with_refreshed_list();
    let disks: Vec<DiskInfo> = disks_list
        .iter()
        .map(|disk| {
            DiskInfo {
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_bytes: disk.total_space(),
                used_bytes: disk.total_space().saturating_sub(disk.available_space()),
                available_bytes: disk.available_space(),
                usage_percent: Some(
                    ((disk.total_space() - disk.available_space()) as f64
                        / disk.total_space() as f64
                        * 100.0) as f64,
                ),
            }
        })
        .collect();

    ResourceInfo {
        cpu,
        memory,
        disks,
    }
}

/// Collect complete system information report.
pub fn collect_system_info(agent_id: Uuid) -> SystemInfoReport {
    SystemInfoReport::new(agent_id)
        .with_os(collect_os_info())
        .with_environment(collect_environment_info())
        .with_process(collect_process_info())
        .with_network(collect_network_info())
        .with_resources(collect_resource_info())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_os_info() {
        let os_info = collect_os_info();
        assert!(!os_info.os_type.is_empty());
        assert!(!os_info.architecture.is_empty());
        assert!(!os_info.hostname.is_empty());
    }

    #[test]
    fn test_collect_environment_info() {
        let env_info = collect_environment_info();
        assert!(!env_info.current_working_dir.is_empty());
        assert!(!env_info.user.is_empty());
    }

    #[test]
    fn test_collect_process_info() {
        let process_info = collect_process_info();
        assert_eq!(process_info.pid, std::process::id());
    }

    #[test]
    fn test_collect_network_info() {
        let network_info = collect_network_info();
        assert!(!network_info.interfaces.is_empty());
    }

    #[test]
    fn test_collect_resource_info() {
        let resource_info = collect_resource_info();
        assert!(resource_info.cpu.cores > 0);
        assert!(resource_info.memory.total_bytes > 0);
    }

    #[test]
    fn test_collect_system_info() {
        let agent_id = Uuid::new_v4();
        let report = collect_system_info(agent_id);

        assert_eq!(report.agent_id, agent_id);
        assert!(report.os.is_some());
        assert!(report.environment.is_some());
        assert!(report.process.is_some());
        assert!(report.network.is_some());
        assert!(report.resources.is_some());
    }
}
