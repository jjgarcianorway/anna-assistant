//! Recipe matcher for fast-path resolution (v0.0.100).
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

use crate::recipe::{Recipe, RecipeKind, RecipeAction, recipe_dir};
use crate::recipe_index::{tokenize, RecipeIndex};
use std::collections::BTreeSet;

/// Minimum score threshold for recipe match (out of 100)
const MATCH_THRESHOLD: u32 = 60;

/// Minimum tokens that must match for a valid match
const MIN_MATCHING_TOKENS: usize = 2;

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
    pub fn can_skip_llm(&self) -> bool {
        self.high_confidence && self.score >= MATCH_THRESHOLD
    }
}

/// Match query against learned recipes
///
/// Returns the best matching recipe if score > threshold, else None.
/// The translator should call this BEFORE escalating to the specialist.
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

    // Check if strong enough
    if score < MATCH_THRESHOLD / 2 {
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

    // Determine high confidence
    let high_confidence = score >= MATCH_THRESHOLD
        && matched_tokens.len() >= 3
        && recipe.is_mature();

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
    intent: &str,      // e.g., "enable syntax highlighting"
    target: &str,      // e.g., "nano"
    index: &RecipeIndex,
) -> Option<MatchResult> {
    // Build search query from intent + target
    let query = format!("{} {}", intent, target);

    // First try exact match with target
    let matches = index.search_recipes(&query, 5);

    for (recipe, score) in &matches {
        // Check if recipe applies to this target
        if recipe.targets.iter().any(|t| t.to_lowercase() == target.to_lowercase()) {
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
    for (recipe, score) in matches {
        if recipe.is_config_edit() && score >= MATCH_THRESHOLD / 2 {
            // Can adapt this recipe for different target
            let substitutions = vec![
                ("target".to_string(), target.to_string()),
            ];

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
    action: &str,      // e.g., "install", "restart"
    target: &str,      // e.g., "htop", "docker"
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

                    return Some(MatchResult {
                        recipe,
                        score,
                        matched_tokens: vec![action.to_string()],
                        high_confidence: score >= MATCH_THRESHOLD,
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
fn extract_action_substitutions(action: &str, target: &str, recipe: &Recipe) -> Vec<(String, String)> {
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
    let patterns = ["vim", "htop", "git", "nano", "curl", "wget", "docker", "nginx"];
    // Must be at least 2 chars, not a common word
    let common_words = ["the", "and", "for", "you", "can", "how", "what", "this", "that", "with"];
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
    ["vim", "nvim", "nano", "emacs", "helix", "micro", "code", "kate", "gedit"].contains(&token)
}

/// Check if token looks like a target (package, service, editor)
fn looks_like_target(token: &str) -> bool {
    looks_like_package_name(token) || looks_like_service_name(token) || looks_like_editor_name(token)
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
}
