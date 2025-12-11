//! Recipe schema for learning engine (v0.0.427).
//!
//! Defines the core recipe structure with:
//! - Pattern matching (intent + keywords + required signals)
//! - Probe definitions
//! - Answer templates (short and detailed)
//! - Safety flags
//! - Origin tracking with citations
//! - Usage statistics

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
    pub pattern: RecipePattern,
    /// Input parameters
    pub inputs: RecipeInputs,
    /// Probes to run
    pub probes: Vec<RecipeProbe>,
    /// Logic/steps to follow
    pub logic: RecipeLogic,
    /// Answer templates
    pub answer_template: AnswerTemplate,
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

/// Pattern for matching questions to recipes
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecipePattern {
    /// Primary intent (e.g., "debug_failed_service", "check_free_ram")
    pub intent: String,
    /// Keywords that should appear in the question
    pub keywords: Vec<String>,
    /// Required probe signals (e.g., ["probe:systemd_failed_units"])
    #[serde(default)]
    pub required_signals: Vec<String>,
    /// Optional signals that improve match quality
    #[serde(default)]
    pub optional_signals: Vec<String>,
}

impl RecipePattern {
    /// Create a new pattern with intent
    pub fn new(intent: &str) -> Self {
        Self {
            intent: intent.to_string(),
            ..Default::default()
        }
    }

    /// Add keywords
    pub fn with_keywords(mut self, keywords: &[&str]) -> Self {
        self.keywords = keywords.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add required signals
    pub fn with_required_signals(mut self, signals: &[&str]) -> Self {
        self.required_signals = signals.iter().map(|s| s.to_string()).collect();
        self
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
        self.params.insert(name.to_string(), description.to_string());
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

/// Answer templates for a recipe
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnswerTemplate {
    /// Short one-line answer
    pub short: String,
    /// Detailed answer with explanation
    pub detailed: String,
    /// Variables available for substitution
    #[serde(default)]
    pub variables: Vec<String>,
}

impl AnswerTemplate {
    /// Create a new template
    pub fn new(short: &str, detailed: &str) -> Self {
        Self {
            short: short.to_string(),
            detailed: detailed.to_string(),
            variables: vec![],
        }
    }

    /// Add available variable
    pub fn with_variable(mut self, var: &str) -> Self {
        self.variables.push(var.to_string());
        self
    }

    /// Render short template with values
    pub fn render_short(&self, values: &HashMap<String, String>) -> String {
        substitute_template(&self.short, values)
    }

    /// Render detailed template with values
    pub fn render_detailed(&self, values: &HashMap<String, String>) -> String {
        substitute_template(&self.detailed, values)
    }
}

/// Substitute {{variable}} placeholders in template
fn substitute_template(template: &str, values: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in values {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
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
            created_at: now_iso8601(),
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
            created_at: now_iso8601(),
            sources: vec![],
            is_seed: false,
        }
    }

    /// Create origin for seed recipe
    pub fn seed() -> Self {
        Self {
            created_from_ticket: None,
            created_by: "seed".to_string(),
            created_at: now_iso8601(),
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
        self.last_used_at = Some(now_iso8601());
    }

    /// Record a failed use
    pub fn record_failure(&mut self) {
        self.uses += 1;
        self.failures += 1;
        self.last_used_at = Some(now_iso8601());
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
        self.uses >= super::MIN_RELIABLE_USES && self.success_rate() >= 0.7
    }
}

impl Default for LearnedRecipe {
    fn default() -> Self {
        Self {
            id: String::new(),
            domain: "general".to_string(),
            pattern: RecipePattern::default(),
            inputs: RecipeInputs::default(),
            probes: vec![],
            logic: RecipeLogic {
                logic_type: LogicType::Template,
                answer_kind: AnswerKind::Diagnostic,
                steps: vec![],
                conditionals: HashMap::new(),
            },
            answer_template: AnswerTemplate::default(),
            safety: RecipeSafety::default(),
            origin: RecipeOrigin::default(),
            stats: RecipeUsageStats::default(),
            version: 1,
            enabled: true,
        }
    }
}

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
        self.stats.uses < super::MIN_RELIABLE_USES || self.stats.success_rate() >= 0.6
    }

    /// Check if recipe is mature (enough usage data)
    pub fn is_mature(&self) -> bool {
        self.stats.uses >= super::MIN_RELIABLE_USES
    }
}

/// Get current timestamp as ISO 8601 string
fn now_iso8601() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_creation() {
        let recipe = LearnedRecipe::new("check-ram", "performance.memory")
            .with_pattern(RecipePattern::new("check_free_ram").with_keywords(&["ram", "memory", "free"]))
            .with_probe(RecipeProbe::new("free", "probe.free"))
            .with_answer(
                "Available RAM: {{available_mb}} MB",
                "Memory status:\n  Total: {{total_mb}} MB\n  Used: {{used_mb}} MB\n  Available: {{available_mb}} MB",
            );

        assert_eq!(recipe.id, "check-ram");
        assert_eq!(recipe.pattern.intent, "check_free_ram");
        assert_eq!(recipe.probes.len(), 1);
    }

    #[test]
    fn test_template_substitution() {
        let template = AnswerTemplate::new(
            "Service {{service_name}} is {{state}}",
            "Details: {{details}}",
        );

        let mut values = HashMap::new();
        values.insert("service_name".to_string(), "nginx".to_string());
        values.insert("state".to_string(), "running".to_string());

        let short = template.render_short(&values);
        assert_eq!(short, "Service nginx is running");
    }

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
