//! LLM Assignment Tests

use super::*;
use std::collections::HashMap;

fn make_assignment(specialist: &str, model: &str, tier: ModelTier) -> LlmAssignment {
    LlmAssignment {
        specialist_id: specialist.to_string(),
        model: model.to_string(),
        tier,
        reason: AssignmentReason::Default,
        assigned_at: 1234567890,
        active: true,
        parameters: HashMap::new(),
    }
}

#[test]
fn test_model_tier() {
    assert_eq!(ModelTier::Light.name(), "Light");
    assert_eq!(ModelTier::DeepThinking.symbol(), "D");
}

#[test]
fn test_assignment_reason() {
    assert_eq!(AssignmentReason::HardwareLimit.name(), "Hardware Limit");
    assert_eq!(AssignmentReason::Default.name(), "Default");
}

#[test]
fn test_assign() {
    let mut tracker = LlmAssignmentTracker::new();
    tracker.assign(make_assignment("junior-desktop", "llama3.1:8b", ModelTier::Standard));

    assert_eq!(tracker.total_count(), 1);
    assert!(tracker.get_assignment("junior-desktop").is_some());
}

#[test]
fn test_reassign_deactivates_old() {
    let mut tracker = LlmAssignmentTracker::new();
    tracker.assign(make_assignment("junior-desktop", "llama3.1:8b", ModelTier::Standard));
    tracker.assign(make_assignment("junior-desktop", "llama3.2:3b", ModelTier::Light));

    assert_eq!(tracker.total_count(), 2);
    assert_eq!(tracker.active_count(), 1);
    let current = tracker.get_assignment("junior-desktop").unwrap();
    assert_eq!(current.model, "llama3.2:3b");
}

#[test]
fn test_add_available_model() {
    let mut tracker = LlmAssignmentTracker::new();
    tracker.add_available_model("llama3.1:8b".to_string());
    tracker.add_available_model("llama3.1:8b".to_string()); // duplicate

    assert_eq!(tracker.available_models.len(), 1);
    assert!(tracker.is_model_available("llama3.1:8b"));
}

#[test]
fn test_set_recommended_tier() {
    let mut tracker = LlmAssignmentTracker::new();
    tracker.set_recommended_tier(ModelTier::Heavy);

    assert_eq!(tracker.recommended_tier, Some(ModelTier::Heavy));
}

#[test]
fn test_by_tier() {
    let mut tracker = LlmAssignmentTracker::new();
    tracker.assign(make_assignment("junior", "llama3.2:3b", ModelTier::Light));
    tracker.assign(make_assignment("senior", "llama3.1:8b", ModelTier::Standard));

    assert_eq!(tracker.by_model_tier(ModelTier::Light).len(), 1);
    assert_eq!(tracker.by_model_tier(ModelTier::Standard).len(), 1);
}

#[test]
fn test_models_in_use() {
    let mut tracker = LlmAssignmentTracker::new();
    tracker.assign(make_assignment("junior", "llama3.2:3b", ModelTier::Light));
    tracker.assign(make_assignment("senior", "llama3.1:8b", ModelTier::Standard));

    let models = tracker.models_in_use();
    assert_eq!(models.len(), 2);
}

#[test]
fn test_get_model_tier() {
    assert_eq!(get_model_tier("llama3.2:1b"), ModelTier::Light);
    assert_eq!(get_model_tier("llama3.1:70b"), ModelTier::Heavy);
    assert_eq!(get_model_tier("deepseek-r1:8b"), ModelTier::DeepThinking);
}

#[test]
fn test_format_llm_tracker() {
    let mut tracker = LlmAssignmentTracker::new();
    tracker.assign(make_assignment("junior", "llama3.2:3b", ModelTier::Light));

    let output = format_llm_tracker(&tracker);
    assert!(output.contains("LLM Assignments"));
    assert!(output.contains("llama3.2:3b"));
}

#[test]
fn test_is_llm_query() {
    assert!(is_llm_query("which model is used?"));
    assert!(is_llm_query("show llm assignments"));
    assert!(is_llm_query("what model does the specialist use?"));
    assert!(!is_llm_query("what is the weather?"));
}

#[test]
fn test_llm_fun_fact() {
    let mut tracker = LlmAssignmentTracker::new();
    tracker.assign(make_assignment("junior", "llama3.2:3b", ModelTier::Light));

    let fact = llm_fun_fact(&tracker);
    assert!(!fact.is_empty());
}
