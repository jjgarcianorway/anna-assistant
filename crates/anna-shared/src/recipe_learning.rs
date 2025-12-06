//! Recipe learning from successful queries (v0.0.94).
//!
//! Learns recipes from verified, high-reliability query outcomes.
//! Only persists when: verified AND reliability_score >= 80.

use crate::recipe::{
    compute_recipe_id, should_persist_recipe, Recipe, RecipeAction, RecipeKind,
    RecipeSignature,
};
use crate::rpc::ServiceDeskResult;
use crate::teams::Team;
use crate::trace::EvidenceKind;

/// Result of attempting to learn from a query outcome
#[derive(Debug)]
pub struct LearnResult {
    /// Whether a recipe was learned
    pub learned: bool,
    /// Recipe ID if learned
    pub recipe_id: Option<String>,
    /// Reason if not learned
    pub reason: Option<String>,
}

impl LearnResult {
    fn learned(recipe_id: String) -> Self {
        Self {
            learned: true,
            recipe_id: Some(recipe_id),
            reason: None,
        }
    }

    fn skipped(reason: impl Into<String>) -> Self {
        Self {
            learned: false,
            recipe_id: None,
            reason: Some(reason.into()),
        }
    }
}

/// Try to learn a recipe from a service desk result.
///
/// Only learns if:
/// - answer_grounded is true (verified)
/// - reliability_score >= 80
/// - There's a meaningful pattern to learn
pub fn try_learn_from_result(result: &ServiceDeskResult) -> LearnResult {
    // Check persistence gate
    let verified = result.reliability_signals.answer_grounded;
    let score = result.reliability_score;

    if !should_persist_recipe(verified, score) {
        return LearnResult::skipped(format!(
            "Not verified or score too low (verified={}, score={})",
            verified, score
        ));
    }

    // Skip if no answer
    if result.answer.trim().is_empty() {
        return LearnResult::skipped("Empty answer");
    }

    // Skip if clarification needed (incomplete pattern)
    if result.needs_clarification {
        return LearnResult::skipped("Clarification needed");
    }

    // Skip if no probes were executed (nothing to learn)
    if result.evidence.probes_executed.is_empty() {
        return LearnResult::skipped("No probes executed");
    }

    // Build signature from result
    let signature = build_signature(result);

    // Determine team from domain
    let team = team_from_domain(&result.domain.to_string());

    // Check if recipe already exists
    let recipe_id = compute_recipe_id(&signature, team);
    if let Ok(existing) = Recipe::load(&recipe_id) {
        // Update success count instead of creating new
        let mut updated = existing;
        updated.success_count += 1;
        updated.reliability_score = score;
        if updated.save().is_err() {
            // Silently fail - not critical
        }
        return LearnResult::learned(recipe_id);
    }

    // Build new recipe
    let recipe = build_recipe(result, signature, team, &recipe_id);

    // Save recipe
    match recipe.save() {
        Ok(()) => LearnResult::learned(recipe_id),
        Err(e) => LearnResult::skipped(format!("Save failed: {}", e)),
    }
}

/// Build recipe signature from result
fn build_signature(result: &ServiceDeskResult) -> RecipeSignature {
    // Extract query from transcript
    let query = extract_user_query(result);

    // Normalize query to pattern (lowercase, trim)
    let query_pattern = normalize_query(&query);

    // Use domain as route class (simplified)
    let route_class = result.domain.to_string();

    RecipeSignature {
        domain: result.domain.to_string(),
        intent: "question".to_string(), // Default; could be extracted from ticket
        route_class,
        query_pattern,
    }
}

/// Extract user query from result transcript
fn extract_user_query(result: &ServiceDeskResult) -> String {
    use crate::transcript::{Actor, TranscriptEventKind};

    for event in &result.transcript.events {
        if let TranscriptEventKind::Message { text } = &event.kind {
            if event.from == Actor::You {
                return text.clone();
            }
        }
    }

    // Fallback to request_id prefix if no query found
    result.request_id.clone()
}

/// Normalize query to a matchable pattern
fn normalize_query(query: &str) -> String {
    query
        .to_lowercase()
        .trim()
        .to_string()
}

/// Map domain string to Team
fn team_from_domain(domain: &str) -> Team {
    match domain.to_lowercase().as_str() {
        "storage" | "disk" => Team::Storage,
        "memory" | "ram" => Team::Performance,
        "network" | "wifi" | "ethernet" => Team::Network,
        "performance" | "cpu" | "process" => Team::Performance,
        "service" | "services" | "systemd" => Team::Services,
        "security" | "firewall" | "permissions" => Team::Security,
        "hardware" | "audio" | "device" => Team::Hardware,
        "desktop" | "editor" | "gui" => Team::Desktop,
        "logs" | "journal" => Team::Logs,
        _ => Team::General,
    }
}

/// Build a recipe from service desk result
fn build_recipe(
    result: &ServiceDeskResult,
    signature: RecipeSignature,
    team: Team,
    recipe_id: &str,
) -> Recipe {
    // Extract evidence kinds from execution trace
    let evidence_kinds: Vec<EvidenceKind> = result
        .execution_trace
        .as_ref()
        .map(|t| t.evidence_kinds.clone())
        .unwrap_or_default();

    // Extract probe commands
    let probe_sequence: Vec<String> = result
        .evidence
        .probes_executed
        .iter()
        .map(|p| p.command.clone())
        .collect();

    // Build intent tags from domain and keywords
    let mut intent_tags = vec![result.domain.to_string()];
    for word in signature.query_pattern.split_whitespace() {
        if word.len() > 3 {
            intent_tags.push(word.to_string());
        }
    }

    Recipe {
        id: recipe_id.to_string(),
        signature,
        team,
        risk_level: crate::ticket::RiskLevel::ReadOnly,
        required_evidence_kinds: evidence_kinds,
        probe_sequence,
        answer_template: result.answer.clone(),
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        success_count: 1,
        reliability_score: result.reliability_score,
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::{EvidenceBlock, ProbeResult, ReliabilitySignals, SpecialistDomain};
    use crate::transcript::Transcript;

    fn mock_result(verified: bool, score: u8) -> ServiceDeskResult {
        ServiceDeskResult {
            request_id: "test-123".to_string(),
            case_number: None,
            assigned_staff: None,
            answer: "Your disk is 50% full.".to_string(),
            domain: SpecialistDomain::Storage,
            reliability_score: score,
            reliability_signals: ReliabilitySignals {
                translator_confident: true,
                probe_coverage: true,
                answer_grounded: verified,
                no_invention: true,
                clarification_not_needed: true,
            },
            evidence: {
                let mut eb = EvidenceBlock::default();
                eb.probes_executed = vec![ProbeResult {
                    command: "df -h".to_string(),
                    stdout: "/dev/sda1 50G 25G 50%".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                    timing_ms: 100,
                }];
                eb
            },
            transcript: Transcript::new(),
            needs_clarification: false,
            clarification_question: None,
            clarification_request: None,
            reliability_explanation: None,
            execution_trace: None,
            proposed_change: None,
            feedback_request: None,
        }
    }

    #[test]
    fn test_learn_verified_high_score() {
        let result = mock_result(true, 85);
        let learn = try_learn_from_result(&result);
        assert!(learn.learned);
        assert!(learn.recipe_id.is_some());
    }

    #[test]
    fn test_skip_unverified() {
        let result = mock_result(false, 85);
        let learn = try_learn_from_result(&result);
        assert!(!learn.learned);
        assert!(learn.reason.unwrap().contains("verified"));
    }

    #[test]
    fn test_skip_low_score() {
        let result = mock_result(true, 60);
        let learn = try_learn_from_result(&result);
        assert!(!learn.learned);
        assert!(learn.reason.unwrap().contains("score"));
    }

    #[test]
    fn test_team_from_domain() {
        assert_eq!(team_from_domain("storage"), Team::Storage);
        assert_eq!(team_from_domain("network"), Team::Network);
        assert_eq!(team_from_domain("desktop"), Team::Desktop);
        assert_eq!(team_from_domain("unknown"), Team::General);
    }
}
