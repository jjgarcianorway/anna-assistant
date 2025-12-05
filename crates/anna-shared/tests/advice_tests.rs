//! Golden tests for ADVISE (rule-based recommendations).
//!
//! Tests verify:
//! - Recommendations are generated from health checks
//! - Severity mapping is correct
//! - Command hints are provided
//! - Output is deterministic

use anna_shared::advice::{
    format_recommendations_markdown, format_recommendations_text, generate_recommendations,
    RecommendationSeverity,
};
use anna_shared::parsers::{CpuInfo, DiskUsage, MemoryInfo, ServiceState, ServiceStatus};
use anna_shared::report::{ReportEvidence, SystemReport};

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

// =============================================================================
// Recommendation generation tests
// =============================================================================

#[test]
fn golden_no_recommendations_when_healthy() {
    let evidence = ReportEvidence {
        memory: Some(make_memory(16, 8)),      // 50% - OK
        disks: vec![make_disk("/", 100, 50)],  // 50% - OK
        block_devices: vec![],
        cpu: Some(make_cpu(8)),
        failed_services: vec![],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);
    let recs = generate_recommendations(&report);

    assert!(recs.is_empty());
    assert_eq!(recs.critical_count(), 0);
    assert_eq!(recs.warning_count(), 0);
}

#[test]
fn golden_disk_critical_recommendation() {
    let evidence = ReportEvidence {
        memory: None,
        disks: vec![make_disk("/", 100, 95)], // 95% - Critical
        block_devices: vec![],
        cpu: None,
        failed_services: vec![],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);
    let recs = generate_recommendations(&report);

    assert!(!recs.is_empty());
    assert_eq!(recs.critical_count(), 1);

    let disk_rec = recs.items.iter().find(|r| r.evidence_ref == "disk_root");
    assert!(disk_rec.is_some());
    let rec = disk_rec.unwrap();
    assert_eq!(rec.severity, RecommendationSeverity::Critical);
    assert!(rec.action.contains("Urgently"));
    assert!(rec.command_hint.is_some());
    assert!(rec.command_hint.as_ref().unwrap().contains("du"));
}

#[test]
fn golden_disk_warning_recommendation() {
    let evidence = ReportEvidence {
        memory: None,
        disks: vec![make_disk("/home", 200, 85)], // 85% - Warning
        block_devices: vec![],
        cpu: None,
        failed_services: vec![],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);
    let recs = generate_recommendations(&report);

    assert!(!recs.is_empty());
    assert_eq!(recs.warning_count(), 1);

    let disk_rec = recs.items.iter().find(|r| r.evidence_ref == "disk_home");
    assert!(disk_rec.is_some());
    let rec = disk_rec.unwrap();
    assert_eq!(rec.severity, RecommendationSeverity::Warning);
    assert!(rec.action.contains("Consider"));
    assert!(rec.command_hint.as_ref().unwrap().contains("/home"));
}

#[test]
fn golden_memory_warning_recommendation() {
    let evidence = ReportEvidence {
        memory: Some(make_memory(16, 15)), // ~94% - Warning
        disks: vec![],
        block_devices: vec![],
        cpu: None,
        failed_services: vec![],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);
    let recs = generate_recommendations(&report);

    assert!(!recs.is_empty());
    assert_eq!(recs.warning_count(), 1);

    let mem_rec = recs.items.iter().find(|r| r.evidence_ref == "memory");
    assert!(mem_rec.is_some());
    let rec = mem_rec.unwrap();
    assert_eq!(rec.severity, RecommendationSeverity::Warning);
    assert!(rec.action.contains("memory"));
    assert!(rec.command_hint.as_ref().unwrap().contains("ps"));
}

#[test]
fn golden_failed_services_recommendation() {
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
        ],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);
    let recs = generate_recommendations(&report);

    assert!(!recs.is_empty());

    let svc_rec = recs.items.iter().find(|r| r.evidence_ref == "services");
    assert!(svc_rec.is_some());
    let rec = svc_rec.unwrap();
    assert_eq!(rec.severity, RecommendationSeverity::Warning);
    assert!(rec.command_hint.as_ref().unwrap().contains("systemctl"));
}

// =============================================================================
// Sorting and ordering tests
// =============================================================================

#[test]
fn golden_critical_before_warning() {
    let evidence = ReportEvidence {
        memory: Some(make_memory(16, 15)),    // Warning
        disks: vec![make_disk("/", 100, 95)], // Critical
        block_devices: vec![],
        cpu: None,
        failed_services: vec![],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);
    let recs = generate_recommendations(&report);

    assert!(recs.items.len() >= 2);
    // First item should be critical
    assert_eq!(recs.items[0].severity, RecommendationSeverity::Critical);
}

#[test]
fn golden_multiple_disks_sorted() {
    let evidence = ReportEvidence {
        memory: None,
        disks: vec![
            make_disk("/", 100, 95),     // Critical
            make_disk("/home", 200, 85), // Warning
            make_disk("/var", 50, 90),   // Warning
        ],
        block_devices: vec![],
        cpu: None,
        failed_services: vec![],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);
    let recs = generate_recommendations(&report);

    assert_eq!(recs.critical_count(), 1);
    assert_eq!(recs.warning_count(), 2);

    // Critical should be first
    assert_eq!(recs.items[0].severity, RecommendationSeverity::Critical);
    assert_eq!(recs.items[0].evidence_ref, "disk_root");
}

// =============================================================================
// Format output tests
// =============================================================================

#[test]
fn golden_text_format_with_recommendations() {
    let evidence = ReportEvidence {
        memory: None,
        disks: vec![make_disk("/", 100, 95)],
        block_devices: vec![],
        cpu: None,
        failed_services: vec![],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);
    let recs = generate_recommendations(&report);
    let text = format_recommendations_text(&recs);

    assert!(text.contains("RECOMMENDED ACTIONS"));
    assert!(text.contains("[CRITICAL]"));
    assert!(text.contains("Evidence:"));
    assert!(text.contains("Try:"));
}

#[test]
fn golden_markdown_format_with_recommendations() {
    let evidence = ReportEvidence {
        memory: None,
        disks: vec![make_disk("/", 100, 95)],
        block_devices: vec![],
        cpu: None,
        failed_services: vec![],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);
    let recs = generate_recommendations(&report);
    let md = format_recommendations_markdown(&recs);

    assert!(md.contains("## Recommended Actions"));
    assert!(md.contains("**CRITICAL**"));
    assert!(md.contains("*Evidence*"));
    assert!(md.contains("`"));
}

// =============================================================================
// Determinism tests
// =============================================================================

#[test]
fn golden_recommendations_deterministic() {
    let evidence = ReportEvidence {
        memory: Some(make_memory(16, 15)),
        disks: vec![
            make_disk("/", 100, 95),
            make_disk("/home", 200, 85),
        ],
        block_devices: vec![],
        cpu: Some(make_cpu(8)),
        failed_services: vec![ServiceStatus {
            name: "nginx.service".to_string(),
            state: ServiceState::Failed,
            description: Some("Web server".to_string()),
        }],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);

    let recs1 = generate_recommendations(&report);
    let recs2 = generate_recommendations(&report);

    // Same number of recommendations
    assert_eq!(recs1.items.len(), recs2.items.len());

    // Same order
    for (r1, r2) in recs1.items.iter().zip(recs2.items.iter()) {
        assert_eq!(r1.severity, r2.severity);
        assert_eq!(r1.evidence_ref, r2.evidence_ref);
        assert_eq!(r1.action, r2.action);
    }

    // Same formatted output
    let text1 = format_recommendations_text(&recs1);
    let text2 = format_recommendations_text(&recs2);
    assert_eq!(text1, text2);
}

// =============================================================================
// Edge case tests
// =============================================================================

#[test]
fn golden_empty_evidence() {
    let evidence = ReportEvidence::default();
    let report = SystemReport::from_evidence(&evidence, None, 100, None);
    let recs = generate_recommendations(&report);

    // May have services recommendation even with empty evidence
    // (services OK is still a health check)
    assert!(recs.items.len() <= 1);
}

#[test]
fn golden_all_systems_critical() {
    let evidence = ReportEvidence {
        memory: Some(make_memory(16, 15)),     // ~94% - Warning
        disks: vec![
            make_disk("/", 100, 96),           // Critical
            make_disk("/home", 200, 97),       // Critical
        ],
        block_devices: vec![],
        cpu: None,
        failed_services: vec![
            ServiceStatus {
                name: "nginx.service".to_string(),
                state: ServiceState::Failed,
                description: None,
            },
            ServiceStatus {
                name: "postgresql.service".to_string(),
                state: ServiceState::Failed,
                description: None,
            },
        ],
    };

    let report = SystemReport::from_evidence(&evidence, None, 100, None);
    let recs = generate_recommendations(&report);

    // Should have 4 recommendations: 2 disk critical, 1 memory warning, 1 services warning
    assert_eq!(recs.critical_count(), 2);
    assert!(recs.warning_count() >= 1);
}
