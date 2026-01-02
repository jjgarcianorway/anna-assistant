//! Golden tests for report formatting.
//!
//! Tests verify that text and markdown outputs contain expected sections.

mod report_test_helpers;

use anna_shared::report::{format_markdown, format_text, ReportEvidence, SystemReport};
use report_test_helpers::{make_cpu, make_disk, make_memory};

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
