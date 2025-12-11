//! Recipe creation from specialist lessons (v0.0.401).
//!
//! Creates recipes from SpecialistLesson data, allowing Anna to remember
//! what she learned from escalations and LLM self-healing.

use crate::recipe::{compute_recipe_id, Recipe, RecipeAction, RecipeKind, RecipeSignature};
use crate::specialist_learning::SpecialistLesson;
use crate::teams::Team;

/// Result of attempting to learn from a specialist lesson
#[derive(Debug)]
pub struct SpecialistLearnResult {
    pub learned: bool,
    pub recipe_id: Option<String>,
    pub reason: Option<String>,
}

impl SpecialistLearnResult {
    fn learned(recipe_id: String) -> Self {
        Self { learned: true, recipe_id: Some(recipe_id), reason: None }
    }

    fn skipped(reason: impl Into<String>) -> Self {
        Self { learned: false, recipe_id: None, reason: Some(reason.into()) }
    }
}

/// Try to learn a recipe from specialist lesson.
/// Specialist lessons have higher base confidence since they come from escalation.
pub fn try_learn_from_specialist(lesson: &SpecialistLesson) -> SpecialistLearnResult {
    // Specialist lessons already passed confidence threshold
    if lesson.confidence < 70 {
        return SpecialistLearnResult::skipped("Specialist confidence too low");
    }

    // Validate query pattern
    if !is_valid_specialist_query(&lesson.query_pattern) {
        return SpecialistLearnResult::skipped("Invalid query pattern");
    }

    // Skip if no probes (nothing actionable)
    if lesson.effective_probes.is_empty() {
        return SpecialistLearnResult::skipped("No effective probes");
    }

    // Build signature - use generic pattern if available
    let signature = build_specialist_signature(lesson);
    let team = team_from_domain(&lesson.domain.to_string());
    let recipe_id = compute_recipe_id(&signature, team);

    // Check existing recipe
    if let Ok(mut existing) = Recipe::load(&recipe_id) {
        existing.success_count += lesson.success_count;
        existing.reliability_score = existing.reliability_score.max(lesson.confidence);
        let _ = existing.save();
        return SpecialistLearnResult::learned(recipe_id);
    }

    // Build new recipe from specialist lesson
    let recipe = build_specialist_recipe(lesson, signature, team, &recipe_id);
    match recipe.save() {
        Ok(()) => SpecialistLearnResult::learned(recipe_id),
        Err(e) => SpecialistLearnResult::skipped(format!("Save failed: {}", e)),
    }
}

/// Check if a query is valid for specialist learning
fn is_valid_specialist_query(query: &str) -> bool {
    let q = query.trim().to_lowercase();
    // Minimum length (8 chars) and word count (2 words)
    if q.len() < 8 { return false; }
    let word_count = q.split_whitespace().count();
    if word_count < 2 { return false; }
    // Reject test patterns
    let test_patterns = ["test-", "test_", "foo", "bar", "baz", "asdf"];
    for p in test_patterns {
        if q.contains(p) { return false; }
    }
    true
}

/// Build recipe signature from specialist lesson
fn build_specialist_signature(lesson: &SpecialistLesson) -> RecipeSignature {
    let query_pattern = if let Some(ref pattern) = lesson.generic_pattern {
        // Use generic pattern for wider matching
        pattern.answer_template.split_whitespace().take(5).collect::<Vec<_>>().join(" ")
    } else {
        lesson.query_pattern.to_lowercase().trim().to_string()
    };

    RecipeSignature {
        domain: lesson.domain.to_string(),
        intent: "question".to_string(),
        route_class: lesson.domain.to_string(),
        query_pattern,
    }
}

/// Build recipe from specialist lesson
fn build_specialist_recipe(
    lesson: &SpecialistLesson,
    signature: RecipeSignature,
    team: Team,
    recipe_id: &str,
) -> Recipe {
    let mut intent_tags = vec![lesson.domain.to_string()];
    for word in lesson.query_pattern.split_whitespace() {
        if word.len() > 3 {
            intent_tags.push(word.to_lowercase());
        }
    }

    // Add category tag if generic pattern exists
    if let Some(ref pattern) = lesson.generic_pattern {
        intent_tags.push(format!("{:?}", pattern.category).to_lowercase());
    }

    Recipe {
        id: recipe_id.to_string(),
        signature,
        team,
        risk_level: crate::ticket::RiskLevel::ReadOnly,
        required_evidence_kinds: vec![],
        probe_sequence: lesson.effective_probes.clone(),
        answer_template: lesson.answer_template.clone(),
        created_at: lesson.learned_at,
        success_count: lesson.success_count,
        reliability_score: lesson.confidence,
        kind: RecipeKind::Query,
        target: None,
        action: RecipeAction::None,
        rollback: None,
        clarification_slots: vec![],
        default_question_id: None,
        populates_facts: vec![],
        intent_tags,
        targets: vec![],
        preconditions: vec![],
        clarify_prereqs: vec![],
        negative_match_patterns: vec![],
    }
}

/// Map domain string to Team
fn team_from_domain(domain: &str) -> Team {
    match domain.to_lowercase().as_str() {
        "storage" | "disk" => Team::Storage,
        "memory" | "ram" => Team::Performance,
        "network" | "wifi" | "ethernet" => Team::Network,
        "performance" | "cpu" | "process" => Team::Performance,
        "service" | "services" | "systemd" | "system" => Team::Services,
        "security" | "firewall" | "permissions" => Team::Security,
        "hardware" | "audio" | "device" => Team::Hardware,
        "desktop" | "editor" | "gui" => Team::Desktop,
        "logs" | "journal" => Team::Logs,
        _ => Team::General,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe_learning::QueryCategory;
    use crate::rpc::SpecialistDomain;
    use crate::specialist_learning::SolutionType;

    fn mock_lesson(confidence: u8) -> SpecialistLesson {
        SpecialistLesson {
            id: "test-lesson".to_string(),
            query_pattern: "check hyprland config".to_string(),
            domain: SpecialistDomain::System,
            category: QueryCategory::General,
            issues_fixed: vec![],
            solution_type: SolutionType::LlmSelfHealing {
                correction_type: "test".to_string(),
            },
            effective_probes: vec!["cat ~/.config/hyprland/hyprland.conf".to_string()],
            answer_template: "Your hyprland config is at ~/.config/hyprland/".to_string(),
            confidence,
            success_count: 1,
            learned_at: 0,
            last_success_at: 0,
            generic_pattern: None,
        }
    }

    #[test]
    fn test_specialist_high_confidence() {
        let lesson = mock_lesson(85);
        let result = try_learn_from_specialist(&lesson);
        assert!(result.learned, "Should learn: {:?}", result.reason);
    }

    #[test]
    fn test_specialist_low_confidence() {
        let lesson = mock_lesson(50);
        let result = try_learn_from_specialist(&lesson);
        assert!(!result.learned);
        assert!(result.reason.unwrap().contains("confidence"));
    }

    #[test]
    fn test_valid_specialist_query() {
        assert!(is_valid_specialist_query("check hyprland config"));
        assert!(is_valid_specialist_query("what is using disk space"));
        assert!(!is_valid_specialist_query("test-123"));
        assert!(!is_valid_specialist_query("x"));
    }
}
