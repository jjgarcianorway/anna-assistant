//! Golden tests for service health checks.
//!
//! Tests verify that failed services are properly detected and reported.

mod report_test_helpers;

use anna_shared::parsers::ServiceState;
use anna_shared::report::{HealthSeverity, ReportEvidence, SystemReport};
use report_test_helpers::make_service;

#[test]
fn golden_failed_services_flagged() {
    let evidence = ReportEvidence {
        memory: None,
        disks: vec![],
        block_devices: vec![],
        cpu: None,
        failed_services: vec![
            make_service("nginx.service", ServiceState::Failed, Some("Web server")),
            make_service(
                "postgresql.service",
                ServiceState::Failed,
                Some("Database"),
            ),
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
        failed_services: vec![make_service(
            "nginx.service",
            ServiceState::Running,
            Some("Web server"),
        )],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);

    let svc_check = report.health_checks.iter().find(|c| c.id == "services");
    assert!(svc_check.is_some());
    assert_eq!(svc_check.unwrap().severity, HealthSeverity::Ok);
}
