//! Tests for canonical intents.

use super::*;

#[test]
fn test_canonical_intent_from_str() {
    assert_eq!(
        CanonicalIntent::from_str("check_free_ram"),
        CanonicalIntent::CheckFreeRam
    );
    assert_eq!(
        CanonicalIntent::from_str("disk_usage"),
        CanonicalIntent::CheckDiskUsage
    );
    assert_eq!(
        CanonicalIntent::from_str("unknown"),
        CanonicalIntent::GeneralQuery
    );
}

#[test]
fn test_required_probes() {
    let probes = CanonicalIntent::CheckDiskUsage.required_probes();
    assert!(probes.contains(&"disk_usage"));
}

#[test]
fn test_knowledge_topics() {
    let topics = CanonicalIntent::CheckFailedServices.knowledge_topics();
    assert!(topics.contains(&"systemctl"));
}

#[test]
fn test_translator_to_canonical() {
    let intent =
        translator_to_canonical("query_metric", "storage", &["disk_usage".to_string()]);
    assert_eq!(intent, CanonicalIntent::CheckDiskUsage);
}

#[test]
fn test_recipe_eligibility() {
    assert!(CanonicalIntent::CheckDiskUsage.is_recipe_eligible());
    assert!(!CanonicalIntent::ExplainConcept.is_recipe_eligible());
}
