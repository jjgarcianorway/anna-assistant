//! Recipe Promotion - Convert tested candidates to library recipes.
//!
//! After a candidate passes testing, it can be promoted to the recipe library.
//! This involves:
//! 1. Converting to Recipe format
//! 2. Saving to the recipe store
//! 3. Updating indexes

use super::candidate::RecipeCandidate;
use super::testing::TestResult;
use crate::recipe::{Recipe, RecipeSource};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Result of attempting to promote a candidate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PromotionResult {
    /// Successfully promoted
    Promoted(PromotedRecipe),
    /// Testing failed
    TestFailed(TestResult),
    /// Promotion failed
    Error(String),
}

/// A successfully promoted recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotedRecipe {
    /// The promoted recipe
    pub recipe: Recipe,
    /// Path where it was saved
    pub saved_to: Option<PathBuf>,
    /// Summary of the promotion
    pub summary: String,
}

/// Promote a candidate to a full recipe
pub fn promote_candidate(candidate: RecipeCandidate) -> PromotionResult {
    // Convert to Recipe
    let recipe = Recipe {
        id: candidate.id.replace("candidate-", "recipe-"),
        name: candidate.name,
        keywords: candidate.keywords,
        patterns: candidate.patterns,
        context: candidate.context,
        commands: candidate.commands,
        verification: candidate.verification,
        source: RecipeSource::Learned,
        success_count: candidate.cluster_success_count,
        last_used: None,
        enabled: true,
    };

    // Try to save to disk
    let save_result = save_recipe(&recipe);

    PromotionResult::Promoted(PromotedRecipe {
        recipe: recipe.clone(),
        saved_to: save_result.ok(),
        summary: format!(
            "Promoted '{}' with {} commands (from {} cluster successes)",
            recipe.name,
            recipe.commands.len(),
            candidate.cluster_success_count
        ),
    })
}

/// Save a recipe to the recipe store
fn save_recipe(recipe: &Recipe) -> Result<PathBuf, String> {
    let recipe_dir = get_recipe_dir()?;
    std::fs::create_dir_all(&recipe_dir).map_err(|e| e.to_string())?;

    let path = recipe_dir.join(format!("{}.json", recipe.id));
    let content = serde_json::to_string_pretty(recipe).map_err(|e| e.to_string())?;

    std::fs::write(&path, content).map_err(|e| e.to_string())?;

    Ok(path)
}

/// Get the recipe directory (system-wide)
fn get_recipe_dir() -> Result<PathBuf, String> {
    Ok(crate::paths::paths().recipes_dir())
}

/// Load all promoted recipes from disk
pub fn load_promoted_recipes() -> Vec<Recipe> {
    let recipe_dir = match get_recipe_dir() {
        Ok(dir) => dir,
        Err(_) => return Vec::new(),
    };

    if !recipe_dir.exists() {
        return Vec::new();
    }

    let mut recipes = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&recipe_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Ok(recipe) = serde_json::from_str::<Recipe>(&content) {
                        recipes.push(recipe);
                    }
                }
            }
        }
    }

    recipes
}

/// Get count of promoted recipes
pub fn promoted_recipe_count() -> usize {
    load_promoted_recipes().len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{Experience, ExperienceContext};
    use crate::skill_promotion::generate_candidate;

    fn make_test_experience() -> Experience {
        Experience {
            id: "test-abc-123".to_string(),
            question: "how to check disk space".to_string(),
            keywords: vec!["disk".to_string(), "space".to_string()],
            successful_commands: vec!["df -h".to_string()],
            answer: "Use df -h".to_string(),
            context: ExperienceContext::default(),
            usefulness_score: 10,
            created_at: "2024-01-01".to_string(),
            last_used: None,
            embedding: None,
        }
    }

    #[test]
    fn test_promotion() {
        let exp = make_test_experience();
        let candidate = generate_candidate(&exp);
        let result = promote_candidate(candidate);

        match result {
            PromotionResult::Promoted(promoted) => {
                assert!(promoted.recipe.id.starts_with("recipe-"));
                assert!(!promoted.recipe.commands.is_empty());
            }
            _ => panic!("Expected Promoted result"),
        }
    }
}
