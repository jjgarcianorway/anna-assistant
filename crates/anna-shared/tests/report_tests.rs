//! Golden tests for deterministic report generation.
//!
//! Tests verify:
//! - Health checks use pinned thresholds
//! - Executive summary is accurate
//! - Output is deterministic (same input = same output)

use anna_shared::parsers::{
    BlockDevice, BlockDeviceType, CpuInfo, DiskUsage, MemoryInfo, ServiceState, ServiceStatus,
};
use anna_shared::report::{
    format_markdown, format_text, HealthSeverity, ReportEvidence, SystemReport,
};
use anna_shared::trace::{EvidenceKind, ExecutionTrace, ProbeStats};

/// Helper to create test memory info
fn make_memory(total_gb: u64, used_gb: u64) -> MemoryInfo {
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
fn make_disk(mount: &str, total_gb: u64, percent: u8) -> DiskUsage {
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
fn make_cpu(cores: u32) -> CpuInfo {
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
fn make_disk_device(name: &str, size_gb: u64) -> BlockDevice {
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
fn make_partition(name: &str, size_gb: u64, mount: &str) -> BlockDevice {
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
fn make_trace(evidence_kinds: Vec<EvidenceKind>) -> ExecutionTrace {
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

// =============================================================================
// Health check threshold tests
// =============================================================================

#[test]
fn golden_disk_warning_at_85_percent() {
    // DISK_WARNING_THRESHOLD is 85%
    let evidence = ReportEvidence {
        memory: None,
        disks: vec![make_disk("/", 100, 85)],
        block_devices: vec![],
        cpu: None,
        failed_services: vec![],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);

    let disk_check = report.health_checks.iter().find(|c| c.id == "disk_root");
    assert!(disk_check.is_some());
    assert_eq!(disk_check.unwrap().severity, HealthSeverity::Warning);
}

#[test]
fn golden_disk_critical_at_95_percent() {
    let evidence = ReportEvidence {
        memory: None,
        disks: vec![make_disk("/", 100, 95)],
        block_devices: vec![],
        cpu: None,
        failed_services: vec![],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);

    let disk_check = report.health_checks.iter().find(|c| c.id == "disk_root");
    assert!(disk_check.is_some());
    assert_eq!(disk_check.unwrap().severity, HealthSeverity::Critical);
}

#[test]
fn golden_disk_ok_at_84_percent() {
    // Just below DISK_WARNING_THRESHOLD (85%)
    let evidence = ReportEvidence {
        memory: None,
        disks: vec![make_disk("/", 100, 84)],
        block_devices: vec![],
        cpu: None,
        failed_services: vec![],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);

    let disk_check = report.health_checks.iter().find(|c| c.id == "disk_root");
    assert!(disk_check.is_some());
    assert_eq!(disk_check.unwrap().severity, HealthSeverity::Ok);
}

#[test]
fn golden_memory_warning_at_90_percent() {
    // 90% = Warning threshold (MEMORY_HIGH_THRESHOLD = 0.9)
    let evidence = ReportEvidence {
        memory: Some(make_memory(16, 15)), // ~94% used
        disks: vec![],
        block_devices: vec![],
        cpu: None,
        failed_services: vec![],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);

    let mem_check = report.health_checks.iter().find(|c| c.id == "memory");
    assert!(mem_check.is_some());
    assert_eq!(mem_check.unwrap().severity, HealthSeverity::Warning);
}

#[test]
fn golden_memory_ok_at_50_percent() {
    let evidence = ReportEvidence {
        memory: Some(make_memory(16, 8)), // 50% used
        disks: vec![],
        block_devices: vec![],
        cpu: None,
        failed_services: vec![],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);

    let mem_check = report.health_checks.iter().find(|c| c.id == "memory");
    assert!(mem_check.is_some());
    assert_eq!(mem_check.unwrap().severity, HealthSeverity::Ok);
}

// =============================================================================
// Executive summary tests
// =============================================================================

#[test]
fn golden_executive_summary_healthy() {
    let evidence = ReportEvidence {
        memory: Some(make_memory(16, 8)),
        disks: vec![make_disk("/", 100, 50)],
        block_devices: vec![],
        cpu: Some(make_cpu(8)),
        failed_services: vec![],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);

    assert!(!report.executive_summary.is_empty());
    assert!(report.executive_summary[0].contains("healthy"));
}

#[test]
fn golden_executive_summary_with_issues() {
    let evidence = ReportEvidence {
        memory: None,
        disks: vec![make_disk("/", 100, 95)], // Critical
        block_devices: vec![],
        cpu: None,
        failed_services: vec![ServiceStatus {
            name: "nginx.service".to_string(),
            state: ServiceState::Failed,
            description: Some("A web server".to_string()),
        }],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);

    assert!(!report.executive_summary.is_empty());
    assert!(report.executive_summary[0].contains("issue"));
    assert!(report.executive_summary[0].contains("critical"));
}

// =============================================================================
// Inventory tests
// =============================================================================

#[test]
fn golden_inventory_cpu_and_memory() {
    let evidence = ReportEvidence {
        memory: Some(make_memory(32, 16)),
        disks: vec![],
        block_devices: vec![],
        cpu: Some(make_cpu(16)),
        failed_services: vec![],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);

    assert_eq!(report.inventory.cpu_cores, Some(16));
    assert_eq!(report.inventory.memory_total_bytes, Some(32 * 1024 * 1024 * 1024));
}

#[test]
fn golden_inventory_block_devices() {
    let evidence = ReportEvidence {
        memory: None,
        disks: vec![],
        block_devices: vec![
            make_disk_device("sda", 500),
            make_partition("sda1", 100, "/boot"),
            make_partition("sda2", 400, "/"),
        ],
        cpu: None,
        failed_services: vec![],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);

    assert_eq!(report.inventory.block_device_count, 3);
    assert!(report.inventory.block_device_summary.contains("1 disk"));
    assert!(report.inventory.block_device_summary.contains("2 partition"));
}

// =============================================================================
// Determinism tests
// =============================================================================

#[test]
fn golden_report_deterministic() {
    let evidence = ReportEvidence {
        memory: Some(make_memory(16, 8)),
        disks: vec![make_disk("/", 100, 50), make_disk("/home", 200, 75)],
        block_devices: vec![make_disk_device("sda", 500)],
        cpu: Some(make_cpu(8)),
        failed_services: vec![],
    };
    let trace = make_trace(vec![
        EvidenceKind::Memory,
        EvidenceKind::Disk,
        EvidenceKind::Cpu,
    ]);

    let report1 = SystemReport::from_evidence(&evidence, Some(&trace), 100, None);
    let report2 = SystemReport::from_evidence(&evidence, Some(&trace), 100, None);

    // Same input = same output
    let text1 = format_text(&report1);
    let text2 = format_text(&report2);
    assert_eq!(text1, text2);

    let md1 = format_markdown(&report1);
    let md2 = format_markdown(&report2);
    assert_eq!(md1, md2);
}

#[test]
fn golden_health_checks_sorted_by_severity() {
    let evidence = ReportEvidence {
        memory: Some(make_memory(16, 8)), // OK
        disks: vec![
            make_disk("/", 100, 95),      // Critical
            make_disk("/home", 200, 85),  // Warning
            make_disk("/var", 50, 50),    // OK
        ],
        block_devices: vec![],
        cpu: None,
        failed_services: vec![],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);

    // Critical should come first
    let severities: Vec<_> = report.health_checks.iter().map(|c| c.severity).collect();
    let mut expected = severities.clone();
    expected.sort_by(|a, b| b.cmp(a)); // Descending by severity

    // Check that critical comes before warning, warning before ok
    let critical_idx = report.health_checks.iter().position(|c| c.severity == HealthSeverity::Critical);
    let warning_idx = report.health_checks.iter().position(|c| c.severity == HealthSeverity::Warning);

    if let (Some(c), Some(w)) = (critical_idx, warning_idx) {
        assert!(c < w, "Critical should come before Warning");
    }
}

// =============================================================================
// Format output tests
// =============================================================================

#[test]
fn golden_text_format_contains_sections() {
    let evidence = ReportEvidence {
        memory: Some(make_memory(16, 8)),
        disks: vec![make_disk("/", 100, 50)],
        block_devices: vec![],
        cpu: Some(make_cpu(8)),
        failed_services: vec![],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);
    let text = format_text(&report);

    assert!(text.contains("SYSTEM REPORT"));
    assert!(text.contains("EXECUTIVE SUMMARY"));
    assert!(text.contains("INVENTORY"));
    assert!(text.contains("HEALTH CHECKS"));
    assert!(text.contains("RELIABILITY"));
}

#[test]
fn golden_markdown_format_contains_sections() {
    let evidence = ReportEvidence {
        memory: Some(make_memory(16, 8)),
        disks: vec![make_disk("/", 100, 50)],
        block_devices: vec![],
        cpu: Some(make_cpu(8)),
        failed_services: vec![],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);
    let md = format_markdown(&report);

    assert!(md.contains("# System Report"));
    assert!(md.contains("## Executive Summary"));
    assert!(md.contains("## Inventory"));
    assert!(md.contains("## Health Checks"));
    assert!(md.contains("## Reliability"));
    assert!(md.contains("|")); // Tables
}

// =============================================================================
// Service health tests
// =============================================================================

#[test]
fn golden_failed_services_flagged() {
    let evidence = ReportEvidence {
        memory: None,
        disks: vec![],
        block_devices: vec![],
        cpu: None,
        failed_services: vec![
            ServiceStatus {
                name: "nginx.service".to_string(),
                state: ServiceState::Failed,
                description: Some("Web server".to_string()),
            },
            ServiceStatus {
                name: "postgresql.service".to_string(),
                state: ServiceState::Failed,
                description: Some("Database".to_string()),
            },
        ],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);

    let svc_check = report.health_checks.iter().find(|c| c.id == "services");
    assert!(svc_check.is_some());
    assert_eq!(svc_check.unwrap().severity, HealthSeverity::Warning);
    assert!(svc_check.unwrap().claim.contains("nginx"));
    assert!(svc_check.unwrap().claim.contains("postgresql"));
}

#[test]
fn golden_no_failed_services_healthy() {
    let evidence = ReportEvidence {
        memory: None,
        disks: vec![],
        block_devices: vec![],
        cpu: None,
        failed_services: vec![ServiceStatus {
            name: "nginx.service".to_string(),
            state: ServiceState::Running,
            description: Some("Web server".to_string()),
        }],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);

    let svc_check = report.health_checks.iter().find(|c| c.id == "services");
    assert!(svc_check.is_some());
    assert_eq!(svc_check.unwrap().severity, HealthSeverity::Ok);
}
