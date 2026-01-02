//! Tests for fallback engine.

use std::collections::HashMap;

use crate::specialist_v2::schema::SpecialistStatus;

use super::FallbackEngine;

#[test]
fn test_memory_fallback() {
    let mut probes = HashMap::new();
    probes.insert(
        "free".to_string(),
        "Mem:           31Gi       14Gi       17Gi".to_string(),
    );

    let engine = FallbackEngine::new("show_memory", "How much free RAM?", probes);
    let result = engine.try_fallback("test");

    assert!(result.success);
    assert!(result.response.has_direct_answer());
}

#[test]
fn test_services_fallback_none() {
    let mut probes = HashMap::new();
    probes.insert(
        "systemctl_failed".to_string(),
        "0 loaded units listed.".to_string(),
    );

    let engine = FallbackEngine::new("check_failed_services", "Any failed services?", probes);
    let result = engine.try_fallback("test");

    assert!(result.success);
    assert!(result.response.main_text().contains("No,"));
}

#[test]
fn test_services_fallback_failed() {
    let mut probes = HashMap::new();
    probes.insert(
        "systemctl_failed".to_string(),
        "foo.service failed\nbar.service failed".to_string(),
    );

    let engine = FallbackEngine::new("check_failed_services", "Any failed services?", probes);
    let result = engine.try_fallback("test");

    assert!(result.success);
    assert!(result.response.main_text().contains("Yes,"));
    assert_eq!(result.response.key_findings.len(), 2);
}

#[test]
fn test_disk_fallback() {
    let mut probes = HashMap::new();
    probes.insert(
        "df".to_string(),
        "Filesystem Size Used Avail Use% Mounted on\n/dev/sda1 100G 85G 15G 85% /".to_string(),
    );

    let engine = FallbackEngine::new("show_disk_usage", "How much disk space?", probes);
    let result = engine.try_fallback("test");

    assert!(result.success);
}

#[test]
fn test_generic_fallback() {
    let probes = HashMap::new();
    let engine = FallbackEngine::new("unknown_intent", "Unknown question", probes);
    let result = engine.try_fallback("test");

    assert!(!result.success);
    assert_eq!(
        result.response.status,
        SpecialistStatus::InsufficientEvidence
    );
}
