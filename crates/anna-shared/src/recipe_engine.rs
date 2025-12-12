//! Recipe Engine - Self-learning recipe system (v0.0.412).
//!
//! Core types for Anna's learning system:
//! - Recipe: Learned, replayable solution pattern
//! - RecipeStep: Individual action in a recipe
//! - EvidenceRequirement: What data a recipe needs
//! - RecipeStore: Persistent storage with matching
//!
//! Design goals:
//! - Minimize hardcoding, maximize learning
//! - All recipes are deterministic and auditable
//! - Generic recipes with parameters, not one per question

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Recipe kind - what type of solution this provides
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipeKind {
    /// Read-only probe execution (no changes)
    ProbeOnly,
    /// Configuration change
    Configure,
    /// System inspection/diagnosis
    Inspect,
    /// Problem diagnosis with suggested fixes
    Diagnose,
    /// Generate a report/summary
    Report,
}

impl Default for RecipeKind {
    fn default() -> Self {
        Self::ProbeOnly
    }
}

impl std::fmt::Display for RecipeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProbeOnly => write!(f, "probe_only"),
            Self::Configure => write!(f, "configure"),
            Self::Inspect => write!(f, "inspect"),
            Self::Diagnose => write!(f, "diagnose"),
            Self::Report => write!(f, "report"),
        }
    }
}

/// Evidence requirement - what data a recipe needs to run
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRequirement {
    None,
    Meminfo,
    Swaps,
    DfRoot,
    SystemdFailed,
    PacmanList,
    JournalErrors,
    NetworkInterfaces,
    GpuInfo,
    AudioDevices,
    /// Extensible custom requirement
    Custom(String),
}

impl std::fmt::Display for EvidenceRequirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Meminfo => write!(f, "meminfo"),
            Self::Swaps => write!(f, "swaps"),
            Self::DfRoot => write!(f, "df_root"),
            Self::SystemdFailed => write!(f, "systemd_failed"),
            Self::PacmanList => write!(f, "pacman_list"),
            Self::JournalErrors => write!(f, "journal_errors"),
            Self::NetworkInterfaces => write!(f, "network_interfaces"),
            Self::GpuInfo => write!(f, "gpu_info"),
            Self::AudioDevices => write!(f, "audio_devices"),
            Self::Custom(s) => write!(f, "custom:{}", s),
        }
    }
}

/// Recipe step type - what kind of action this step performs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipeStepType {
    /// Run an existing probe by ID
    RunProbe,
    /// Run an explicit shell command
    RunCommand,
    /// Check a condition on previous outputs
    CheckCondition,
    /// Edit a file (templated)
    EditFile,
    /// Render the final answer
    RenderAnswer,
    /// Call another recipe (composition)
    Subrecipe,
}

impl std::fmt::Display for RecipeStepType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RunProbe => write!(f, "run_probe"),
            Self::RunCommand => write!(f, "run_command"),
            Self::CheckCondition => write!(f, "check_condition"),
            Self::EditFile => write!(f, "edit_file"),
            Self::RenderAnswer => write!(f, "render_answer"),
            Self::Subrecipe => write!(f, "subrecipe"),
        }
    }
}

/// A single step in a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeStep {
    /// Step identifier (unique within recipe)
    pub id: String,
    /// What kind of step this is
    pub kind: RecipeStepType,
    /// Human-readable description
    pub description: String,
    /// Step-specific parameters (serialized)
    pub params: HashMap<String, String>,
    /// Step IDs that must complete first
    pub depends_on: Vec<String>,
}

impl RecipeStep {
    /// Create a new run_probe step
    pub fn probe(id: &str, probe_id: &str, description: &str) -> Self {
        let mut params = HashMap::new();
        params.insert("probe_id".to_string(), probe_id.to_string());
        Self {
            id: id.to_string(),
            kind: RecipeStepType::RunProbe,
            description: description.to_string(),
            params,
            depends_on: vec![],
        }
    }

    /// Create a new run_command step
    pub fn command(id: &str, cmd: &str, description: &str) -> Self {
        let mut params = HashMap::new();
        params.insert("command".to_string(), cmd.to_string());
        Self {
            id: id.to_string(),
            kind: RecipeStepType::RunCommand,
            description: description.to_string(),
            params,
            depends_on: vec![],
        }
    }

    /// Create a new render_answer step
    pub fn render(id: &str, template: &str, description: &str) -> Self {
        let mut params = HashMap::new();
        params.insert("template".to_string(), template.to_string());
        Self {
            id: id.to_string(),
            kind: RecipeStepType::RenderAnswer,
            description: description.to_string(),
            params,
            depends_on: vec![],
        }
    }

    /// Add dependency
    pub fn depends(mut self, step_id: &str) -> Self {
        self.depends_on.push(step_id.to_string());
        self
    }

    /// Check if step is read-only
    pub fn is_read_only(&self) -> bool {
        matches!(
            self.kind,
            RecipeStepType::RunProbe
                | RecipeStepType::CheckCondition
                | RecipeStepType::RenderAnswer
        )
    }

    /// Check if step requires user confirmation
    pub fn requires_confirmation(&self) -> bool {
        matches!(
            self.kind,
            RecipeStepType::EditFile | RecipeStepType::RunCommand
        ) && !self.is_safe_command()
    }

    /// Check if command is in the safe list
    fn is_safe_command(&self) -> bool {
        if self.kind != RecipeStepType::RunCommand {
            return true;
        }
        let cmd = self.params.get("command").map(|s| s.as_str()).unwrap_or("");
        // Safe read-only commands
        let safe_prefixes = [
            "cat ",
            "head ",
            "tail ",
            "ls ",
            "df ",
            "free ",
            "ps ",
            "top -bn1",
            "systemctl status",
            "systemctl is-",
            "journalctl ",
            "lsblk",
            "lscpu",
            "ip addr",
            "ip link",
            "ip route",
            "ss -",
            "pacman -Q",
            "which ",
            "echo ",
            "printf ",
            "test ",
            "stat ",
            "file ",
            "wc ",
            "grep ",
        ];
        safe_prefixes.iter().any(|p| cmd.starts_with(p))
    }
}

/// Risk level for recipe execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// No system changes
    ReadOnly,
    /// Minor changes, easily reversible
    Low,
    /// Significant changes
    Medium,
    /// Potentially destructive
    High,
}

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

/// A recipe parameter (variable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeParameter {
    /// Parameter name (e.g., "service_name")
    pub name: String,
    /// Description
    pub description: String,
    /// How to extract from query
    pub extraction_hint: String,
    /// Optional default value
    pub default: Option<String>,
    /// Whether required
    pub required: bool,
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
