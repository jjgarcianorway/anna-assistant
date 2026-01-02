//! Golden tests for health check thresholds.
//!
//! Tests verify that health checks use pinned thresholds:
//! - Disk warning at 85%, critical at 95%
//! - Memory warning at 90%

mod report_test_helpers;

use anna_shared::report::{HealthSeverity, ReportEvidence, SystemReport};
use report_test_helpers::{make_disk, make_memory};

// =============================================================================
// Disk threshold tests
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

// =============================================================================
// Memory threshold tests
// =============================================================================

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
