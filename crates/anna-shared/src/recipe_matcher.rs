//! Recipe matcher for fast-path resolution (v0.0.373).
//!
//! The translator uses this to check if a learned recipe can answer a query
//! WITHOUT calling the LLM specialist. This is the key to Anna's learning:
//!
//! 1. First query: Specialist (LLM) generates answer -> Recipe is learned
//! 2. Similar query: Translator finds matching recipe -> No LLM needed!
//!
//! The matcher uses semantic similarity based on:
//! - Intent (what the user wants to do)
//! - Target (what they want to do it to)
//! - Action verbs (enable, install, configure, etc.)
//!
//! v0.0.373: Dynamic thresholds based on recipe maturity and reliability.

use crate::recipe::{recipe_dir, Recipe, RecipeAction, RecipeKind};
use crate::recipe_index::{tokenize, RecipeIndex};
use std::collections::BTreeSet;

/// Base minimum score threshold for recipe match (out of 100)
/// v0.0.373: Now dynamically adjusted based on recipe maturity
const BASE_MATCH_THRESHOLD: u32 = 60;

/// Minimum tokens that must match for a valid match
const MIN_MATCHING_TOKENS: usize = 2;

/// v0.0.373: Calculate dynamic match threshold based on recipe maturity
/// Immature recipes need higher scores to match (prevents wrong answers)
/// Mature, high-reliability recipes can match with lower scores
fn dynamic_threshold(recipe: &Recipe) -> u32 {
    let maturity_factor = match recipe.success_count {
        0 => 25,     // Untested: need very high score
        1..=2 => 15, // New: need higher score
        3..=5 => 10, // Young: slightly elevated
        6..=10 => 5, // Maturing: slight boost
        _ => 0,      // Mature: base threshold
    };

    let reliability_factor = match recipe.reliability_score {
        90..=100 => 0, // Excellent: no penalty
        80..=89 => 5,  // Good: small boost needed
        70..=79 => 10, // Okay: moderate boost
        _ => 15,       // Low: need much higher match
    };

    // Higher threshold = harder to match = fewer wrong answers
    (BASE_MATCH_THRESHOLD + maturity_factor + reliability_factor).min(95)
}

/// Result of matching a query against learned recipes
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// The matched recipe
    pub recipe: Recipe,
    /// Match score (0-100)
    pub score: u32,
    /// Tokens that matched between query and recipe
    pub matched_tokens: Vec<String>,
    /// Whether this is a high-confidence match (can skip LLM)
    pub high_confidence: bool,
    /// Suggested parameter substitutions
    pub substitutions: Vec<(String, String)>,
}

impl MatchResult {
    /// Check if this match is strong enough to use without LLM
    /// v0.0.373: Uses dynamic threshold based on recipe maturity
    pub fn can_skip_llm(&self) -> bool {
        let threshold = dynamic_threshold(&self.recipe);
        self.high_confidence && self.score >= threshold
    }
}

/// Match query against learned recipes
///
/// Returns the best matching recipe if score > threshold, else None.
/// The translator should call this BEFORE escalating to the specialist.
/// v0.0.373: Uses dynamic thresholds based on recipe maturity/reliability.
pub fn match_recipe(query: &str, index: &RecipeIndex) -> Option<MatchResult> {
    let query_tokens: BTreeSet<String> = tokenize(query).into_iter().collect();

    if query_tokens.len() < MIN_MATCHING_TOKENS {
        return None;
    }

    // Search recipes
    let matches = index.search_recipes(query, 5);

    if matches.is_empty() {
        return None;
    }

    // Get best match
    let (recipe, raw_score) = matches.into_iter().next()?;

    // Normalize score to 0-100
    let max_possible = (query_tokens.len() * 3) as u32 + 10; // rough estimate
    let score = ((raw_score as f32 / max_possible as f32) * 100.0).min(100.0) as u32;

    // v0.0.373: Use dynamic threshold based on recipe maturity
    let threshold = dynamic_threshold(&recipe);

    // Check if strong enough (use half of dynamic threshold for early filtering)
    if score < threshold / 2 {
        return None;
    }

    // Compute matched tokens
    let recipe_tokens: BTreeSet<String> = recipe
        .intent_tags
        .iter()
        .chain(recipe.targets.iter())
        .flat_map(|s| tokenize(s))
        .chain(tokenize(&recipe.signature.query_pattern))
        .collect();

    let matched_tokens: Vec<String> = query_tokens
        .iter()
        .filter(|t| recipe_tokens.contains(*t))
        .cloned()
        .collect();

    if matched_tokens.len() < MIN_MATCHING_TOKENS {
        return None;
    }

    // v0.0.373: Determine high confidence using dynamic threshold
    let high_confidence = score >= threshold && matched_tokens.len() >= 3 && recipe.is_mature();

    // Extract substitutions (e.g., different package name, different editor)
    let substitutions = extract_substitutions(query, &recipe);

    Some(MatchResult {
        recipe,
        score,
        matched_tokens,
        high_confidence,
        substitutions,
    })
}

/// Try to find a recipe for a config action
///
/// Looks for recipes that can be adapted for similar config changes.
/// E.g., "enable syntax highlighting in nano" can use vim syntax recipe
/// with path/command substitutions.
pub fn match_config_recipe(
    intent: &str, // e.g., "enable syntax highlighting"
    target: &str, // e.g., "nano"
    index: &RecipeIndex,
) -> Option<MatchResult> {
    // Build search query from intent + target
    let query = format!("{} {}", intent, target);

    // First try exact match with target
    let matches = index.search_recipes(&query, 5);

    for (recipe, score) in &matches {
        // Check if recipe applies to this target
        if recipe
            .targets
            .iter()
            .any(|t| t.to_lowercase() == target.to_lowercase())
        {
            return Some(MatchResult {
                recipe: recipe.clone(),
                score: *score,
                matched_tokens: vec![intent.to_string(), target.to_string()],
                high_confidence: true,
                substitutions: vec![],
            });
        }
    }

    // Then try similar recipes that could be adapted
    // v0.0.373: Use dynamic threshold
    for (recipe, score) in matches {
        let threshold = dynamic_threshold(&recipe);
        if recipe.is_config_edit() && score >= threshold / 2 {
            // Can adapt this recipe for different target
            let substitutions = vec![("target".to_string(), target.to_string())];

            return Some(MatchResult {
                recipe,
                score,
                matched_tokens: vec![intent.to_string()],
                high_confidence: false, // Needs verification
                substitutions,
            });
        }
    }

    None
}

/// Try to find a recipe for a package/service action
///
/// Looks for recipes that can be adapted for similar package operations.
/// E.g., "install htop" can use "install vim" recipe with package substitution.
pub fn match_action_recipe(
    action: &str, // e.g., "install", "restart"
    target: &str, // e.g., "htop", "docker"
    index: &RecipeIndex,
) -> Option<MatchResult> {
    // Search for similar action recipes
    let query = format!("{} {}", action, target);
    let matches = index.search_recipes(&query, 10);

    for (recipe, score) in matches {
        // Check if this is an action recipe
        match &recipe.kind {
            RecipeKind::Query => {
                // Check if query pattern contains the same action verb
                if recipe.signature.query_pattern.contains(action) {
                    let substitutions = extract_action_substitutions(action, target, &recipe);
                    let threshold = dynamic_threshold(&recipe);

                    return Some(MatchResult {
                        recipe,
                        score,
                        matched_tokens: vec![action.to_string()],
                        high_confidence: score >= threshold,
                        substitutions,
                    });
                }
            }
            _ => {
                // Config edit recipes can also match action patterns
                if let RecipeAction::EnsureLine { line } = &recipe.action {
                    if line.contains(action) || line.contains(target) {
                        return Some(MatchResult {
                            recipe,
                            score,
                            matched_tokens: vec![action.to_string(), target.to_string()],
                            high_confidence: true,
                            substitutions: vec![],
                        });
                    }
                }
            }
        }
    }

    None
}

/// Extract parameter substitutions between query and recipe
fn extract_substitutions(query: &str, recipe: &Recipe) -> Vec<(String, String)> {
    let mut subs = Vec::new();

    let query_tokens = tokenize(query);
    let pattern_tokens = tokenize(&recipe.signature.query_pattern);

    // Find tokens in query that aren't in pattern (likely parameters)
    for qt in &query_tokens {
        if !pattern_tokens.contains(qt) {
            // Try to identify what kind of parameter this is
            if looks_like_package_name(qt) {
                subs.push(("package".to_string(), qt.clone()));
            } else if looks_like_service_name(qt) {
                subs.push(("service".to_string(), qt.clone()));
            } else if looks_like_editor_name(qt) {
                subs.push(("editor".to_string(), qt.clone()));
            }
        }
    }

    subs
}

/// Extract substitutions for action recipes
fn extract_action_substitutions(
    action: &str,
    target: &str,
    recipe: &Recipe,
) -> Vec<(String, String)> {
    let mut subs = Vec::new();

    // Extract original target from recipe
    let pattern_tokens = tokenize(&recipe.signature.query_pattern);

    // The new target is a substitution for whatever was in the original
    for pt in pattern_tokens {
        if pt != action && looks_like_target(&pt) {
            subs.push((pt, target.to_string()));
            break;
        }
    }

    subs
}

/// Check if token looks like a package name
fn looks_like_package_name(token: &str) -> bool {
    // Common package patterns
    let patterns = [
        "vim", "htop", "git", "nano", "curl", "wget", "docker", "nginx",
    ];
    // Must be at least 2 chars, not a common word
    let common_words = [
        "the", "and", "for", "you", "can", "how", "what", "this", "that", "with",
    ];
    if token.len() < 2 || common_words.contains(&token) {
        return false;
    }
    patterns.contains(&token) || token.chars().all(|c| c.is_alphanumeric() || c == '-')
}

/// Check if token looks like a service name
fn looks_like_service_name(token: &str) -> bool {
    token.ends_with(".service")
        || token.ends_with("d")
        || ["docker", "nginx", "sshd", "httpd", "cups", "bluetooth"].contains(&token)
}

/// Check if token looks like an editor name
fn looks_like_editor_name(token: &str) -> bool {
    [
        "vim", "nvim", "nano", "emacs", "helix", "micro", "code", "kate", "gedit",
    ]
    .contains(&token)
}

/// Check if token looks like a target (package, service, editor)
fn looks_like_target(token: &str) -> bool {
    looks_like_package_name(token)
        || looks_like_service_name(token)
        || looks_like_editor_name(token)
}

/// v0.0.104: Try to match an SSH-related query to builtin SSH recipes
pub fn match_ssh_recipe(query: &str) -> Option<&'static crate::ssh_recipes::SshRecipe> {
    crate::ssh_recipes::match_query(query)
}

/// Load recipe index from disk
pub fn load_recipe_index() -> RecipeIndex {
    RecipeIndex::build_from_disk()
}

/// Get recipes count
pub fn recipe_count() -> usize {
    let dir = recipe_dir();
    if !dir.exists() {
        return 0;
    }
    std::fs::read_dir(&dir)
        .map(|entries| entries.filter_map(|e| e.ok()).count())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_looks_like_package_name() {
        assert!(looks_like_package_name("vim"));
        assert!(looks_like_package_name("htop"));
        assert!(looks_like_package_name("my-package"));
        assert!(!looks_like_package_name("the"));
    }

    #[test]
    fn test_looks_like_service_name() {
        assert!(looks_like_service_name("docker"));
        assert!(looks_like_service_name("sshd"));
        assert!(looks_like_service_name("nginx.service"));
        assert!(!looks_like_service_name("install"));
    }

    #[test]
    fn test_looks_like_editor_name() {
        assert!(looks_like_editor_name("vim"));
        assert!(looks_like_editor_name("nano"));
        assert!(looks_like_editor_name("emacs"));
        assert!(!looks_like_editor_name("htop"));
    }

    #[test]
    fn test_empty_index() {
        let index = RecipeIndex::new();
        let result = match_recipe("install htop", &index);
        assert!(result.is_none());
    }

    #[test]
    fn test_dynamic_threshold_maturity() {
        use crate::recipe::RecipeSignature;
        use crate::teams::Team;
        use crate::ticket::RiskLevel;

        let sig = RecipeSignature {
            domain: "test".to_string(),
            intent: "test".to_string(),
            route_class: "test".to_string(),
            query_pattern: "test query".to_string(),
        };

        // Create test recipes with different maturity levels
        let mut new_recipe = Recipe::new(
            sig.clone(),
            Team::General,
            RiskLevel::ReadOnly,
            vec![],
            vec![],
            "test".to_string(),
            85,
        );
        new_recipe.success_count = 0;

        let mut young_recipe = Recipe::new(
            sig.clone(),
            Team::General,
            RiskLevel::ReadOnly,
            vec![],
            vec![],
            "test".to_string(),
            85,
        );
        young_recipe.success_count = 2;

        let mut mature_recipe = Recipe::new(
            sig.clone(),
            Team::General,
            RiskLevel::ReadOnly,
            vec![],
            vec![],
            "test".to_string(),
            95,
        );
        mature_recipe.success_count = 20;

        // New recipes should require higher match scores
        let new_threshold = dynamic_threshold(&new_recipe);
        let young_threshold = dynamic_threshold(&young_recipe);
        let mature_threshold = dynamic_threshold(&mature_recipe);

        assert!(
            new_threshold > young_threshold,
            "new={} should > young={}",
            new_threshold,
            young_threshold
        );
        assert!(
            young_threshold > mature_threshold,
            "young={} should > mature={}",
            young_threshold,
            mature_threshold
        );
    }

    #[test]
    fn test_dynamic_threshold_reliability() {
        use crate::recipe::RecipeSignature;
        use crate::teams::Team;
        use crate::ticket::RiskLevel;

        let sig = RecipeSignature {
            domain: "test".to_string(),
            intent: "test".to_string(),
            route_class: "test".to_string(),
            query_pattern: "test query".to_string(),
        };

        // Same maturity, different reliability
        let mut high_reliability = Recipe::new(
            sig.clone(),
            Team::General,
            RiskLevel::ReadOnly,
            vec![],
            vec![],
            "test".to_string(),
            95,
        );
        high_reliability.success_count = 10;

        let mut low_reliability = Recipe::new(
            sig.clone(),
            Team::General,
            RiskLevel::ReadOnly,
            vec![],
            vec![],
            "test".to_string(),
            65,
        );
        low_reliability.success_count = 10;

        let high_threshold = dynamic_threshold(&high_reliability);
        let low_threshold = dynamic_threshold(&low_reliability);

        assert!(
            low_threshold > high_threshold,
            "low reliability should need higher match score"
        );
    }
}
