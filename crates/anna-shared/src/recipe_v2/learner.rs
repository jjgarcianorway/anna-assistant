//! Recipe learning from successful tickets (v0.0.420).
//!
//! Learning conditions:
//! - ticket.status == "resolved"
//! - ticket.confidence >= 0.9
//! - Actions are deterministic and safe/guarded
//! - Clear intent and keywords from translator
//!
//! Conservative learning: Only from high-confidence verified tickets.

use std::collections::HashMap;

use super::{
    FactRequirement, RecipeDomain, RecipeStepAction, RecipeStepV2, RecipeStorageV2, RecipeV2,
    TriggerPattern, LEARNING_THRESHOLD,
};

/// Observation from a successful ticket
#[derive(Debug, Clone)]
pub struct TicketObservation {
    /// Intent from translator
    pub intent: String,
    /// Keywords from translator
    pub keywords: Vec<String>,
    /// Domain (from specialist)
    pub domain: String,
    /// Probes that were used
    pub probes_used: Vec<String>,
    /// Answer template
    pub answer: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Whether the ticket was verified successful
    pub verified: bool,
    /// Facts that were collected during resolution
    pub facts: HashMap<String, String>,
    /// Citations used
    pub citations: Vec<String>,
}

impl TicketObservation {
    /// Check if this observation is eligible for learning
    pub fn is_learnable(&self) -> bool {
        self.verified && self.confidence >= LEARNING_THRESHOLD && !self.intent.is_empty()
    }

    /// Generate a candidate recipe ID from intent
    pub fn candidate_recipe_id(&self) -> String {
        let domain = RecipeDomain::from_str(&self.domain);
        let intent_parts: Vec<&str> = self.intent.split('_').collect();

        // Build ID like "memory.show_free" or "vim.syntax.enable"
        if intent_parts.len() >= 2 {
            format!("{}.{}", domain.subdir(), intent_parts.join("."))
        } else {
            format!("{}.{}", domain.subdir(), self.intent)
        }
    }
}

/// Recipe learner - learns from successful tickets
pub struct RecipeLearner {
    /// Observations grouped by intent
    observations: HashMap<String, Vec<TicketObservation>>,
    /// Minimum observations before learning
    min_observations: usize,
    /// Minimum success rate to trigger learning
    min_success_rate: f32,
}

impl Default for RecipeLearner {
    fn default() -> Self {
        Self::new()
    }
}

impl RecipeLearner {
    /// Create a new learner
    pub fn new() -> Self {
        Self {
            observations: HashMap::new(),
            min_observations: 2,
            min_success_rate: 0.80,
        }
    }

    /// Record an observation from a ticket
    pub fn record(&mut self, obs: TicketObservation) {
        if obs.is_learnable() {
            let intent = obs.intent.clone();
            self.observations
                .entry(intent.clone())
                .or_default()
                .push(obs);

            // Keep only last 20 observations per intent
            if let Some(list) = self.observations.get_mut(&intent) {
                if list.len() > 20 {
                    list.remove(0);
                }
            }
        }
    }

    /// Check if ready to learn a recipe for an intent
    pub fn ready_to_learn(&self, intent: &str) -> bool {
        if let Some(obs) = self.observations.get(intent) {
            if obs.len() >= self.min_observations {
                let success_rate =
                    obs.iter().filter(|o| o.verified).count() as f32 / obs.len() as f32;
                return success_rate >= self.min_success_rate;
            }
        }
        false
    }

    /// Try to learn a recipe from observations
    pub fn try_learn(&self, intent: &str) -> Option<RecipeV2> {
        if !self.ready_to_learn(intent) {
            return None;
        }

        let obs_list = self.observations.get(intent)?;
        let best_obs = obs_list.iter().max_by(|a, b| {
            a.confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        })?;

        Some(observation_to_recipe(best_obs))
    }

    /// Learn a recipe and save to storage
    pub fn learn_and_save(
        &self,
        intent: &str,
        storage: &mut RecipeStorageV2,
    ) -> Result<RecipeV2, String> {
        let recipe = self.try_learn(intent).ok_or("Not ready to learn")?;

        // Check if recipe already exists
        if let Some(existing) = storage.get(&recipe.id) {
            // Update existing recipe's stats rather than replacing
            if let Some(existing_mut) = storage.get_mut(&recipe.id) {
                existing_mut.stats.record_success(0);
                return Ok(existing_mut.clone());
            }
        }

        storage.upsert(recipe.clone())?;
        Ok(recipe)
    }

    /// Get all observations for an intent
    pub fn get_observations(&self, intent: &str) -> Option<&Vec<TicketObservation>> {
        self.observations.get(intent)
    }

    /// Clear observations for an intent
    pub fn clear_observations(&mut self, intent: &str) {
        self.observations.remove(intent);
    }

    /// Clear all observations
    pub fn clear_all(&mut self) {
        self.observations.clear();
    }
}

/// Convert a ticket observation to a recipe
fn observation_to_recipe(obs: &TicketObservation) -> RecipeV2 {
    let domain = RecipeDomain::from_str(&obs.domain);
    let recipe_id = obs.candidate_recipe_id();

    // Build title from intent
    let title = obs
        .intent
        .replace('_', " ")
        .split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let mut recipe = RecipeV2::new(&recipe_id, &title, domain).with_trigger(TriggerPattern::new(
        &obs.intent,
        obs.keywords.iter().map(|s| s.as_str()).collect(),
    ));

    // Add probe steps
    for probe in &obs.probes_used {
        recipe = recipe.with_step(RecipeStepV2::probe(&format!("Run {}", probe), probe));
    }

    // Add explanation step with answer template
    if !obs.answer.is_empty() {
        recipe = recipe.with_step(RecipeStepV2::explain("Explain result", &obs.answer));
    }

    // Add citations
    for citation in &obs.citations {
        recipe = recipe.with_citation(citation);
    }

    // Add fact requirements (only include facts that seem important)
    for (key, value) in &obs.facts {
        if is_important_fact(key) {
            recipe = recipe.with_fact(FactRequirement::eq(key, value));
        }
    }

    recipe
}

/// Check if a fact key is important enough to include as a requirement
fn is_important_fact(key: &str) -> bool {
    matches!(
        key,
        "editor" | "shell" | "os" | "package_manager" | "init_system"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_observation(intent: &str, verified: bool, confidence: f32) -> TicketObservation {
        TicketObservation {
            intent: intent.to_string(),
            keywords: vec!["test".to_string()],
            domain: "generic".to_string(),
            probes_used: vec!["test_probe".to_string()],
            answer: "Test answer".to_string(),
            confidence,
            verified,
            facts: HashMap::new(),
            citations: vec!["test:citation".to_string()],
        }
    }

    #[test]
    fn test_learnable_check() {
        let obs = make_observation("test_intent", true, 0.95);
        assert!(obs.is_learnable());

        let obs = make_observation("test_intent", false, 0.95);
        assert!(!obs.is_learnable());

        let obs = make_observation("test_intent", true, 0.5);
        assert!(!obs.is_learnable());
    }

    #[test]
    fn test_ready_to_learn() {
        let mut learner = RecipeLearner::new();

        // Not ready with 0 observations
        assert!(!learner.ready_to_learn("test"));

        // Not ready with 1 observation
        learner.record(make_observation("test", true, 0.95));
        assert!(!learner.ready_to_learn("test"));

        // Ready with 2 observations
        learner.record(make_observation("test", true, 0.92));
        assert!(learner.ready_to_learn("test"));
    }

    #[test]
    fn test_try_learn() {
        let mut learner = RecipeLearner::new();

        learner.record(make_observation("show_memory", true, 0.95));
        learner.record(make_observation("show_memory", true, 0.92));

        let recipe = learner.try_learn("show_memory");
        assert!(recipe.is_some());

        let r = recipe.unwrap();
        assert!(r.id.contains("show.memory"));
        assert!(!r.steps.is_empty());
    }

    #[test]
    fn test_candidate_recipe_id() {
        let obs = TicketObservation {
            intent: "show_free_memory".to_string(),
            keywords: vec![],
            domain: "performance".to_string(),
            probes_used: vec![],
            answer: String::new(),
            confidence: 0.95,
            verified: true,
            facts: HashMap::new(),
            citations: vec![],
        };

        assert_eq!(obs.candidate_recipe_id(), "performance.show.free.memory");
    }
}
