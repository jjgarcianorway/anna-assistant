//! Team-tagged recipes for learning from successful patterns.
//!
//! Recipes capture successful ticket resolutions for future reference.
//! Only persisted when: Ticket status = Verified, reliability score >= 80.
//!
//! Storage: ~/.anna/recipes/{recipe_id}.json
//!
//! v0.0.27: Extended with RecipeKind for config edits and change actions.

use crate::teams::Team;
use crate::ticket::RiskLevel;
use crate::trace::EvidenceKind;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

/// Kind of recipe action (v0.0.27)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RecipeKind {
    /// Read-only query (default, no system changes)
    Query,
    /// Append a line to config file if not present
    ConfigEditLineAppend,
    /// Ensure a specific line exists in config file
    ConfigEnsureLine,
    /// Install a package (future)
    #[serde(other)]
    Unknown,
}

impl Default for RecipeKind {
    fn default() -> Self {
        Self::Query
    }
}

/// Target for a recipe action (v0.0.27)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipeTarget {
    /// Application identifier (e.g., "vim", "nano", "bash")
    pub app_id: String,
    /// Config path template (e.g., "$HOME/.vimrc")
    pub config_path_template: String,
}

impl RecipeTarget {
    pub fn new(app_id: impl Into<String>, config_path: impl Into<String>) -> Self {
        Self {
            app_id: app_id.into(),
            config_path_template: config_path.into(),
        }
    }

    /// Expand config path template with environment variables
    pub fn expand_path(&self) -> PathBuf {
        let expanded = self.config_path_template
            .replace("$HOME", &std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))
            .replace("~", &std::env::var("HOME").unwrap_or_else(|_| ".".to_string()));
        PathBuf::from(expanded)
    }
}

/// Action specification for config edit recipes (v0.0.27)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum RecipeAction {
    /// Ensure a line exists in the config file
    EnsureLine { line: String },
    /// Append a line to the end of the config file
    AppendLine { line: String },
    /// No action (read-only query)
    None,
}

impl Default for RecipeAction {
    fn default() -> Self {
        Self::None
    }
}

/// Rollback information for reversible changes (v0.0.27)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollbackInfo {
    /// Path to the backup file
    pub backup_path: PathBuf,
    /// Description of how to rollback
    pub description: String,
    /// Whether rollback has been tested
    #[serde(default)]
    pub tested: bool,
}

impl RollbackInfo {
    pub fn new(backup_path: PathBuf, description: impl Into<String>) -> Self {
        Self {
            backup_path,
            description: description.into(),
            tested: false,
        }
    }
}

/// Signature that uniquely identifies a query pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecipeSignature {
    /// Domain (e.g., "system", "network", "storage")
    pub domain: String,
    /// Intent (e.g., "question", "investigate", "request")
    pub intent: String,
    /// Route class from classifier
    pub route_class: String,
    /// Normalized query pattern (lowercase, trimmed)
    pub query_pattern: String,
}

impl RecipeSignature {
    /// Create a new recipe signature
    pub fn new(
        domain: impl Into<String>,
        intent: impl Into<String>,
        route_class: impl Into<String>,
        query: &str,
    ) -> Self {
        Self {
            domain: domain.into(),
            intent: intent.into(),
            route_class: route_class.into(),
            query_pattern: normalize_query(query),
        }
    }

    /// Compute a deterministic hash of this signature
    pub fn hash_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

/// Normalize query for pattern matching
fn normalize_query(query: &str) -> String {
    query.to_lowercase().trim().to_string()
}

/// A learned recipe from a successful ticket resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    /// Unique recipe ID (hash of signature + team)
    pub id: String,
    /// Query signature this recipe applies to
    pub signature: RecipeSignature,
    /// Team that handled this successfully
    pub team: Team,
    /// Risk level of the original request
    pub risk_level: RiskLevel,
    /// Evidence kinds required for this pattern
    pub required_evidence_kinds: Vec<EvidenceKind>,
    /// Probe sequence that worked
    pub probe_sequence: Vec<String>,
    /// Answer template (with placeholders for evidence)
    #[serde(default)]
    pub answer_template: String,
    /// Unix timestamp when created
    pub created_at: u64,
    /// Number of successful uses
    #[serde(default)]
    pub success_count: u32,
    /// Reliability score when created
    pub reliability_score: u8,
    // v0.0.27 fields
    /// Kind of recipe (Query, ConfigEnsureLine, etc.)
    #[serde(default)]
    pub kind: RecipeKind,
    /// Target for config edit recipes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<RecipeTarget>,
    /// Action to perform
    #[serde(default)]
    pub action: RecipeAction,
    /// Rollback information for reversible changes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rollback: Option<RollbackInfo>,
}

impl Recipe {
    /// Create a new read-only query recipe from verified ticket
    pub fn new(
        signature: RecipeSignature,
        team: Team,
        risk_level: RiskLevel,
        required_evidence_kinds: Vec<EvidenceKind>,
        probe_sequence: Vec<String>,
        answer_template: String,
        reliability_score: u8,
    ) -> Self {
        let id = compute_recipe_id(&signature, team);
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            id,
            signature,
            team,
            risk_level,
            required_evidence_kinds,
            probe_sequence,
            answer_template,
            created_at,
            success_count: 1,
            reliability_score,
            kind: RecipeKind::Query,
            target: None,
            action: RecipeAction::None,
            rollback: None,
        }
    }

    /// Create a config edit recipe (v0.0.27)
    pub fn config_edit(
        signature: RecipeSignature,
        team: Team,
        target: RecipeTarget,
        action: RecipeAction,
        reliability_score: u8,
    ) -> Self {
        let id = compute_recipe_id(&signature, team);
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let kind = match &action {
            RecipeAction::EnsureLine { .. } => RecipeKind::ConfigEnsureLine,
            RecipeAction::AppendLine { .. } => RecipeKind::ConfigEditLineAppend,
            RecipeAction::None => RecipeKind::Query,
        };

        Self {
            id,
            signature,
            team,
            risk_level: RiskLevel::LowRiskChange,
            required_evidence_kinds: vec![],
            probe_sequence: vec![],
            answer_template: String::new(),
            created_at,
            success_count: 1,
            reliability_score,
            kind,
            target: Some(target),
            action,
            rollback: None,
        }
    }

    /// Set rollback info
    pub fn with_rollback(mut self, rollback: RollbackInfo) -> Self {
        self.rollback = Some(rollback);
        self
    }

    /// Increment success count
    pub fn record_success(&mut self) {
        self.success_count = self.success_count.saturating_add(1);
    }

    /// Check if recipe is mature (used successfully multiple times)
    pub fn is_mature(&self) -> bool {
        self.success_count >= 3
    }

    /// Check if this is a config edit recipe
    pub fn is_config_edit(&self) -> bool {
        matches!(
            self.kind,
            RecipeKind::ConfigEnsureLine | RecipeKind::ConfigEditLineAppend
        )
    }

    /// Get filesystem path for this recipe
    pub fn file_path(&self) -> PathBuf {
        recipe_dir().join(format!("{}.json", self.id))
    }

    /// Save recipe to disk
    pub fn save(&self) -> std::io::Result<()> {
        let dir = recipe_dir();
        std::fs::create_dir_all(&dir)?;
        let path = self.file_path();
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, json)
    }

    /// Load recipe from disk
    pub fn load(recipe_id: &str) -> std::io::Result<Self> {
        let path = recipe_filename(recipe_id);
        let json = std::fs::read_to_string(path)?;
        serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

/// Compute deterministic recipe ID from signature and team
pub fn compute_recipe_id(signature: &RecipeSignature, team: Team) -> String {
    let mut hasher = DefaultHasher::new();
    signature.hash(&mut hasher);
    team.to_string().hash(&mut hasher);
    let hash = hasher.finish();
    format!("{:016x}", hash)
}

/// Get the recipes directory path
pub fn recipe_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".anna")
        .join("recipes")
}

/// Get path for a specific recipe file
pub fn recipe_filename(recipe_id: &str) -> PathBuf {
    recipe_dir().join(format!("{}.json", recipe_id))
}

/// Check if a recipe should be persisted
/// Only persist when: Verified status AND reliability >= 80
pub fn should_persist_recipe(verified: bool, score: u8) -> bool {
    verified && score >= 80
}

/// Clear all recipes (for reset) (v0.0.28)
pub fn clear_all_recipes() -> std::io::Result<()> {
    let dir = recipe_dir();
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

/// Count recipes in store
pub fn recipe_count() -> usize {
    let dir = recipe_dir();
    if !dir.exists() {
        return 0;
    }
    std::fs::read_dir(&dir)
        .map(|entries| entries.filter_map(|e| e.ok()).count())
        .unwrap_or(0)
}

// Tests moved to tests/recipe_tests.rs to keep this file under 400 lines
