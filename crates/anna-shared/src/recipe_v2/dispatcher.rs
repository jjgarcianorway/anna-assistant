//! Recipe V2 dispatcher integration (v0.0.420).
//!
//! This module provides integration between the recipe system and the
//! request dispatcher, enabling:
//! - Recipe matching before specialist calls
//! - Learning from successful tickets
//! - Recipe-based fast path execution

use std::collections::HashMap;

use super::{
    find_best_recipe, get_seed_recipes, MatchResult, RecipeLearner, RecipeStorageV2,
    RecipeV2, TicketObservation,
};

/// Recipe dispatcher - integrates recipes into the request flow
pub struct RecipeDispatcher {
    /// Recipe storage
    pub storage: RecipeStorageV2,
    /// Recipe learner
    pub learner: RecipeLearner,
    /// Whether recipe matching is enabled
    pub matching_enabled: bool,
    /// Whether learning is enabled
    pub learning_enabled: bool,
}

impl Default for RecipeDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl RecipeDispatcher {
    /// Create a new dispatcher
    pub fn new() -> Self {
        Self {
            storage: RecipeStorageV2::new(),
            learner: RecipeLearner::new(),
            matching_enabled: true,
            learning_enabled: true,
        }
    }

    /// Initialize the dispatcher (load recipes, etc.)
    pub fn init(&mut self) -> Result<(), String> {
        // Load all recipes
        self.storage.load_all()?;

        // If no recipes loaded, add seed recipes
        if self.storage.is_empty() {
            for recipe in get_seed_recipes() {
                // Don't fail on seed recipe errors
                let _ = self.storage.upsert(recipe);
            }
        }

        Ok(())
    }

    /// Try to match a recipe for the given query
    pub fn try_match(
        &self,
        intent: &str,
        keywords: &[String],
        facts: &HashMap<String, String>,
    ) -> Option<MatchResult> {
        if !self.matching_enabled {
            return None;
        }

        let available: Vec<&RecipeV2> = self.storage.get_available();
        find_best_recipe(&available, intent, keywords, facts)
    }

    /// Check if a recipe matches with high confidence (skip specialist)
    pub fn should_skip_specialist(
        &self,
        intent: &str,
        keywords: &[String],
        facts: &HashMap<String, String>,
    ) -> Option<RecipeV2> {
        let result = self.try_match(intent, keywords, facts)?;
        if result.should_auto_apply() {
            Some(result.recipe)
        } else {
            None
        }
    }

    /// Record a successful ticket for learning
    pub fn record_success(&mut self, observation: TicketObservation) {
        if !self.learning_enabled {
            return;
        }

        let intent = observation.intent.clone();
        self.learner.record(observation);

        // Try to learn a new recipe if ready
        if self.learner.ready_to_learn(&intent) {
            if let Ok(recipe) = self.learner.learn_and_save(&intent, &mut self.storage) {
                tracing::info!(
                    "Learned new recipe: {} ({})",
                    recipe.id,
                    recipe.title
                );
            }
        }
    }

    /// Record a recipe execution result
    pub fn record_execution(&mut self, recipe_id: &str, success: bool, duration_ms: u64) {
        if let Some(recipe) = self.storage.get_mut(recipe_id) {
            if success {
                recipe.record_success(duration_ms);
            } else {
                recipe.record_failure(duration_ms);
            }

            // Check if recipe should be disabled
            if recipe.stats.should_disable() {
                tracing::warn!(
                    "Disabling recipe {} due to low success rate: {:.1}%",
                    recipe.id,
                    recipe.stats.success_rate() * 100.0
                );
                recipe.enabled = false;
            }

            // Save updated stats (ignore errors)
            let _ = super::storage::save_user_recipe(recipe);
        }
    }

    /// Get recipe by ID
    pub fn get_recipe(&self, id: &str) -> Option<&RecipeV2> {
        self.storage.get(id)
    }

    /// Get all available recipes
    pub fn list_recipes(&self) -> Vec<&RecipeV2> {
        self.storage.get_available()
    }

    /// Get recipe count
    pub fn recipe_count(&self) -> usize {
        self.storage.len()
    }

    /// Enable/disable recipe matching
    pub fn set_matching_enabled(&mut self, enabled: bool) {
        self.matching_enabled = enabled;
    }

    /// Enable/disable learning
    pub fn set_learning_enabled(&mut self, enabled: bool) {
        self.learning_enabled = enabled;
        self.storage.set_learning_enabled(enabled);
    }

    /// Reset all user recipes
    pub fn reset_user_recipes(&mut self) -> Result<(), String> {
        self.storage.clear_user_recipes()?;
        self.learner.clear_all();
        Ok(())
    }
}

/// Query context for recipe matching
#[derive(Debug, Clone, Default)]
pub struct RecipeQuery {
    /// Normalized intent from translator
    pub intent: String,
    /// Keywords from query
    pub keywords: Vec<String>,
    /// Known facts (editor, os, etc.)
    pub facts: HashMap<String, String>,
}

impl RecipeQuery {
    /// Create a new query
    pub fn new(intent: &str) -> Self {
        Self {
            intent: intent.to_string(),
            keywords: Vec::new(),
            facts: HashMap::new(),
        }
    }

    /// Add keywords
    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }

    /// Add a fact
    pub fn with_fact(mut self, key: &str, value: &str) -> Self {
        self.facts.insert(key.to_string(), value.to_string());
        self
    }
}

/// Result of recipe-based execution
#[derive(Debug, Clone)]
pub struct RecipeResult {
    /// Recipe that was executed
    pub recipe_id: String,
    /// Whether execution was successful
    pub success: bool,
    /// Answer text from recipe
    pub answer: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Citations from recipe
    pub citations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatcher_init() {
        let mut dispatcher = RecipeDispatcher::new();
        // Note: init() may fail if directories don't exist, which is fine for tests
        let _ = dispatcher.init();
    }

    #[test]
    fn test_recipe_query() {
        let query = RecipeQuery::new("show_memory")
            .with_keywords(vec!["memory".into(), "free".into()])
            .with_fact("os", "arch");

        assert_eq!(query.intent, "show_memory");
        assert_eq!(query.keywords.len(), 2);
        assert_eq!(query.facts.get("os"), Some(&"arch".to_string()));
    }
}
