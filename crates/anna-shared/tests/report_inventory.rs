//! Golden tests for system inventory generation.
//!
//! Tests verify that inventory data is correctly captured from evidence.

mod report_test_helpers;

use anna_shared::report::{ReportEvidence, SystemReport};
use report_test_helpers::{make_cpu, make_disk_device, make_memory, make_partition};

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
    assert_eq!(
        report.inventory.memory_total_bytes,
        Some(32 * 1024 * 1024 * 1024)
    );
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
    assert!(report
        .inventory
        .block_device_summary
        .contains("2 partition"));
}
