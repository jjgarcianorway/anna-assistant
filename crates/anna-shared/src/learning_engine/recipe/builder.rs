//! Builder methods for LearnedRecipe (v0.0.427).
//!
//! Provides fluent builder API for constructing recipes with:
//! - Pattern configuration
//! - Probe addition
//! - Answer template setup
//! - Safety and origin configuration

use super::{AnswerTemplate, LearnedRecipe, RecipeOrigin, RecipePattern, RecipeProbe, RecipeSafety};

impl LearnedRecipe {
    /// Create a new recipe with ID
    pub fn new(id: &str, domain: &str) -> Self {
        Self {
            id: id.to_string(),
            domain: domain.to_string(),
            ..Default::default()
        }
    }

    /// Builder: set pattern
    pub fn with_pattern(mut self, pattern: RecipePattern) -> Self {
        self.pattern = pattern;
        self
    }

    /// Builder: add probe
    pub fn with_probe(mut self, probe: RecipeProbe) -> Self {
        self.probes.push(probe);
        self
    }

    /// Builder: set answer template
    pub fn with_answer(mut self, short: &str, detailed: &str) -> Self {
        self.answer_template = AnswerTemplate::new(short, detailed);
        self
    }

    /// Builder: set safety
    pub fn with_safety(mut self, safety: RecipeSafety) -> Self {
        self.safety = safety;
        self
    }

    /// Builder: set origin
    pub fn with_origin(mut self, origin: RecipeOrigin) -> Self {
        self.origin = origin;
        self
    }

    /// Builder: add step
    pub fn with_step(mut self, step: &str) -> Self {
        self.logic.steps.push(step.to_string());
        self
    }

    /// Check if recipe is healthy (good success rate or not enough data)
    pub fn is_healthy(&self) -> bool {
        self.stats.uses < crate::learning_engine::MIN_RELIABLE_USES
            || self.stats.success_rate() >= 0.6
    }

    /// Check if recipe is mature (enough usage data)
    pub fn is_mature(&self) -> bool {
        self.stats.uses >= crate::learning_engine::MIN_RELIABLE_USES
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_creation() {
        let recipe = LearnedRecipe::new("check-ram", "performance.memory")
            .with_pattern(
                RecipePattern::new("check_free_ram").with_keywords(&["ram", "memory", "free"]),
            )
            .with_probe(RecipeProbe::new("free", "probe.free"))
            .with_answer(
                "Available RAM: {{available_mb}} MB",
                "Memory status:\n  Total: {{total_mb}} MB\n  Used: {{used_mb}} MB\n  Available: {{available_mb}} MB",
            );

        assert_eq!(recipe.id, "check-ram");
        assert_eq!(recipe.pattern.intent, "check_free_ram");
        assert_eq!(recipe.probes.len(), 1);
    }
}
