//! Recipe-based fast path for queries (v0.0.101, v0.0.102: direct answers).
//! Checks recipe index BEFORE LLM translator. High-confidence matches skip LLM entirely.

use anna_shared::recipe::{Recipe, RecipeKind, RecipeAction};
use anna_shared::recipe_index::RecipeIndex;
use anna_shared::recipe_matcher::{match_recipe, MatchResult};
use anna_shared::rpc::{EvidenceBlock, QueryIntent, ReliabilitySignals, ServiceDeskResult, SpecialistDomain, TranslatorTicket};
use anna_shared::shell_recipes;
use anna_shared::git_recipes;
use anna_shared::trace::{ExecutionTrace, ProbeStats};
use anna_shared::transcript::Transcript;
use tracing::info;

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
        info!("Shell recipe match found: {}", result.recipe.as_ref().map(|r| r.id.clone()).unwrap_or_default());
        return result;
    }

    // Third, check built-in git recipes
    if let Some(result) = check_git_recipes(query) {
        info!("Git recipe match found");
        return result;
    }

    RecipeFastPathResult::no_match()
}

/// Check query against built-in shell recipes
fn check_shell_recipes(query: &str) -> Option<RecipeFastPathResult> {
    let q = query.to_lowercase();

    // Detect shell from query or environment
    let shell = if q.contains("bash") || q.contains("bashrc") {
        Some(shell_recipes::Shell::Bash)
    } else if q.contains("zsh") || q.contains("zshrc") {
        Some(shell_recipes::Shell::Zsh)
    } else if q.contains("fish") {
        Some(shell_recipes::Shell::Fish)
    } else {
        shell_recipes::Shell::detect()
    };

    // Detect feature from query
    let feature = shell_recipes::detect_feature(&q)?;
    let shell = shell?;

    // Find matching recipe
    let recipe = shell_recipes::find_recipe(shell, feature)?;

    // Build a synthetic Recipe for the result
    let synthetic_recipe = Recipe {
        id: format!("shell-{}-{:?}", shell, feature),
        signature: anna_shared::recipe::RecipeSignature::new(
            "desktop",
            "request",
            "shell_config",
            query,
        ),
        team: anna_shared::teams::Team::Desktop,
        risk_level: anna_shared::ticket::RiskLevel::LowRiskChange,
        required_evidence_kinds: vec![],
        probe_sequence: vec![],
        answer_template: format!(
            "To {} in {}:\n\nAdd to ~/{}\n```\n{}\n```\n\n{}",
            feature.display_name(),
            shell.display_name(),
            shell.config_path().display(),
            recipe.lines.join("\n"),
            recipe.rollback_hint.as_deref().unwrap_or("To undo: remove the added lines")
        ),
        created_at: 0,
        success_count: 100, // Built-in = mature
        reliability_score: 95,
        kind: RecipeKind::ShellConfig,
        target: None,
        action: RecipeAction::EnsureLine { line: recipe.lines.join("\n") },
        rollback: None,
        clarification_slots: vec![],
        default_question_id: None,
        populates_facts: vec![],
        intent_tags: feature.keywords().iter().map(|s| s.to_string()).collect(),
        targets: vec![shell.display_name().to_lowercase()],
        preconditions: vec![],
        clarify_prereqs: vec![],
    };

    Some(RecipeFastPathResult {
        matched: true,
        ticket: Some(ticket_from_recipe(&synthetic_recipe)),
        recipe: Some(synthetic_recipe),
        score: 90,
        matched_tokens: vec![shell.display_name().to_lowercase(), feature.display_name().to_string()],
        skip_llm: true,
    })
}

/// Check query against built-in git recipes
fn check_git_recipes(query: &str) -> Option<RecipeFastPathResult> {
    let q = query.to_lowercase();

    // Must mention "git" to match git recipes
    if !q.contains("git") {
        return None;
    }

    // Detect feature from query
    let feature = git_recipes::detect_feature(&q)?;

    // Find matching recipes
    let recipes = git_recipes::find_recipe(feature);
    if recipes.is_empty() {
        return None;
    }

    let recipe = &recipes[0];

    // Build answer from recipe
    let answer = if recipe.needs_parameters() {
        format!(
            "To configure {}:\n\nCommands:\n{}\n\nNote: Replace {{name}} and {{email}} with your values.\n\n{}",
            feature.display_name(),
            recipe.commands.iter().map(|c| format!("  {}", c)).collect::<Vec<_>>().join("\n"),
            recipe.rollback_hint.as_deref().unwrap_or("")
        )
    } else {
        format!(
            "To configure {}:\n\nRun:\n{}\n\n{}",
            feature.display_name(),
            recipe.commands.iter().map(|c| format!("  {}", c)).collect::<Vec<_>>().join("\n"),
            recipe.rollback_hint.as_deref().unwrap_or("")
        )
    };

    // Build a synthetic Recipe
    let synthetic_recipe = Recipe {
        id: format!("git-{:?}", feature),
        signature: anna_shared::recipe::RecipeSignature::new(
            "system",
            "request",
            "git_config",
            query,
        ),
        team: anna_shared::teams::Team::General,
        risk_level: anna_shared::ticket::RiskLevel::LowRiskChange,
        required_evidence_kinds: vec![],
        probe_sequence: vec![],
        answer_template: answer,
        created_at: 0,
        success_count: 100,
        reliability_score: 95,
        kind: RecipeKind::GitConfig,
        target: None,
        action: RecipeAction::None,
        rollback: None,
        clarification_slots: vec![],
        default_question_id: None,
        populates_facts: vec![],
        intent_tags: feature.keywords().iter().map(|s| s.to_string()).collect(),
        targets: vec!["git".to_string()],
        preconditions: vec![],
        clarify_prereqs: vec![],
    };

    Some(RecipeFastPathResult {
        matched: true,
        ticket: Some(ticket_from_recipe(&synthetic_recipe)),
        recipe: Some(synthetic_recipe),
        score: 90,
        matched_tokens: vec!["git".to_string(), feature.display_name().to_string()],
        skip_llm: true,
    })
}

/// Map recipe team to specialist domain
fn team_to_domain(team: &anna_shared::teams::Team) -> SpecialistDomain {
    match team {
        anna_shared::teams::Team::Network => SpecialistDomain::Network,
        anna_shared::teams::Team::Storage => SpecialistDomain::Storage,
        anna_shared::teams::Team::Security => SpecialistDomain::Security,
        _ => SpecialistDomain::System,
    }
}

/// Create a TranslatorTicket from a recipe
fn ticket_from_recipe(recipe: &Recipe) -> TranslatorTicket {
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

    ServiceDeskResult {
        request_id,
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
    }
}

/// Check if a recipe result can provide a direct answer (has answer_template)
pub fn can_answer_directly(result: &RecipeFastPathResult) -> bool {
    result.skip_llm && result.recipe.as_ref()
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
            assert!(result.recipe.as_ref().map(|r| !r.id.starts_with("git")).unwrap_or(true));
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
        assert!(service_result.execution_trace.as_ref().unwrap().answer_is_deterministic);
    }
}
