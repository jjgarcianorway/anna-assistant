//! Recipe Learner (v0.0.416).
//!
//! Extracts reusable recipes from successful tickets.
//!
//! Learning process:
//! 1. Track tickets with same intent + high confidence
//! 2. Identify common probe patterns
//! 3. Extract answer structure (not exact words)
//! 4. Create generic, parameterized recipe
//!
//! NO HARDCODING of specific questions or answers.

mod candidates;
mod intent_learners;
mod observation;
mod utils;

pub use candidates::LearningCandidates;
pub use observation::TicketObservation;
pub use utils::{candidates_path, current_secs, find_common_probes};

use crate::canonical_intents::CanonicalIntent;
use crate::learned_recipes::{LearnedRecipe, RecipeStore};
use intent_learners::*;

/// Recipe learner
pub struct RecipeLearner {
    candidates: LearningCandidates,
    store: RecipeStore,
}

impl RecipeLearner {
    pub fn new() -> Self {
        Self {
            candidates: LearningCandidates::load(),
            store: RecipeStore::load(),
        }
    }

    /// Record a successful ticket
    pub fn record_success(&mut self, observation: TicketObservation) {
        let intent = observation.intent;
        self.candidates.record(observation);

        // Check if we can learn a recipe
        if self.candidates.ready_to_learn(intent) && !self.has_recipe(intent) {
            if let Some(recipe) = self.try_learn_recipe(intent) {
                self.store.upsert(recipe);
                let _ = self.store.save();
            }
        }

        let _ = self.candidates.save();
    }

    /// Check if we already have a recipe for this intent
    pub fn has_recipe(&self, intent: CanonicalIntent) -> bool {
        self.store.find_for_intent(intent).is_some()
    }

    /// Get recipe for intent
    pub fn get_recipe(&self, intent: CanonicalIntent) -> Option<&LearnedRecipe> {
        self.store.find_for_intent(intent)
    }

    /// Try to learn a recipe from observations
    fn try_learn_recipe(&self, intent: CanonicalIntent) -> Option<LearnedRecipe> {
        let observations = self.candidates.get_observations(intent);
        if observations.len() < 2 {
            return None;
        }

        // Find common probes across observations
        let common_probes = find_common_probes(&observations);
        if common_probes.is_empty() {
            return None;
        }

        // Try to create recipe based on intent type
        let recipe = match intent {
            CanonicalIntent::CheckDiskUsage => {
                learn_disk_usage_recipe(&observations, &common_probes)
            }
            CanonicalIntent::CheckFreeRam => learn_memory_recipe(&observations, &common_probes),
            CanonicalIntent::CheckSwapPresence => learn_swap_recipe(&observations, &common_probes),
            CanonicalIntent::CheckFailedServices => {
                learn_failed_services_recipe(&observations, &common_probes)
            }
            CanonicalIntent::CheckUptime => learn_uptime_recipe(&observations, &common_probes),
            CanonicalIntent::CheckBootTime => learn_boot_time_recipe(&observations, &common_probes),
            _ => learn_generic_recipe(intent, &observations, &common_probes),
        };

        recipe
    }

    /// Record recipe usage
    pub fn record_recipe_result(&mut self, recipe_id: &str, success: bool, confidence: f32) {
        if let Some(recipe) = self.store.get_mut(recipe_id) {
            if success {
                recipe.stats.record_success(confidence);
            } else {
                recipe.stats.record_failure();
            }
            recipe.last_used_at = current_secs();

            // Auto-deprecate if success rate drops
            if recipe.stats.uses >= 10 && recipe.stats.success_rate() < 0.5 {
                recipe.deprecated = true;
            }
        }
        let _ = self.store.save();
    }

    /// Get store reference
    pub fn store(&self) -> &RecipeStore {
        &self.store
    }
}

impl Default for RecipeLearner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_observation_recording() {
        let mut candidates = LearningCandidates::new();

        let obs = TicketObservation {
            ticket_id: "TEST-001".to_string(),
            intent: CanonicalIntent::CheckDiskUsage,
            domain: "storage".to_string(),
            probes_used: vec!["disk_usage".to_string()],
            probe_outputs: HashMap::new(),
            answer_summary: "Disk at 50%".to_string(),
            confidence: 0.9,
            successful: true,
            timestamp: 0,
        };

        candidates.record(obs);
        assert!(!candidates.ready_to_learn(CanonicalIntent::CheckDiskUsage)); // Need 2+
    }

    #[test]
    fn test_common_probes() {
        let obs1 = TicketObservation {
            ticket_id: "T1".to_string(),
            intent: CanonicalIntent::CheckDiskUsage,
            domain: "storage".to_string(),
            probes_used: vec!["disk_usage".to_string(), "block_devices".to_string()],
            probe_outputs: HashMap::new(),
            answer_summary: "".to_string(),
            confidence: 0.9,
            successful: true,
            timestamp: 0,
        };

        let obs2 = TicketObservation {
            ticket_id: "T2".to_string(),
            intent: CanonicalIntent::CheckDiskUsage,
            domain: "storage".to_string(),
            probes_used: vec!["disk_usage".to_string()],
            probe_outputs: HashMap::new(),
            answer_summary: "".to_string(),
            confidence: 0.9,
            successful: true,
            timestamp: 0,
        };

        let observations = vec![&obs1, &obs2];
        let common = find_common_probes(&observations);

        assert_eq!(common, vec!["disk_usage".to_string()]);
    }
}
