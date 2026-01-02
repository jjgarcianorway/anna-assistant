//! Tests for QuestionIntent and related types.

use super::builder::IntentBuilder;
use super::constraints::AnswerConstraints;
use super::enums::{IntentCategory, Subject};

#[test]
fn test_intent_builder() {
    let intent = IntentBuilder::new("int_001")
        .category(IntentCategory::Fact)
        .subject(Subject::Memory)
        .scope(super::enums::Scope::Single)
        .allow_fields(vec!["free"])
        .build();

    assert_eq!(intent.category, IntentCategory::Fact);
    assert_eq!(intent.subject, Subject::Memory);
    assert!(!intent.allows_extras());
    assert!(intent.is_field_allowed("free"));
    assert!(!intent.is_field_allowed("total"));
}

#[test]
fn test_clarification_stops_execution() {
    let intent = IntentBuilder::new("int_002")
        .needs_clarification("Which service?", vec!["nginx", "apache", "postgresql"])
        .build();

    assert!(intent.needs_clarification());
}

#[test]
fn test_category_allows_tutorials() {
    assert!(!IntentCategory::Fact.allows_tutorials());
    assert!(!IntentCategory::Status.allows_tutorials());
    assert!(IntentCategory::Explanation.allows_tutorials());
    assert!(IntentCategory::ActionRequest.allows_tutorials());
}

#[test]
fn test_constraints_default_no_extras() {
    let constraints = AnswerConstraints::default();
    assert!(!constraints.allow_extras);
}

#[test]
fn test_single_fact_constraints() {
    let constraints = AnswerConstraints::single_fact("free_ram");
    assert_eq!(constraints.max_items, Some(1));
    assert!(!constraints.allow_extras);
    assert_eq!(constraints.allowed_fields, vec!["free_ram"]);
}

#[test]
fn test_boolean_constraints() {
    let constraints = AnswerConstraints::boolean();
    assert_eq!(constraints.max_items, Some(1));
    assert!(!constraints.allow_extras);
}

#[test]
fn test_field_allowed() {
    let intent = IntentBuilder::new("int_003")
        .allow_fields(vec!["free", "total"])
        .build();

    assert!(intent.is_field_allowed("free"));
    assert!(intent.is_field_allowed("total"));
    assert!(!intent.is_field_allowed("cached"));
}

#[test]
fn test_extras_allowed() {
    let intent = IntentBuilder::new("int_004")
        .category(IntentCategory::Diagnosis)
        .allow_extras()
        .build();

    assert!(intent.allows_extras());
    assert!(intent.is_field_allowed("anything"));
}
