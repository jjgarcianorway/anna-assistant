//! Query matching for SSH recipes (v0.0.196).

use super::recipes::builtin_recipes;
use super::types::SshRecipe;

/// Match a query to an SSH recipe
pub fn match_query(query: &str) -> Option<&'static SshRecipe> {
    let query_lower = query.to_lowercase();

    // Must mention SSH
    if !query_lower.contains("ssh") && !query_lower.contains("key") {
        return None;
    }

    // Lazy static for builtin recipes
    static RECIPES: std::sync::OnceLock<Vec<SshRecipe>> = std::sync::OnceLock::new();
    let recipes = RECIPES.get_or_init(builtin_recipes);

    // Score each feature by keyword matches
    let mut best_match: Option<(usize, &SshRecipe)> = None;

    for recipe in recipes {
        let score = recipe
            .feature
            .keywords()
            .iter()
            .filter(|kw| query_lower.contains(*kw))
            .count();

        if score > 0 {
            match best_match {
                None => best_match = Some((score, recipe)),
                Some((best_score, _)) if score > best_score => best_match = Some((score, recipe)),
                _ => {}
            }
        }
    }

    best_match.map(|(_, recipe)| recipe)
}
