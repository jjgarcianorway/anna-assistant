//! Recipe struct and core implementation.

use super::step::RecipeStep;
use super::types::{EvidenceRequirement, RecipeKind, RecipeParameter, RiskLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A learned recipe - deterministic, replayable solution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    /// Stable internal ID
    pub id: String,
    /// Human-friendly name
    pub name: String,
    /// Recipe kind
    pub kind: RecipeKind,
    /// Domain (Desktop, Network, Storage, etc.)
    pub domain: String,
    /// Natural language intent pattern
    pub intent_pattern: String,
    /// Tags for matching
    pub tags: Vec<String>,
    /// Trigger patterns for matching
    pub trigger_patterns: Vec<String>,
    /// Evidence requirements
    pub required_evidence: Vec<EvidenceRequirement>,
    /// Recipe steps
    pub steps: Vec<RecipeStep>,
    /// Recipe parameters (variables like {{service_name}})
    pub parameters: Vec<RecipeParameter>,
    /// Risk level
    pub risk_level: RiskLevel,
    /// Ticket ID that created this recipe
    pub created_from_ticket: Option<String>,
    /// Creation timestamp (Unix secs)
    pub created_at: u64,
    /// Last used timestamp
    pub last_used_at: u64,
    /// Total use count
    pub use_count: u32,
    /// Successful executions
    pub success_count: u32,
    /// Failed executions
    pub failure_count: u32,
    /// Baseline confidence for reuse
    pub confidence_baseline: f32,
    /// Documentation sources
    pub doc_sources: Vec<String>,
    /// Recipe version
    pub version: u32,
    /// Whether recipe is deprecated
    pub deprecated: bool,
    /// Recipe IDs this supersedes
    pub supersedes: Vec<String>,
}

impl Recipe {
    /// Create a new recipe
    pub fn new(id: &str, name: &str, kind: RecipeKind, domain: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            kind,
            domain: domain.to_string(),
            intent_pattern: String::new(),
            tags: vec![],
            trigger_patterns: vec![],
            required_evidence: vec![],
            steps: vec![],
            parameters: vec![],
            risk_level: RiskLevel::ReadOnly,
            created_from_ticket: None,
            created_at: current_secs(),
            last_used_at: 0,
            use_count: 0,
            success_count: 0,
            failure_count: 0,
            confidence_baseline: 0.8,
            doc_sources: vec![],
            version: 1,
            deprecated: false,
            supersedes: vec![],
        }
    }

    /// Builder: set intent pattern
    pub fn with_intent(mut self, pattern: &str) -> Self {
        self.intent_pattern = pattern.to_string();
        self
    }

    /// Builder: add tags
    pub fn with_tags(mut self, tags: Vec<&str>) -> Self {
        self.tags = tags.into_iter().map(String::from).collect();
        self
    }

    /// Builder: add trigger patterns
    pub fn with_triggers(mut self, patterns: Vec<&str>) -> Self {
        self.trigger_patterns = patterns.into_iter().map(String::from).collect();
        self
    }

    /// Builder: set evidence requirements
    pub fn with_evidence(mut self, reqs: Vec<EvidenceRequirement>) -> Self {
        self.required_evidence = reqs;
        self
    }

    /// Builder: add steps
    pub fn with_steps(mut self, steps: Vec<RecipeStep>) -> Self {
        self.steps = steps;
        self
    }

    /// Builder: add parameters
    pub fn with_params(mut self, params: Vec<RecipeParameter>) -> Self {
        self.parameters = params;
        self
    }

    /// Builder: set doc sources
    pub fn with_docs(mut self, docs: Vec<&str>) -> Self {
        self.doc_sources = docs.into_iter().map(String::from).collect();
        self
    }

    /// Builder: set created from ticket
    pub fn from_ticket(mut self, ticket_id: &str) -> Self {
        self.created_from_ticket = Some(ticket_id.to_string());
        self
    }

    /// Record successful use
    pub fn record_success(&mut self) {
        self.use_count += 1;
        self.success_count += 1;
        self.last_used_at = current_secs();
        // Slightly boost confidence on success
        self.confidence_baseline = (self.confidence_baseline + 0.01).min(0.99);
    }

    /// Record failed use
    pub fn record_failure(&mut self) {
        self.use_count += 1;
        self.failure_count += 1;
        self.last_used_at = current_secs();
        // Slightly reduce confidence on failure
        self.confidence_baseline = (self.confidence_baseline - 0.05).max(0.3);
    }

    /// Calculate success rate
    pub fn success_rate(&self) -> f32 {
        if self.use_count == 0 {
            return 1.0;
        }
        self.success_count as f32 / self.use_count as f32
    }

    /// Check if recipe should be deprecated
    pub fn should_deprecate(&self) -> bool {
        self.use_count >= 10 && self.success_rate() < 0.6
    }

    /// Check if recipe is active (not deprecated, reasonable success)
    pub fn is_active(&self) -> bool {
        !self.deprecated && (self.use_count < 3 || self.success_rate() >= 0.5)
    }

    /// Check if this is a read-only recipe
    pub fn is_read_only(&self) -> bool {
        self.steps.iter().all(|s| s.is_read_only())
    }

    /// Check if any step requires confirmation
    pub fn requires_confirmation(&self) -> bool {
        self.steps.iter().any(|s| s.requires_confirmation())
    }

    /// Get required parameter names
    pub fn required_params(&self) -> Vec<&str> {
        self.parameters
            .iter()
            .filter(|p| p.required)
            .map(|p| p.name.as_str())
            .collect()
    }

    /// Substitute parameters in a command template
    pub fn substitute_params(&self, template: &str, values: &HashMap<String, String>) -> String {
        let mut result = template.to_string();
        for param in &self.parameters {
            let placeholder = format!("{{{{{}}}}}", param.name);
            if let Some(value) = values.get(&param.name) {
                result = result.replace(&placeholder, value);
            } else if let Some(default) = &param.default {
                result = result.replace(&placeholder, default);
            }
        }
        result
    }
}

fn current_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_creation() {
        let recipe = Recipe::new(
            "svc-status",
            "Service Status Check",
            RecipeKind::Inspect,
            "services",
        )
        .with_intent("check status of a systemd service")
        .with_tags(vec!["systemd", "service", "status"])
        .with_triggers(vec!["service status", "is service running"]);

        assert_eq!(recipe.id, "svc-status");
        assert_eq!(recipe.kind, RecipeKind::Inspect);
        assert!(recipe.is_read_only());
    }

    #[test]
    fn test_recipe_success_rate() {
        let mut recipe = Recipe::new("test", "Test", RecipeKind::ProbeOnly, "system");
        recipe.use_count = 10;
        recipe.success_count = 8;
        recipe.failure_count = 2;

        assert!((recipe.success_rate() - 0.8).abs() < 0.01);
        assert!(!recipe.should_deprecate());
    }

    #[test]
    fn test_param_substitution() {
        let recipe =
            Recipe::new("test", "Test", RecipeKind::Inspect, "services").with_params(vec![
                RecipeParameter {
                    name: "service_name".to_string(),
                    description: "Name of service".to_string(),
                    extraction_hint: "word before 'service'".to_string(),
                    default: None,
                    required: true,
                },
            ]);

        let mut values = HashMap::new();
        values.insert("service_name".to_string(), "nginx".to_string());

        let result = recipe.substitute_params("systemctl status {{service_name}}", &values);
        assert_eq!(result, "systemctl status nginx");
    }
}
