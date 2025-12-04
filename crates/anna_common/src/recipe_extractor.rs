//! Recipe Extractor v0.0.80 - Extract reusable recipes from successful cases
//!
//! Gate rules:
//! - reliability >= 80% for read-only recipes
//! - reliability >= 95% for mutation recipes (v0.0.80)
//! - >= 2 evidence items collected (>= 3 for mutations)
//! - successful outcome
//! - user did not reject recipe creation

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::case_engine::{CaseState, IntentType};
use crate::case_file_v1::CaseFileV1;
use crate::recipes::{
    IntentPattern, Recipe, RecipeRiskLevel, RecipeStatus, RecipeToolPlan, RecipeToolStep,
};

/// Minimum reliability score required for recipe extraction
pub const MIN_RELIABILITY_FOR_RECIPE: u8 = 80;

/// Minimum reliability score required for mutation recipe extraction (v0.0.80)
pub const MIN_RELIABILITY_FOR_MUTATION_RECIPE: u8 = 95;

/// Minimum evidence items required
pub const MIN_EVIDENCE_FOR_RECIPE: usize = 2;

/// Minimum evidence items required for mutation recipes (v0.0.80)
pub const MIN_EVIDENCE_FOR_MUTATION_RECIPE: usize = 3;

// ============================================================================
// Recipe Extraction Result
// ============================================================================

/// Result of attempting to extract a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeExtractionResult {
    /// Whether extraction succeeded
    pub extracted: bool,
    /// Recipe ID if extracted
    pub recipe_id: Option<String>,
    /// Reason for extraction or rejection
    pub reason: String,
    /// XP gained from this extraction
    pub xp_gained: u64,
}

// ============================================================================
// Gate Check
// ============================================================================

/// Check if a case qualifies for recipe extraction
pub fn check_recipe_gate(case: &CaseFileV1) -> (bool, String) {
    // Gate 1: Must be successful
    if !case.success {
        return (false, "Case was not successful".to_string());
    }

    // Gate 2: Reliability >= 80%
    if case.reliability_score < MIN_RELIABILITY_FOR_RECIPE {
        return (
            false,
            format!(
                "Reliability {} < {} required",
                case.reliability_score, MIN_RELIABILITY_FOR_RECIPE
            ),
        );
    }

    // Gate 3: >= 2 evidence items
    if case.evidence_count < MIN_EVIDENCE_FOR_RECIPE {
        return (
            false,
            format!(
                "Evidence count {} < {} required",
                case.evidence_count, MIN_EVIDENCE_FOR_RECIPE
            ),
        );
    }

    // Gate 4: Must be SYSTEM_QUERY or DIAGNOSE (not HOWTO/META)
    match case.intent {
        IntentType::SystemQuery | IntentType::Diagnose => {}
        _ => {
            return (
                false,
                format!("Intent {} not eligible for recipes", case.intent),
            );
        }
    }

    (true, "Passed all gates".to_string())
}

/// Check if a case qualifies for mutation recipe extraction (v0.0.80)
/// Stricter requirements: 95% reliability, 3+ evidence items
pub fn check_mutation_recipe_gate(
    case: &CaseFileV1,
    mutation_succeeded: bool,
    rollback_occurred: bool,
) -> (bool, String) {
    // Gate 1: Must be successful case AND mutation must have succeeded
    if !case.success {
        return (false, "Case was not successful".to_string());
    }
    if !mutation_succeeded {
        return (false, "Mutation did not succeed".to_string());
    }
    if rollback_occurred {
        return (false, "Mutation was rolled back - not eligible".to_string());
    }

    // Gate 2: Reliability >= 95% for mutations (stricter than read-only)
    if case.reliability_score < MIN_RELIABILITY_FOR_MUTATION_RECIPE {
        return (
            false,
            format!(
                "Reliability {} < {}% required for mutation recipes",
                case.reliability_score, MIN_RELIABILITY_FOR_MUTATION_RECIPE
            ),
        );
    }

    // Gate 3: >= 3 evidence items for mutations (stricter)
    if case.evidence_count < MIN_EVIDENCE_FOR_MUTATION_RECIPE {
        return (
            false,
            format!(
                "Evidence count {} < {} required for mutation recipes",
                case.evidence_count, MIN_EVIDENCE_FOR_MUTATION_RECIPE
            ),
        );
    }

    // Gate 4: Must be ACTION_REQUEST intent for mutations
    match case.intent {
        IntentType::ActionRequest | IntentType::Diagnose => {}
        _ => {
            return (
                false,
                format!("Intent {} not eligible for mutation recipes", case.intent),
            );
        }
    }

    (true, "Passed all mutation recipe gates".to_string())
}

/// Mutation recipe extraction result (v0.0.80)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationRecipeExtractionResult {
    /// Whether extraction succeeded
    pub extracted: bool,
    /// Recipe ID if extracted
    pub recipe_id: Option<String>,
    /// Reason for extraction or rejection
    pub reason: String,
    /// Risk level of the mutation
    pub risk_level: RecipeRiskLevel,
    /// XP gained from this extraction
    pub xp_gained: u64,
}

/// Extract a mutation recipe from a successful mutation case (v0.0.80)
pub fn extract_mutation_recipe(
    case: &CaseFileV1,
    mutation_type: &str,
    mutation_target: &str,
    mutation_succeeded: bool,
    rollback_occurred: bool,
) -> MutationRecipeExtractionResult {
    // Check gates
    let (passed, reason) = check_mutation_recipe_gate(case, mutation_succeeded, rollback_occurred);
    if !passed {
        return MutationRecipeExtractionResult {
            extracted: false,
            recipe_id: None,
            reason,
            risk_level: RecipeRiskLevel::HighRisk,
            xp_gained: 0,
        };
    }

    // Generate mutation recipe ID
    let recipe_id = generate_mutation_recipe_id(mutation_type, mutation_target);

    // Determine risk level based on mutation type
    let risk_level = match mutation_type {
        "service_restart" | "service_start" => RecipeRiskLevel::LowRisk,
        "package_install" | "service_enable" => RecipeRiskLevel::MediumRisk,
        "config_edit" | "service_stop" | "service_disable" | "package_remove" => {
            RecipeRiskLevel::HighRisk
        }
        _ => RecipeRiskLevel::MediumRisk,
    };

    // XP for mutation recipes (higher due to stricter requirements)
    let xp_gained = match risk_level {
        RecipeRiskLevel::ReadOnly => 50,
        RecipeRiskLevel::LowRisk => 100,
        RecipeRiskLevel::MediumRisk => 150,
        RecipeRiskLevel::HighRisk => 200,
    };

    MutationRecipeExtractionResult {
        extracted: true,
        recipe_id: Some(recipe_id),
        reason: format!(
            "Mutation recipe extracted: {} {} with {}% reliability",
            mutation_type, mutation_target, case.reliability_score
        ),
        risk_level,
        xp_gained,
    }
}

/// Generate a recipe ID for mutation recipes
fn generate_mutation_recipe_id(mutation_type: &str, target: &str) -> String {
    let prefix = match mutation_type {
        s if s.starts_with("service") => "svc",
        s if s.starts_with("package") => "pkg",
        s if s.starts_with("config") => "cfg",
        _ => "mut",
    };

    let target_slug: String = target
        .chars()
        .filter(|c| c.is_alphanumeric())
        .take(15)
        .collect();

    let timestamp = Utc::now().format("%Y%m%d%H%M%S");
    format!("{}-{}-{}", prefix, target_slug, timestamp)
}

/// Check if a CaseState qualifies for recipe extraction
pub fn check_state_recipe_gate(state: &CaseState) -> (bool, String) {
    // Gate 1: Must be successful
    if !state.success {
        return (false, "Case was not successful".to_string());
    }

    // Gate 2: Reliability >= 80%
    if let Some(reliability) = state.reliability_score {
        if reliability < MIN_RELIABILITY_FOR_RECIPE {
            return (
                false,
                format!(
                    "Reliability {} < {} required",
                    reliability, MIN_RELIABILITY_FOR_RECIPE
                ),
            );
        }
    } else {
        return (false, "No reliability score".to_string());
    }

    // Gate 3: >= 2 evidence items
    if state.evidence_ids.len() < MIN_EVIDENCE_FOR_RECIPE {
        return (
            false,
            format!(
                "Evidence count {} < {} required",
                state.evidence_ids.len(),
                MIN_EVIDENCE_FOR_RECIPE
            ),
        );
    }

    // Gate 4: Must have classified intent
    if let Some(intent) = &state.intent {
        match intent {
            IntentType::SystemQuery | IntentType::Diagnose => {}
            _ => {
                return (false, format!("Intent {} not eligible for recipes", intent));
            }
        }
    } else {
        return (false, "No classified intent".to_string());
    }

    (true, "Passed all gates".to_string())
}

// ============================================================================
// Recipe Extraction
// ============================================================================

/// Extract a recipe from a successful case
pub fn extract_recipe(case: &CaseFileV1) -> RecipeExtractionResult {
    // Check gates
    let (passed, reason) = check_recipe_gate(case);
    if !passed {
        return RecipeExtractionResult {
            extracted: false,
            recipe_id: None,
            reason,
            xp_gained: 0,
        };
    }

    // Generate recipe ID
    let recipe_id = generate_recipe_id(&case.intent, &case.request);

    // Build intent pattern from the request
    let keywords = extract_keywords(&case.request);
    let _intent_pattern = IntentPattern {
        intent_type: case.intent.to_string().to_lowercase(),
        keywords: keywords.clone(),
        targets: vec![],
        negative_keywords: vec![],
        examples: vec![case.request.clone()],
    };

    // Build tool plan from evidence
    let tool_plan = build_tool_plan(case);

    // Determine recipe status based on reliability
    let status = if case.reliability_score >= 90 {
        RecipeStatus::Active
    } else {
        RecipeStatus::Draft
    };

    // Calculate XP
    let xp_gained = if status == RecipeStatus::Active {
        100
    } else {
        50
    };

    RecipeExtractionResult {
        extracted: true,
        recipe_id: Some(recipe_id),
        reason: format!(
            "Recipe extracted with status {:?}, reliability {}%",
            status, case.reliability_score
        ),
        xp_gained,
    }
}

/// Generate a recipe ID from intent and request
fn generate_recipe_id(intent: &IntentType, request: &str) -> String {
    let intent_prefix = match intent {
        IntentType::SystemQuery => "sq",
        IntentType::Diagnose => "dx",
        IntentType::ActionRequest => "ar",
        IntentType::Howto => "ht",
        IntentType::Meta => "mt",
    };

    // Extract first meaningful word from request
    let word = request
        .to_lowercase()
        .split_whitespace()
        .find(|w| {
            w.len() > 3 && !["what", "how", "is", "the", "my", "do", "can", "why"].contains(w)
        })
        .unwrap_or("generic")
        .chars()
        .filter(|c| c.is_alphanumeric())
        .take(10)
        .collect::<String>();

    let timestamp = Utc::now().format("%Y%m%d%H%M%S");
    format!("{}-{}-{}", intent_prefix, word, timestamp)
}

/// Extract keywords from request for intent matching
fn extract_keywords(request: &str) -> Vec<String> {
    let stop_words = [
        "what", "how", "is", "the", "my", "do", "can", "i", "a", "an", "have", "does", "am",
    ];

    request
        .to_lowercase()
        .split_whitespace()
        .filter(|w| !stop_words.contains(w) && w.len() > 2)
        .take(5)
        .map(|w| w.chars().filter(|c| c.is_alphanumeric()).collect())
        .collect()
}

/// Build tool plan from case evidence
fn build_tool_plan(case: &CaseFileV1) -> RecipeToolPlan {
    let steps: Vec<RecipeToolStep> = case
        .evidence
        .iter()
        .map(|e| RecipeToolStep {
            tool_name: e.tool_name.clone(),
            parameters: std::collections::HashMap::new(),
            is_mutation: false,
            description: e.summary.clone(),
            condition: None,
        })
        .collect();

    RecipeToolPlan {
        description: format!("Tool plan for {}", case.intent),
        steps,
        has_mutations: false,
    }
}

// ============================================================================
// XP Calculation
// ============================================================================

/// Calculate XP for a case based on outcome
pub fn calculate_case_xp(reliability: u8, success: bool, recipe_extracted: bool) -> u64 {
    if !success {
        return 0;
    }

    let mut xp: u64 = 0;

    // Base XP for success
    xp += 10;

    // Bonus for high reliability
    if reliability >= 90 {
        xp += 50;
    } else if reliability >= 85 {
        xp += 30;
    } else if reliability >= 80 {
        xp += 20;
    }

    // Bonus for recipe extraction
    if recipe_extracted {
        xp += 100;
    }

    xp
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_success_required() {
        let mut case = CaseFileV1::new("test-1", "test request");
        case.success = false;
        case.reliability_score = 90;
        case.evidence_count = 3;
        case.intent = IntentType::SystemQuery;

        let (passed, _) = check_recipe_gate(&case);
        assert!(!passed);
    }

    #[test]
    fn test_gate_reliability_required() {
        let mut case = CaseFileV1::new("test-2", "test request");
        case.success = true;
        case.reliability_score = 75;
        case.evidence_count = 3;
        case.intent = IntentType::SystemQuery;

        let (passed, reason) = check_recipe_gate(&case);
        assert!(!passed);
        assert!(reason.contains("Reliability"));
    }

    #[test]
    fn test_gate_evidence_required() {
        let mut case = CaseFileV1::new("test-3", "test request");
        case.success = true;
        case.reliability_score = 90;
        case.evidence_count = 1;
        case.intent = IntentType::SystemQuery;

        let (passed, reason) = check_recipe_gate(&case);
        assert!(!passed);
        assert!(reason.contains("Evidence"));
    }

    #[test]
    fn test_gate_passes() {
        let mut case = CaseFileV1::new("test-4", "what cpu do I have");
        case.success = true;
        case.reliability_score = 85;
        case.evidence_count = 2;
        case.intent = IntentType::SystemQuery;

        let (passed, _) = check_recipe_gate(&case);
        assert!(passed);
    }

    #[test]
    fn test_extract_keywords() {
        let keywords = extract_keywords("what cpu do I have");
        assert!(keywords.contains(&"cpu".to_string()));
        assert!(!keywords.contains(&"what".to_string()));
    }

    #[test]
    fn test_calculate_xp() {
        assert_eq!(calculate_case_xp(90, true, true), 160); // 10 + 50 + 100
        assert_eq!(calculate_case_xp(85, true, false), 40); // 10 + 30
        assert_eq!(calculate_case_xp(90, false, false), 0);
    }

    // v0.0.80: Mutation recipe tests
    #[test]
    fn test_mutation_gate_reliability_95() {
        let mut case = CaseFileV1::new("test-mut-1", "restart docker");
        case.success = true;
        case.reliability_score = 94; // Just below threshold
        case.evidence_count = 3;
        case.intent = IntentType::ActionRequest;

        let (passed, reason) = check_mutation_recipe_gate(&case, true, false);
        assert!(!passed);
        assert!(reason.contains("95%"));
    }

    #[test]
    fn test_mutation_gate_rollback_blocks() {
        let mut case = CaseFileV1::new("test-mut-2", "restart docker");
        case.success = true;
        case.reliability_score = 98;
        case.evidence_count = 3;
        case.intent = IntentType::ActionRequest;

        let (passed, reason) = check_mutation_recipe_gate(&case, true, true);
        assert!(!passed);
        assert!(reason.contains("rolled back"));
    }

    #[test]
    fn test_mutation_gate_passes() {
        let mut case = CaseFileV1::new("test-mut-3", "restart docker");
        case.success = true;
        case.reliability_score = 98;
        case.evidence_count = 4;
        case.intent = IntentType::ActionRequest;

        let (passed, _) = check_mutation_recipe_gate(&case, true, false);
        assert!(passed);
    }

    #[test]
    fn test_mutation_recipe_extraction() {
        let mut case = CaseFileV1::new("test-mut-4", "restart docker");
        case.success = true;
        case.reliability_score = 98;
        case.evidence_count = 4;
        case.intent = IntentType::ActionRequest;

        let result = extract_mutation_recipe(&case, "service_restart", "docker", true, false);
        assert!(result.extracted);
        assert!(result.recipe_id.is_some());
        assert_eq!(result.risk_level, RecipeRiskLevel::LowRisk);
        assert!(result.xp_gained > 0);
    }

    #[test]
    fn test_mutation_recipe_id_generation() {
        let id = generate_mutation_recipe_id("service_restart", "NetworkManager");
        assert!(id.starts_with("svc-"));
        assert!(id.contains("NetworkManager"));

        let id = generate_mutation_recipe_id("package_install", "vim");
        assert!(id.starts_with("pkg-"));

        let id = generate_mutation_recipe_id("config_edit", "/etc/pacman.conf");
        assert!(id.starts_with("cfg-"));
    }
}
