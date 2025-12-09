//! Recipe-based fast path for queries (v0.0.101, v0.0.102: direct answers).
//! Checks recipe index BEFORE LLM translator. High-confidence matches skip LLM entirely.
//! v0.0.163: Built-in recipe matchers extracted to separate module.

use anna_shared::recipe::{Recipe, RecipeKind};
use anna_shared::recipe_index::RecipeIndex;
use anna_shared::recipe_matcher::{match_recipe, MatchResult};
use anna_shared::rpc::{
    EvidenceBlock, QueryIntent, ReliabilitySignals, ServiceDeskResult, SpecialistDomain,
    TranslatorTicket,
};
use anna_shared::trace::{ExecutionTrace, ProbeStats};
use anna_shared::transcript::Transcript;
use tracing::info;

// Re-export built-in recipe matchers
pub use crate::recipe_builtins::{
    check_cron_recipes, check_docker_recipes, check_git_recipes, check_shell_recipes,
    check_ssh_recipes, check_systemd_recipes,
};

/// Minimum score to skip LLM and use recipe directly
const RECIPE_SKIP_LLM_THRESHOLD: u32 = 70;

/// Result of recipe fast path check
#[derive(Debug)]
pub struct RecipeFastPathResult {
    /// Whether a recipe was matched
    pub matched: bool,
    /// The ticket to use (if matched)
    pub ticket: Option<TranslatorTicket>,
    /// The recipe that was matched
    pub recipe: Option<Recipe>,
    /// Match score
    pub score: u32,
    /// Matched tokens
    pub matched_tokens: Vec<String>,
    /// Whether we can skip the LLM
    pub skip_llm: bool,
}

impl RecipeFastPathResult {
    fn no_match() -> Self {
        Self {
            matched: false,
            ticket: None,
            recipe: None,
            score: 0,
            matched_tokens: vec![],
            skip_llm: false,
        }
    }

    fn from_recipe(result: MatchResult) -> Self {
        let skip_llm = result.score >= RECIPE_SKIP_LLM_THRESHOLD && result.high_confidence;
        let ticket = if skip_llm {
            Some(ticket_from_recipe(&result.recipe))
        } else {
            None
        };

        Self {
            matched: true,
            ticket,
            recipe: Some(result.recipe),
            score: result.score,
            matched_tokens: result.matched_tokens,
            skip_llm,
        }
    }
}

/// Check recipe index for a matching recipe
pub fn check_recipe_fast_path(query: &str, index: &RecipeIndex) -> RecipeFastPathResult {
    // First, try the general recipe matcher
    if let Some(result) = match_recipe(query, index) {
        info!(
            "Recipe match found: score={}, tokens={:?}, skip_llm={}",
            result.score,
            result.matched_tokens,
            result.score >= RECIPE_SKIP_LLM_THRESHOLD && result.high_confidence
        );
        return RecipeFastPathResult::from_recipe(result);
    }

    // Second, check built-in shell recipes
    if let Some(result) = check_shell_recipes(query) {
        info!(
            "Shell recipe match found: {}",
            result
                .recipe
                .as_ref()
                .map(|r| r.id.clone())
                .unwrap_or_default()
        );
        return result;
    }

    // Third, check built-in git recipes
    if let Some(result) = check_git_recipes(query) {
        info!("Git recipe match found");
        return result;
    }

    // Fourth, check built-in SSH recipes (v0.0.104)
    if let Some(result) = check_ssh_recipes(query) {
        info!(
            "SSH recipe match found: {}",
            result
                .recipe
                .as_ref()
                .map(|r| r.id.clone())
                .unwrap_or_default()
        );
        return result;
    }

    // Fifth, check built-in systemd recipes (v0.0.233)
    if let Some(result) = check_systemd_recipes(query) {
        info!(
            "Systemd recipe match found: {}",
            result
                .recipe
                .as_ref()
                .map(|r| r.id.clone())
                .unwrap_or_default()
        );
        return result;
    }

    // Sixth, check built-in cron recipes (v0.0.234)
    if let Some(result) = check_cron_recipes(query) {
        info!(
            "Cron recipe match found: {}",
            result
                .recipe
                .as_ref()
                .map(|r| r.id.clone())
                .unwrap_or_default()
        );
        return result;
    }

    // Seventh, check built-in Docker recipes (v0.0.235)
    if let Some(result) = check_docker_recipes(query) {
        info!(
            "Docker recipe match found: {}",
            result
                .recipe
                .as_ref()
                .map(|r| r.id.clone())
                .unwrap_or_default()
        );
        return result;
    }

    RecipeFastPathResult::no_match()
}

/// Map recipe team to specialist domain
pub fn team_to_domain(team: &anna_shared::teams::Team) -> SpecialistDomain {
    match team {
        anna_shared::teams::Team::Network => SpecialistDomain::Network,
        anna_shared::teams::Team::Storage => SpecialistDomain::Storage,
        anna_shared::teams::Team::Security => SpecialistDomain::Security,
        _ => SpecialistDomain::System,
    }
}

/// Create a TranslatorTicket from a recipe
pub fn ticket_from_recipe(recipe: &Recipe) -> TranslatorTicket {
    let intent = match recipe.kind {
        RecipeKind::Query => QueryIntent::Question,
        _ => QueryIntent::Request,
    };

    TranslatorTicket {
        intent,
        domain: team_to_domain(&recipe.team),
        entities: recipe.targets.clone(),
        needs_probes: recipe.probe_sequence.clone(),
        clarification_question: None,
        confidence: (recipe.reliability_score as f32) / 100.0,
        answer_contract: None,
    }
}

/// v0.0.102: Build a ServiceDeskResult directly from a recipe
pub fn build_recipe_result(
    request_id: String,
    recipe: &Recipe,
    matched_tokens: &[String],
    transcript: Transcript,
) -> ServiceDeskResult {
    let signals = ReliabilitySignals {
        translator_confident: true,
        probe_coverage: true,
        answer_grounded: true,
        no_invention: true,
        clarification_not_needed: true,
    };
    let trace = ExecutionTrace::deterministic_route(
        &format!("recipe:{}", recipe.id),
        ProbeStats::default(),
        vec![],
    );
    let answer = format!(
        "{}\n\n*Recipe: {} (matched: {})*",
        recipe.answer_template,
        recipe.id,
        matched_tokens.join(", ")
    );

    // v0.0.103: Ask for feedback if recipe confidence is borderline (60-75)
    // or if recipe is new (success_count < 3)
    let feedback_request = if recipe.reliability_score >= 60 && recipe.reliability_score <= 75 {
        Some(
            anna_shared::recipe_feedback::FeedbackRequest::borderline_confidence(
                &recipe.id,
                recipe.reliability_score,
            ),
        )
    } else if recipe.success_count < 3 {
        Some(anna_shared::recipe_feedback::FeedbackRequest::new_recipe(
            &recipe.id,
        ))
    } else {
        None
    };

    ServiceDeskResult {
        request_id,
        case_number: None,
        assigned_staff: None,
        staff_id: None,
        answer,
        reliability_score: recipe.reliability_score,
        reliability_signals: signals,
        reliability_explanation: None,
        domain: team_to_domain(&recipe.team),
        evidence: EvidenceBlock::default(),
        needs_clarification: false,
        clarification_question: None,
        clarification_request: None,
        transcript,
        execution_trace: Some(trace),
        proposed_change: None,
        proposed_changes: Vec::new(),
        feedback_request,
    }
}

/// Check if a recipe result can provide a direct answer (has answer_template)
pub fn can_answer_directly(result: &RecipeFastPathResult) -> bool {
    result.skip_llm
        && result
            .recipe
            .as_ref()
            .map(|r| !r.answer_template.is_empty())
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_match_empty_index() {
        let index = RecipeIndex::new();
        let result = check_recipe_fast_path("random query", &index);
        assert!(!result.matched);
    }

    #[test]
    fn test_shell_recipe_match() {
        let index = RecipeIndex::new();
        // Use "zsh" because syntax highlighting recipe exists only for zsh
        let result = check_recipe_fast_path("enable syntax highlighting in zsh", &index);
        // Should match built-in shell recipe
        assert!(result.matched);
        assert!(result.skip_llm);
    }

    #[test]
    fn test_shell_recipe_match_bash_color() {
        let index = RecipeIndex::new();
        // Bash has colored prompt recipe
        let result = check_recipe_fast_path("enable colored prompt in bash", &index);
        assert!(result.matched);
        assert!(result.skip_llm);
    }

    #[test]
    fn test_git_recipe_match() {
        let index = RecipeIndex::new();
        let result = check_recipe_fast_path("configure git aliases", &index);
        assert!(result.matched);
        assert!(result.skip_llm);
    }

    #[test]
    fn test_git_recipe_no_match_without_git() {
        let index = RecipeIndex::new();
        let result = check_recipe_fast_path("configure aliases", &index);
        // Should not match git recipes without "git" in query
        // (might match other recipes though)
        if result.matched {
            assert!(result
                .recipe
                .as_ref()
                .map(|r| !r.id.starts_with("git"))
                .unwrap_or(true));
        }
    }

    #[test]
    fn test_can_answer_directly_with_template() {
        let index = RecipeIndex::new();
        let result = check_recipe_fast_path("enable syntax highlighting in zsh", &index);
        // Should be able to answer directly (has answer_template)
        assert!(can_answer_directly(&result));
    }

    #[test]
    fn test_build_recipe_result() {
        let index = RecipeIndex::new();
        let result = check_recipe_fast_path("enable colored prompt in bash", &index);
        assert!(result.matched);

        let recipe = result.recipe.as_ref().unwrap();
        let transcript = Transcript::new();
        let service_result = build_recipe_result(
            "test-123".to_string(),
            recipe,
            &result.matched_tokens,
            transcript,
        );

        // Verify the result
        assert_eq!(service_result.request_id, "test-123");
        assert!(service_result.answer.contains("PS1")); // Colored prompt has PS1
        assert!(service_result.reliability_score >= 90);
        assert!(service_result.execution_trace.is_some());
        // Recipe answers are deterministic
        assert!(
            service_result
                .execution_trace
                .as_ref()
                .unwrap()
                .answer_is_deterministic
        );
    }

    #[test]
    fn test_ssh_recipe_match_generate_key() {
        let index = RecipeIndex::new();
        let result = check_recipe_fast_path("how do I generate an ssh key", &index);
        assert!(result.matched);
        assert!(result.skip_llm);
        assert!(result
            .recipe
            .as_ref()
            .unwrap()
            .answer_template
            .contains("ssh-keygen"));
    }

    #[test]
    fn test_ssh_recipe_match_github() {
        let index = RecipeIndex::new();
        let result = check_recipe_fast_path("setup ssh for github", &index);
        assert!(result.matched);
        assert!(result.skip_llm);
        assert!(result
            .recipe
            .as_ref()
            .unwrap()
            .answer_template
            .contains("github"));
    }

    #[test]
    fn test_ssh_recipe_match_copy_key() {
        let index = RecipeIndex::new();
        let result = check_recipe_fast_path("ssh copy key to server", &index);
        assert!(result.matched);
        assert!(result.skip_llm);
        assert!(result
            .recipe
            .as_ref()
            .unwrap()
            .answer_template
            .contains("ssh-copy-id"));
    }
}
