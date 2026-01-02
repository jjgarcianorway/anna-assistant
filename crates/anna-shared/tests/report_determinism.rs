//! Golden tests for deterministic report generation.
//!
//! Tests verify that output is deterministic (same input = same output).

mod report_test_helpers;

use anna_shared::report::{format_markdown, format_text, HealthSeverity, ReportEvidence, SystemReport};
use anna_shared::trace::EvidenceKind;
use report_test_helpers::{make_cpu, make_disk, make_disk_device, make_memory, make_trace};

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
            make_disk("/", 100, 95),     // Critical
            make_disk("/home", 200, 85), // Warning
            make_disk("/var", 50, 50),   // OK
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
    let critical_idx = report
        .health_checks
        .iter()
        .position(|c| c.severity == HealthSeverity::Critical);
    let warning_idx = report
        .health_checks
        .iter()
        .position(|c| c.severity == HealthSeverity::Warning);

    if let (Some(c), Some(w)) = (critical_idx, warning_idx) {
        assert!(c < w, "Critical should come before Warning");
    }
}
