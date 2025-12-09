//! RAG-lite recipe search functions (v0.0.177).

use super::{recipe_dir, Recipe};

/// A recipe match with relevance score
#[derive(Debug, Clone)]
pub struct RecipeMatch {
    pub recipe: Recipe,
    pub score: u32,
    pub matched_keywords: Vec<String>,
}

/// Search recipes by keywords (v0.0.35 RAG-lite)
/// Returns top N matches sorted by score descending, then by id ascending for determinism.
pub fn search_recipes_by_keywords(keywords: &[&str], limit: usize) -> Vec<RecipeMatch> {
    let dir = recipe_dir();
    if !dir.exists() {
        return Vec::new();
    }

    let mut matches: Vec<RecipeMatch> = Vec::new();

    // Load all recipes and score them
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if let Ok(json) = std::fs::read_to_string(entry.path()) {
                if let Ok(recipe) = serde_json::from_str::<Recipe>(&json) {
                    if let Some(match_result) = score_recipe(&recipe, keywords) {
                        matches.push(match_result);
                    }
                }
            }
        }
    }

    // Sort by score desc, then by id asc for determinism
    matches.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.recipe.id.cmp(&b.recipe.id))
    });

    matches.truncate(limit);
    matches
}

/// Score a recipe against keywords
fn score_recipe(recipe: &Recipe, keywords: &[&str]) -> Option<RecipeMatch> {
    let mut score: u32 = 0;
    let mut matched = Vec::new();

    // Build searchable text from recipe
    let searchable = format!(
        "{} {} {} {}",
        recipe.signature.query_pattern,
        recipe.signature.domain,
        recipe.signature.route_class,
        recipe.answer_template
    )
    .to_lowercase();

    // Score each keyword
    for &kw in keywords {
        let kw_lower = kw.to_lowercase();
        if searchable.contains(&kw_lower) {
            score += 10;
            matched.push(kw.to_string());

            // Bonus for exact route_class match
            if recipe.signature.route_class.to_lowercase() == kw_lower {
                score += 20;
            }
            // Bonus for domain match
            if recipe.signature.domain.to_lowercase() == kw_lower {
                score += 15;
            }
        }
    }

    // Bonus for high reliability
    score += (recipe.reliability_score / 10) as u32;

    // Bonus for maturity
    if recipe.is_mature() {
        score += 5;
    }

    if score > 0 && !matched.is_empty() {
        Some(RecipeMatch {
            recipe: recipe.clone(),
            score,
            matched_keywords: matched,
        })
    } else {
        None
    }
}

/// Find recipes for config edit intents (v0.0.35)
/// Used before escalating to junior reviewer
pub fn find_config_edit_recipes(app_id: &str) -> Vec<Recipe> {
    let dir = recipe_dir();
    if !dir.exists() {
        return Vec::new();
    }

    let mut recipes: Vec<Recipe> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if let Ok(json) = std::fs::read_to_string(entry.path()) {
                if let Ok(recipe) = serde_json::from_str::<Recipe>(&json) {
                    if recipe.is_config_edit() {
                        if let Some(target) = &recipe.target {
                            if target.app_id == app_id {
                                recipes.push(recipe);
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort by success_count desc for consistency
    recipes.sort_by(|a, b| b.success_count.cmp(&a.success_count));
    recipes
}
