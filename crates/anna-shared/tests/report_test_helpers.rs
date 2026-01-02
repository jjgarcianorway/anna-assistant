//! Shared test helpers for report testing.

use anna_shared::parsers::{
    BlockDevice, BlockDeviceType, CpuInfo, DiskUsage, MemoryInfo, ServiceState, ServiceStatus,
};
use anna_shared::trace::{EvidenceKind, ExecutionTrace, ProbeStats};

/// Helper to create test memory info
pub fn make_memory(total_gb: u64, used_gb: u64) -> MemoryInfo {
    let gb = 1024 * 1024 * 1024;
    MemoryInfo {
        total_bytes: total_gb * gb,
        used_bytes: used_gb * gb,
        free_bytes: (total_gb - used_gb) * gb / 2,
        shared_bytes: 0,
        buff_cache_bytes: (total_gb - used_gb) * gb / 2,
        available_bytes: (total_gb - used_gb) * gb,
        swap_total_bytes: None,
        swap_used_bytes: None,
        swap_free_bytes: None,
    }
}

/// Helper to create test disk usage
pub fn make_disk(mount: &str, total_gb: u64, percent: u8) -> DiskUsage {
    let gb = 1024 * 1024 * 1024;
    let total = total_gb * gb;
    let used = (total as f64 * (percent as f64 / 100.0)) as u64;
    DiskUsage {
        filesystem: format!("/dev/sda{}", if mount == "/" { "1" } else { "2" }),
        mount: mount.to_string(),
        size_bytes: total,
        used_bytes: used,
        available_bytes: total - used,
        percent_used: percent,
    }
}

/// Helper to create test CPU info
pub fn make_cpu(cores: u32) -> CpuInfo {
    CpuInfo {
        architecture: "x86_64".to_string(),
        model_name: "Test CPU".to_string(),
        cpu_count: cores,
        cores_per_socket: Some(cores / 2),
        threads_per_core: Some(2),
        sockets: Some(1),
        vendor_id: None,
        cpu_family: None,
        model: None,
    }
}

/// Helper to create test block device
pub fn make_disk_device(name: &str, size_gb: u64) -> BlockDevice {
    let gb = 1024 * 1024 * 1024;
    BlockDevice {
        name: name.to_string(),
        size_bytes: size_gb * gb,
        device_type: BlockDeviceType::Disk,
        mountpoints: vec![],
        parent: None,
        read_only: false,
    }
}

/// Helper to create test partition
pub fn make_partition(name: &str, size_gb: u64, mount: &str) -> BlockDevice {
    let gb = 1024 * 1024 * 1024;
    BlockDevice {
        name: name.to_string(),
        size_bytes: size_gb * gb,
        device_type: BlockDeviceType::Part,
        mountpoints: vec![mount.to_string()],
        parent: Some("sda".to_string()),
        read_only: false,
    }
}

/// Helper to create test trace
pub fn make_trace(evidence_kinds: Vec<EvidenceKind>) -> ExecutionTrace {
    ExecutionTrace::deterministic_route(
        "test_report",
        ProbeStats {
            planned: 5,
            succeeded: 5,
            failed: 0,
            timed_out: 0,
        },
        evidence_kinds,
    )
}

/// Helper to create test service status
pub fn make_service(name: &str, state: ServiceState, description: Option<&str>) -> ServiceStatus {
    ServiceStatus {
        name: name.to_string(),
        state,
        description: description.map(|s| s.to_string()),
    }
}
