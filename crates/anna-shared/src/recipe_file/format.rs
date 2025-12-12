//! Recipe file format definitions (v0.0.406).
//!
//! Defines the TOML structure for authored recipes.
//!
//! Example recipe:
//! ```toml
//! [id]
//! name = "check_failed_services"
//! domain = "services"
//! version = "1"
//!
//! [match]
//! intent = "diagnose"
//! keywords = ["failed", "services"]
//!
//! [plan]
//! steps = [
//!   { id = "list_failed", probe = "failed_services" }
//! ]
//!
//! [answer]
//! template = """
//! There are {failed_count} failed services:
//! {failed_list}
//! """
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A file-based recipe (TOML format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRecipe {
    /// Recipe identification
    pub id: RecipeId,
    /// Matching criteria
    #[serde(rename = "match")]
    pub match_criteria: RecipeMatch,
    /// Execution plan
    pub plan: RecipePlan,
    /// Answer rendering
    pub answer: RecipeAnswer,
    /// Optional metadata
    #[serde(default)]
    pub meta: RecipeMeta,
}

/// Recipe identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeId {
    /// Unique recipe name (e.g., "check_failed_services")
    pub name: String,
    /// Domain this recipe handles (e.g., "services", "storage")
    pub domain: String,
    /// Recipe version for tracking changes
    #[serde(default = "default_version")]
    pub version: String,
}

fn default_version() -> String {
    "1".to_string()
}

/// Matching criteria to determine if recipe applies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeMatch {
    /// Intent to match (e.g., "diagnose", "configure", "query_metric")
    pub intent: String,
    /// Keywords that should be present in query (any match)
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Required keywords (all must match)
    #[serde(default)]
    pub required_keywords: Vec<String>,
    /// Specific parameter key to match
    #[serde(default)]
    pub key: Option<String>,
    /// Target parameter (e.g., "editor", "service")
    #[serde(default)]
    pub target: Option<String>,
    /// Additional parameter constraints
    #[serde(default)]
    pub params: HashMap<String, String>,
    /// Minimum confidence threshold (0-100)
    #[serde(default = "default_confidence")]
    pub min_confidence: u8,
}

fn default_confidence() -> u8 {
    60
}

/// Execution plan with steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipePlan {
    /// Ordered list of steps to execute
    pub steps: Vec<RecipeStep>,
    /// Whether to stop on first error
    #[serde(default = "default_true")]
    pub stop_on_error: bool,
    /// Backup paths before modifying (for safety)
    #[serde(default)]
    pub backup_paths: Vec<String>,
}

fn default_true() -> bool {
    true
}

/// A single step in the recipe plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeStep {
    /// Step identifier
    pub id: String,
    /// Probe ID to run (from probe_registry)
    #[serde(default)]
    pub probe: Option<String>,
    /// Direct command to run
    #[serde(default)]
    pub cmd: Option<String>,
    /// Whether this step requires user confirmation
    #[serde(default)]
    pub needs_confirm: ConfirmLevel,
    /// Description for user (shown during confirm)
    #[serde(default)]
    pub description: Option<String>,
    /// Variables to extract from output (regex patterns)
    #[serde(default)]
    pub extract: HashMap<String, String>,
    /// Condition to run this step (e.g., "prev_exit_code == 0")
    #[serde(default)]
    pub condition: Option<String>,
    /// Timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_timeout() -> u64 {
    30
}

/// Confirmation level for a step
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmLevel {
    /// No confirmation needed (read-only or safe)
    #[default]
    None,
    /// Confirm once at start
    Once,
    /// Confirm before each command
    Each,
    /// Always confirm with full command display
    Always,
}

/// Answer template for rendering results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeAnswer {
    /// Template with placeholders like {variable_name}
    pub template: String,
    /// Default values for missing variables
    #[serde(default)]
    pub defaults: HashMap<String, String>,
    /// Whether to append raw probe output
    #[serde(default)]
    pub include_raw_output: bool,
}

/// Optional metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecipeMeta {
    /// Recipe author
    #[serde(default)]
    pub author: Option<String>,
    /// Creation/update timestamp
    #[serde(default)]
    pub updated: Option<String>,
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
    /// Example queries this handles
    #[serde(default)]
    pub examples: Vec<String>,
    /// Related recipe names
    #[serde(default)]
    pub related: Vec<String>,
}

impl FileRecipe {
    /// Get the full recipe ID (domain/name)
    pub fn full_id(&self) -> String {
        format!("{}/{}", self.id.domain, self.id.name)
    }

    /// Check if recipe requires any confirmations
    pub fn requires_confirmation(&self) -> bool {
        self.plan
            .steps
            .iter()
            .any(|s| s.needs_confirm != ConfirmLevel::None)
    }

    /// Check if recipe is read-only (no commands, only probes)
    pub fn is_read_only(&self) -> bool {
        self.plan.steps.iter().all(|s| s.cmd.is_none())
    }

    /// Get all probe IDs used by this recipe
    pub fn probe_ids(&self) -> Vec<&str> {
        self.plan
            .steps
            .iter()
            .filter_map(|s| s.probe.as_deref())
            .collect()
    }
}

impl RecipeStep {
    /// Check if this step runs a probe
    pub fn is_probe(&self) -> bool {
        self.probe.is_some()
    }

    /// Check if this step runs a command
    pub fn is_command(&self) -> bool {
        self.cmd.is_some()
    }

    /// Get the command to run (probe lookup or direct command)
    pub fn get_command(&self, probe_lookup: impl Fn(&str) -> Option<String>) -> Option<String> {
        if let Some(ref probe_id) = self.probe {
            probe_lookup(probe_id)
        } else {
            self.cmd.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_recipe() {
        let toml = r#"
[id]
name = "check_disk"
domain = "storage"

[match]
intent = "diagnose"

[plan]
steps = [
  { id = "df", probe = "disk_usage" }
]

[answer]
template = "Disk usage: {disk_info}"
"#;
        let recipe: FileRecipe = toml::from_str(toml).unwrap();
        assert_eq!(recipe.id.name, "check_disk");
        assert_eq!(recipe.id.domain, "storage");
        assert_eq!(recipe.plan.steps.len(), 1);
        assert!(recipe.is_read_only());
    }

    #[test]
    fn test_parse_config_recipe() {
        let toml = r#"
[id]
name = "enable_vim_syntax"
domain = "desktop"
version = "1"

[match]
intent = "configure"
target = "editor"
params = { editor = "vim", feature = "syntax" }

[plan]
backup_paths = ["~/.vimrc"]
steps = [
  { id = "ensure", cmd = "touch ~/.vimrc", needs_confirm = "none" },
  { id = "append", cmd = "echo 'syntax enable' >> ~/.vimrc", needs_confirm = "once", description = "Add syntax highlighting" }
]

[answer]
template = "Enabled syntax highlighting in vim."
"#;
        let recipe: FileRecipe = toml::from_str(toml).unwrap();
        assert_eq!(recipe.id.name, "enable_vim_syntax");
        assert!(recipe.requires_confirmation());
        assert!(!recipe.is_read_only());
        assert_eq!(recipe.plan.backup_paths, vec!["~/.vimrc"]);
    }

    #[test]
    fn test_full_id() {
        let recipe = FileRecipe {
            id: RecipeId {
                name: "test".to_string(),
                domain: "system".to_string(),
                version: "1".to_string(),
            },
            match_criteria: RecipeMatch {
                intent: "diagnose".to_string(),
                keywords: vec![],
                required_keywords: vec![],
                key: None,
                target: None,
                params: HashMap::new(),
                min_confidence: 60,
            },
            plan: RecipePlan {
                steps: vec![],
                stop_on_error: true,
                backup_paths: vec![],
            },
            answer: RecipeAnswer {
                template: "Test".to_string(),
                defaults: HashMap::new(),
                include_raw_output: false,
            },
            meta: RecipeMeta::default(),
        };
        assert_eq!(recipe.full_id(), "system/test");
    }
}
