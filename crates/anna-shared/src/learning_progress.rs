//! Learning progress tracking (v0.0.288).
//!
//! Tracks Anna's growth from knowing little to becoming knowledgeable.
//! All data is derived from actual recipe learning - no hardcoded content.
//!
//! Key principle: Anna starts empty, learns from specialists, improves over time.

use crate::recipe::Recipe;
use crate::stats::GlobalStats;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Learning progress summary - shows Anna's growth
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LearningProgress {
    /// Total recipes learned
    pub recipes_total: usize,
    /// Recipes by category
    pub by_category: HashMap<String, CategoryProgress>,
    /// Recent learning (last 7 days)
    pub recent_learned: usize,
    /// Fast path ratio (recipes used vs specialist calls)
    pub self_sufficiency: f32,
    /// Overall average reliability
    pub avg_reliability: u8,
    /// Categories Anna knows well (>=5 recipes, >=70% reliability)
    pub strong_areas: Vec<String>,
    /// Categories needing more learning (<3 recipes or <60% reliability)
    pub growing_areas: Vec<String>,
}

/// Progress in a specific category
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategoryProgress {
    /// Category name
    pub name: String,
    /// Recipe count
    pub recipe_count: usize,
    /// Average reliability for this category
    pub avg_reliability: u8,
    /// Total times recipes in this category were used
    pub usage_count: u64,
    /// Success rate (verified / total)
    pub success_rate: f32,
}

impl LearningProgress {
    /// Build progress from current recipe state
    pub fn from_recipes(recipes: &[Recipe]) -> Self {
        if recipes.is_empty() {
            return Self::default();
        }

        // Aggregate by category
        let mut by_category: HashMap<String, CategoryProgress> = HashMap::new();
        let mut total_reliability: u64 = 0;

        for recipe in recipes {
            let entry = by_category
                .entry(recipe.signature.domain.clone())
                .or_insert_with(|| CategoryProgress {
                    name: recipe.signature.domain.clone(),
                    ..Default::default()
                });

            entry.recipe_count += 1;
            entry.usage_count += recipe.success_count as u64;
            total_reliability += recipe.reliability_score as u64;

            // Update running average reliability
            let n = entry.recipe_count as f32;
            entry.avg_reliability = ((entry.avg_reliability as f32 * (n - 1.0)
                + recipe.reliability_score as f32)
                / n) as u8;
        }

        // Identify strong and growing areas
        let strong_areas: Vec<String> = by_category
            .values()
            .filter(|c| c.recipe_count >= 5 && c.avg_reliability >= 70)
            .map(|c| c.name.clone())
            .collect();

        let growing_areas: Vec<String> = by_category
            .values()
            .filter(|c| c.recipe_count < 3 || c.avg_reliability < 60)
            .map(|c| c.name.clone())
            .collect();

        let avg_reliability = if recipes.is_empty() {
            0
        } else {
            (total_reliability / recipes.len() as u64) as u8
        };

        Self {
            recipes_total: recipes.len(),
            by_category,
            recent_learned: 0, // Would need timestamp tracking
            self_sufficiency: 0.0,
            avg_reliability,
            strong_areas,
            growing_areas,
        }
    }

    /// Add stats context for self-sufficiency calculation
    pub fn with_stats(mut self, stats: &GlobalStats) -> Self {
        if stats.total_requests > 0 {
            // Self-sufficiency = how often Anna uses recipes vs specialists
            let recipe_uses = stats.fast_path_hits + stats.recipe_hits;
            self.self_sufficiency = recipe_uses as f32 / stats.total_requests as f32;
        }
        self
    }

    /// Generate a learning summary message (for translator to naturalize)
    pub fn summary_for_translator(&self) -> String {
        if self.recipes_total == 0 {
            return "Anna is just starting out with no learned recipes yet. \
                    She relies entirely on specialist consultations."
                .to_string();
        }

        let mut parts = Vec::new();

        // Recipe count
        parts.push(format!("Anna knows {} recipes", self.recipes_total));

        // Self-sufficiency
        let pct = (self.self_sufficiency * 100.0) as u8;
        if pct >= 50 {
            parts.push(format!(
                "handles {}% of requests from her own knowledge",
                pct
            ));
        } else if pct > 0 {
            parts.push(format!("handles {}% independently (still learning)", pct));
        }

        // Strong areas
        if !self.strong_areas.is_empty() {
            parts.push(format!("strong in {}", self.strong_areas.join(", ")));
        }

        // Growing areas
        if !self.growing_areas.is_empty() {
            parts.push(format!(
                "still learning about {}",
                self.growing_areas.join(", ")
            ));
        }

        parts.join(". ") + "."
    }

    /// Check if Anna is still a beginner (few recipes)
    pub fn is_beginner(&self) -> bool {
        self.recipes_total < 10
    }

    /// Check if Anna has good coverage
    pub fn has_good_coverage(&self) -> bool {
        self.recipes_total >= 50 && self.self_sufficiency >= 0.5
    }
}

/// Load all recipes and compute progress
pub fn compute_learning_progress() -> LearningProgress {
    let recipes = load_all_recipes();
    LearningProgress::from_recipes(&recipes)
}

/// Load all learned recipes from disk
fn load_all_recipes() -> Vec<Recipe> {
    use crate::recipe::recipe_dir;

    let dir = recipe_dir();
    let mut recipes = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(recipe) = serde_json::from_str::<Recipe>(&content) {
                        recipes.push(recipe);
                    }
                }
            }
        }
    }

    recipes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe::RecipeSignature;
    use crate::teams::Team;
    use crate::ticket::RiskLevel;

    fn test_recipe(domain: &str, reliability: u8) -> Recipe {
        let signature = RecipeSignature {
            domain: domain.to_string(),
            intent: "test".to_string(),
            route_class: "test".to_string(),
            query_pattern: "test".to_string(),
        };

        Recipe::new(
            signature,
            Team::General,
            RiskLevel::ReadOnly,
            vec![],
            vec![],
            "test answer".to_string(),
            reliability,
        )
    }

    #[test]
    fn test_empty_progress() {
        let progress = LearningProgress::from_recipes(&[]);
        assert!(progress.is_beginner());
        assert_eq!(progress.recipes_total, 0);
    }

    #[test]
    fn test_beginner_detection() {
        let recipes = vec![test_recipe("storage", 80), test_recipe("network", 75)];
        let progress = LearningProgress::from_recipes(&recipes);
        assert!(progress.is_beginner()); // Less than 10 recipes
    }

    #[test]
    fn test_category_aggregation() {
        let recipes = vec![
            test_recipe("storage", 80),
            test_recipe("storage", 90),
            test_recipe("network", 70),
        ];
        let progress = LearningProgress::from_recipes(&recipes);

        assert_eq!(progress.by_category.len(), 2);
        assert_eq!(progress.by_category.get("storage").unwrap().recipe_count, 2);
    }
}
