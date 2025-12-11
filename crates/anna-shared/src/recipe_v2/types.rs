//! RecipeV2 - The main recipe type (v0.0.420).

use serde::{Deserialize, Serialize};

use super::{
    FactRequirement, RecipeDomain, RecipeStats, RecipeStepV2, TriggerPattern,
};
use crate::specialist_contract::KnowledgeCitation;

/// A reusable troubleshooting or configuration pattern (v0.0.420).
///
/// Recipes are learned from successful tickets and can be matched to future
/// queries to skip specialist calls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeV2 {
    /// Unique identifier like "vim.syntax.enable", "system.boot.slow.service"
    pub id: String,
    /// Version number, incremented when updated
    #[serde(default = "default_version")]
    pub version: u32,
    /// Human-readable title
    pub title: String,
    /// Domain classification
    #[serde(default)]
    pub domain: RecipeDomain,
    /// Patterns that trigger this recipe
    pub trigger_patterns: Vec<TriggerPattern>,
    /// Facts that must be satisfied for this recipe to apply
    #[serde(default)]
    pub required_facts: Vec<FactRequirement>,
    /// Steps to execute
    pub steps: Vec<RecipeStepV2>,
    /// Citations from knowledge sources (man pages, wiki, etc.)
    #[serde(default)]
    pub citations: Vec<String>,
    /// Knowledge citations with full provenance
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub knowledge_citations: Vec<KnowledgeCitation>,
    /// Usage statistics
    #[serde(default)]
    pub stats: RecipeStats,
    /// Unix timestamp when created
    #[serde(default)]
    pub created_at: u64,
    /// Unix timestamp when last updated
    #[serde(default)]
    pub updated_at: u64,
    /// Whether this is a global (shipped) recipe vs user-learned
    #[serde(default)]
    pub is_global: bool,
    /// Whether this recipe is currently enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_version() -> u32 {
    1
}

fn default_enabled() -> bool {
    true
}

impl RecipeV2 {
    /// Create a new recipe
    pub fn new(id: &str, title: &str, domain: RecipeDomain) -> Self {
        let now = current_timestamp();
        Self {
            id: id.to_string(),
            version: 1,
            title: title.to_string(),
            domain,
            trigger_patterns: Vec::new(),
            required_facts: Vec::new(),
            steps: Vec::new(),
            citations: Vec::new(),
            knowledge_citations: Vec::new(),
            stats: RecipeStats::new(),
            created_at: now,
            updated_at: now,
            is_global: false,
            enabled: true,
        }
    }

    /// Add a trigger pattern
    pub fn with_trigger(mut self, trigger: TriggerPattern) -> Self {
        self.trigger_patterns.push(trigger);
        self
    }

    /// Add a fact requirement
    pub fn with_fact(mut self, fact: FactRequirement) -> Self {
        self.required_facts.push(fact);
        self
    }

    /// Add a step
    pub fn with_step(mut self, step: RecipeStepV2) -> Self {
        self.steps.push(step);
        self
    }

    /// Add a citation
    pub fn with_citation(mut self, citation: &str) -> Self {
        self.citations.push(citation.to_string());
        self
    }

    /// Mark as global recipe
    pub fn global(mut self) -> Self {
        self.is_global = true;
        self
    }

    /// Check if recipe is available for matching
    pub fn is_available(&self) -> bool {
        self.enabled && !self.stats.should_disable()
    }

    /// Check if recipe needs user confirmation for any step
    pub fn needs_confirmation(&self) -> bool {
        self.steps.iter().any(|s| s.kind.requires_confirmation())
    }

    /// Check if recipe makes any changes
    pub fn is_mutating(&self) -> bool {
        self.steps.iter().any(|s| s.kind.is_mutating())
    }

    /// Get the best trigger pattern score for given intent/keywords
    pub fn best_trigger_score(&self, intent: &str, keywords: &[String]) -> f32 {
        self.trigger_patterns
            .iter()
            .map(|t| t.match_score(intent, keywords))
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    /// Get minimum confidence threshold from triggers
    pub fn min_confidence(&self) -> f32 {
        self.trigger_patterns
            .iter()
            .map(|t| t.min_confidence)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.7)
    }

    /// Increment version
    pub fn bump_version(&mut self) {
        self.version += 1;
        self.updated_at = current_timestamp();
    }

    /// Record successful execution
    pub fn record_success(&mut self, duration_ms: u64) {
        self.stats.record_success(duration_ms);
    }

    /// Record failed execution
    pub fn record_failure(&mut self, duration_ms: u64) {
        self.stats.record_failure(duration_ms);
    }

    /// Get file path for this recipe
    pub fn file_path(&self, base_dir: &std::path::Path) -> std::path::PathBuf {
        base_dir
            .join(self.domain.subdir())
            .join(format!("{}.json", self.id))
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe_v2::step::{RecipeStepAction, RecipeStepKind, RecipeStepV2};

    #[test]
    fn test_recipe_creation() {
        let recipe = RecipeV2::new("test.recipe", "Test Recipe", RecipeDomain::Generic)
            .with_trigger(TriggerPattern::new("test_intent", vec!["test", "recipe"]))
            .with_step(RecipeStepV2::probe("Test probe", "echo test"));

        assert_eq!(recipe.id, "test.recipe");
        assert_eq!(recipe.trigger_patterns.len(), 1);
        assert_eq!(recipe.steps.len(), 1);
    }

    #[test]
    fn test_needs_confirmation() {
        let mut recipe = RecipeV2::new("test", "Test", RecipeDomain::Generic);

        // No confirmation needed for probe-only
        recipe.steps.push(RecipeStepV2::probe("probe", "echo"));
        assert!(!recipe.needs_confirmation());

        // Confirmation needed for risky change
        recipe.steps.push(RecipeStepV2::risky_change(
            "risky",
            RecipeStepAction::run_command("rm -rf /"),
        ));
        assert!(recipe.needs_confirmation());
    }
}
