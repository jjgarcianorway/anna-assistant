//! Golden tests for executive summary generation.
//!
//! Tests verify that executive summaries accurately reflect system status.

mod report_test_helpers;

use anna_shared::parsers::ServiceState;
use anna_shared::report::{ReportEvidence, SystemReport};
use report_test_helpers::{make_cpu, make_disk, make_memory, make_service};

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
        failed_services: vec![make_service(
            "nginx.service",
            ServiceState::Failed,
            Some("A web server"),
        )],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);

    assert!(!report.executive_summary.is_empty());
    assert!(report.executive_summary[0].contains("issue"));
    assert!(report.executive_summary[0].contains("critical"));
}
