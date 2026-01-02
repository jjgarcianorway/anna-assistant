//! Core recipe types and data structures (v0.0.427).
//!
//! Defines the fundamental types used throughout the recipe system:
//! - LearnedRecipe: Main recipe structure
//! - RecipeInputs: Input parameter definitions
//! - RecipeProbe: Probe execution definitions
//! - RecipeLogic: Logic and conditional branching
//! - RecipeSafety: Safety and risk information
//! - RecipeOrigin: Source and citation tracking
//! - RecipeUsageStats: Usage statistics and reliability metrics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A learned or seed recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedRecipe {
    /// Unique stable identifier
    pub id: String,
    /// Domain classification
    pub domain: String,
    /// Pattern for matching
    pub pattern: super::RecipePattern,
    /// Input parameters
    pub inputs: RecipeInputs,
    /// Probes to run
    pub probes: Vec<RecipeProbe>,
    /// Logic/steps to follow
    pub logic: RecipeLogic,
    /// Answer templates
    pub answer_template: super::AnswerTemplate,
    /// Safety information
    pub safety: RecipeSafety,
    /// Origin/source information
    pub origin: RecipeOrigin,
    /// Usage statistics
    pub stats: RecipeUsageStats,
    /// Recipe version
    pub version: u32,
    /// Whether recipe is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

impl Default for LearnedRecipe {
    fn default() -> Self {
        Self {
            id: String::new(),
            domain: "general".to_string(),
            pattern: super::RecipePattern::default(),
            inputs: RecipeInputs::default(),
            probes: vec![],
            logic: RecipeLogic {
                logic_type: LogicType::Template,
                answer_kind: AnswerKind::Diagnostic,
                steps: vec![],
                conditionals: HashMap::new(),
            },
            answer_template: super::AnswerTemplate::default(),
            safety: RecipeSafety::default(),
            origin: RecipeOrigin::default(),
            stats: RecipeUsageStats::default(),
            version: 1,
            enabled: true,
        }
    }
}

/// Input parameters for a recipe
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecipeInputs {
    /// Parameter definitions (name -> description)
    /// Use "?" suffix for optional params (e.g., "service_name?")
    pub params: HashMap<String, String>,
    /// Whether to require user confirmation before execution
    #[serde(default)]
    pub requires_confirmation: bool,
}

impl RecipeInputs {
    /// Add a required parameter
    pub fn with_param(mut self, name: &str, description: &str) -> Self {
        self.params
            .insert(name.to_string(), description.to_string());
        self
    }

    /// Add an optional parameter (name ends with ?)
    pub fn with_optional_param(mut self, name: &str, description: &str) -> Self {
        let key = if name.ends_with('?') {
            name.to_string()
        } else {
            format!("{}?", name)
        };
        self.params.insert(key, description.to_string());
        self
    }

    /// Check if a param is optional
    pub fn is_optional(&self, name: &str) -> bool {
        name.ends_with('?') || self.params.contains_key(&format!("{}?", name))
    }

    /// Get required param names (no ? suffix)
    pub fn required_params(&self) -> Vec<String> {
        self.params
            .keys()
            .filter(|k| !k.ends_with('?'))
            .cloned()
            .collect()
    }
}

/// A probe to run as part of a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeProbe {
    /// Probe identifier
    pub id: String,
    /// Tool/command to run
    pub tool: String,
    /// Parameters to pass (may reference recipe params via {{param_name}})
    #[serde(default)]
    pub params: Vec<String>,
    /// Whether this probe is optional
    #[serde(default)]
    pub optional: bool,
    /// Timeout in milliseconds
    #[serde(default = "default_probe_timeout")]
    pub timeout_ms: u64,
}

fn default_probe_timeout() -> u64 {
    5000
}

impl RecipeProbe {
    /// Create a new required probe
    pub fn new(id: &str, tool: &str) -> Self {
        Self {
            id: id.to_string(),
            tool: tool.to_string(),
            params: vec![],
            optional: false,
            timeout_ms: default_probe_timeout(),
        }
    }

    /// Make optional
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Add parameter
    pub fn with_param(mut self, param: &str) -> Self {
        self.params.push(param.to_string());
        self
    }
}

/// Logic/steps for a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeLogic {
    /// Type of logic (template, conditional, etc.)
    pub logic_type: LogicType,
    /// Kind of answer (diagnostic, fix, explanation)
    pub answer_kind: AnswerKind,
    /// Human-readable steps
    pub steps: Vec<String>,
    /// Conditional branches (probe_id -> condition -> step)
    #[serde(default)]
    pub conditionals: HashMap<String, Vec<ConditionalBranch>>,
}

/// Type of recipe logic
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LogicType {
    /// Simple template-based answer
    #[default]
    Template,
    /// Conditional logic based on probe results
    Conditional,
    /// Sequential steps with decisions
    Sequential,
}

/// Kind of answer the recipe produces
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AnswerKind {
    /// Diagnostic information
    #[default]
    Diagnostic,
    /// Actionable fix
    Fix,
    /// Explanation/education
    Explanation,
    /// Status check
    Status,
}

/// A conditional branch in recipe logic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalBranch {
    /// Condition to check (e.g., "contains:failed", "empty", "matches:.*running.*")
    pub condition: String,
    /// Action to take if condition is true
    pub action: String,
    /// Template to use for answer
    #[serde(default)]
    pub answer_template: Option<String>,
}

/// Safety information for a recipe
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecipeSafety {
    /// Risk level
    pub risk: RiskLevel,
    /// Whether a backup is recommended
    #[serde(default)]
    pub needs_backup: bool,
    /// Whether sudo/root is required
    #[serde(default)]
    pub requires_sudo: bool,
    /// Warning message to show user
    #[serde(default)]
    pub warning: Option<String>,
}

/// Risk level for recipes
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// Safe read-only operations
    #[default]
    Low,
    /// Minor changes, easily reversible
    Medium,
    /// Significant changes, may need manual intervention
    High,
}

/// Origin/source information for a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeOrigin {
    /// Source ticket ID (if learned)
    #[serde(default)]
    pub created_from_ticket: Option<String>,
    /// Who/what created this recipe
    pub created_by: String,
    /// Creation timestamp (ISO 8601)
    pub created_at: String,
    /// Sources cited (man pages, wiki, help output)
    #[serde(default)]
    pub sources: Vec<String>,
    /// Whether this is a seed recipe
    #[serde(default)]
    pub is_seed: bool,
}

impl Default for RecipeOrigin {
    fn default() -> Self {
        Self {
            created_from_ticket: None,
            created_by: "system".to_string(),
            created_at: super::utils::now_iso8601(),
            sources: vec![],
            is_seed: false,
        }
    }
}

impl RecipeOrigin {
    /// Create origin for learned recipe
    pub fn learned(ticket_id: &str, specialist: &str) -> Self {
        Self {
            created_from_ticket: Some(ticket_id.to_string()),
            created_by: specialist.to_string(),
            created_at: super::utils::now_iso8601(),
            sources: vec![],
            is_seed: false,
        }
    }

    /// Create origin for seed recipe
    pub fn seed() -> Self {
        Self {
            created_from_ticket: None,
            created_by: "seed".to_string(),
            created_at: super::utils::now_iso8601(),
            sources: vec![],
            is_seed: true,
        }
    }

    /// Add a source citation
    pub fn with_source(mut self, source: &str) -> Self {
        self.sources.push(source.to_string());
        self
    }
}

/// Usage statistics for a recipe
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecipeUsageStats {
    /// Total times used
    pub uses: u32,
    /// Successful uses
    pub successes: u32,
    /// Failed uses
    pub failures: u32,
    /// Last used timestamp (ISO 8601)
    #[serde(default)]
    pub last_used_at: Option<String>,
}

impl RecipeUsageStats {
    /// Record a successful use
    pub fn record_success(&mut self) {
        self.uses += 1;
        self.successes += 1;
        self.last_used_at = Some(super::utils::now_iso8601());
    }

    /// Record a failed use
    pub fn record_failure(&mut self) {
        self.uses += 1;
        self.failures += 1;
        self.last_used_at = Some(super::utils::now_iso8601());
    }

    /// Get success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f32 {
        if self.uses == 0 {
            0.0
        } else {
            self.successes as f32 / self.uses as f32
        }
    }

    /// Check if recipe is reliable (enough uses + good success rate)
    pub fn is_reliable(&self) -> bool {
        self.uses >= crate::learning_engine::MIN_RELIABLE_USES && self.success_rate() >= 0.7
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_stats() {
        let mut stats = RecipeUsageStats::default();
        stats.record_success();
        stats.record_success();
        stats.record_failure();

        assert_eq!(stats.uses, 3);
        assert_eq!(stats.successes, 2);
        assert!((stats.success_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_inputs_params() {
        let inputs = RecipeInputs::default()
            .with_param("service_name", "Name of the service")
            .with_optional_param("limit", "Log line limit");

        assert!(!inputs.is_optional("service_name"));
        assert!(inputs.is_optional("limit"));
        assert_eq!(inputs.required_params().len(), 1);
    }
}
