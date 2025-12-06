//! Recipe store for learned solutions (v0.0.75).
//!
//! Stores reusable recipes learned from high-reliability ticket resolutions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::Path;

/// Risk level for recipe actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipeRisk {
    /// Read-only, safe to execute
    ReadOnly,
    /// Modifies user config (reversible)
    ConfigChange,
    /// System-level change (needs confirmation)
    SystemChange,
    /// Potentially destructive (requires explicit confirmation)
    Destructive,
}

impl RecipeRisk {
    pub fn requires_confirmation(&self) -> bool {
        matches!(self, Self::SystemChange | Self::Destructive)
    }

    pub fn display(&self) -> &'static str {
        match self {
            Self::ReadOnly => "Read-only",
            Self::ConfigChange => "Config change",
            Self::SystemChange => "System change",
            Self::Destructive => "Potentially destructive",
        }
    }
}

/// A single step in a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeStep {
    /// Step description
    pub description: String,
    /// Template for the action (with placeholders like {editor}, {file})
    pub action_template: String,
    /// Required evidence to execute this step
    pub required_evidence: Vec<String>,
    /// Whether this step mutates the system
    pub mutates: bool,
    /// Rollback instructions (if mutates)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rollback: Option<String>,
}

/// Citation for recipe documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    /// Source type: "man", "help", "wiki", "internal"
    pub source_type: String,
    /// Source reference (e.g., "man vim", "vim --help")
    pub source_ref: String,
    /// Relevant excerpt (truncated)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
}

/// A learned recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    /// Unique recipe ID
    pub id: String,
    /// Category (e.g., "editor_config", "system_info", "troubleshooting")
    pub category: String,
    /// Human-readable title
    pub title: String,
    /// Query classes that trigger this recipe
    pub triggers: Vec<String>,
    /// Required evidence kinds for this recipe
    pub required_evidence: Vec<String>,
    /// Risk level
    pub risk: RecipeRisk,
    /// Recipe steps
    pub steps: Vec<RecipeStep>,
    /// Citations for teaching mode
    #[serde(default)]
    pub citations: Vec<Citation>,
    /// Metadata: ticket ID this was learned from
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub learned_from_ticket: Option<String>,
    /// Metadata: reliability score at learn time
    #[serde(default)]
    pub learned_reliability: u8,
    /// Metadata: creation timestamp
    pub created_at: u64,
    /// Usage count
    #[serde(default)]
    pub usage_count: u64,
    /// Last used timestamp
    #[serde(default)]
    pub last_used: u64,
}

impl Recipe {
    /// Create a new recipe
    pub fn new(id: &str, category: &str, title: &str) -> Self {
        Self {
            id: id.to_string(),
            category: category.to_string(),
            title: title.to_string(),
            triggers: Vec::new(),
            required_evidence: Vec::new(),
            risk: RecipeRisk::ReadOnly,
            steps: Vec::new(),
            citations: Vec::new(),
            learned_from_ticket: None,
            learned_reliability: 0,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            usage_count: 0,
            last_used: 0,
        }
    }

    /// Add a trigger query class
    pub fn with_trigger(mut self, query_class: &str) -> Self {
        self.triggers.push(query_class.to_string());
        self
    }

    /// Add required evidence
    pub fn with_evidence(mut self, evidence: &str) -> Self {
        self.required_evidence.push(evidence.to_string());
        self
    }

    /// Set risk level
    pub fn with_risk(mut self, risk: RecipeRisk) -> Self {
        self.risk = risk;
        self
    }

    /// Add a step
    pub fn with_step(mut self, step: RecipeStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Add citation
    pub fn with_citation(mut self, citation: Citation) -> Self {
        self.citations.push(citation);
        self
    }

    /// Mark as learned from ticket
    pub fn learned_from(mut self, ticket_id: &str, reliability: u8) -> Self {
        self.learned_from_ticket = Some(ticket_id.to_string());
        self.learned_reliability = reliability;
        self
    }

    /// Check if recipe matches query and evidence
    pub fn matches(&self, query_class: &str, available_evidence: &[String]) -> bool {
        // Check trigger match
        if !self.triggers.iter().any(|t| t == query_class) {
            return false;
        }

        // Check evidence requirements
        self.required_evidence
            .iter()
            .all(|req| available_evidence.iter().any(|ev| ev == req))
    }

    /// Record usage
    pub fn record_usage(&mut self) {
        self.usage_count += 1;
        self.last_used = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
    }
}

/// Recipe store with persistence
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecipeStore {
    /// Version for migration
    pub version: u32,
    /// Recipes by ID
    pub recipes: HashMap<String, Recipe>,
    /// Index: query_class -> recipe IDs
    #[serde(default)]
    pub trigger_index: HashMap<String, Vec<String>>,
}

impl RecipeStore {
    pub fn new() -> Self {
        Self {
            version: 1,
            recipes: HashMap::new(),
            trigger_index: HashMap::new(),
        }
    }

    /// Default path
    pub fn default_path() -> std::path::PathBuf {
        std::path::PathBuf::from("/var/lib/anna/recipes_v2.json")
    }

    /// Load from file
    pub fn load(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::new());
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let store: Self = serde_json::from_reader(reader)?;
        Ok(store)
    }

    /// Save to file
    pub fn save(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let path = path.as_ref();

        // Ensure parent exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Atomic write via temp file
        let temp_path = path.with_extension("json.tmp");
        {
            let file = File::create(&temp_path)?;
            let writer = BufWriter::new(file);
            serde_json::to_writer_pretty(writer, self)?;
        }
        fs::rename(&temp_path, path)?;

        Ok(())
    }

    /// Add a recipe
    pub fn add(&mut self, recipe: Recipe) {
        // Update trigger index
        for trigger in &recipe.triggers {
            self.trigger_index
                .entry(trigger.clone())
                .or_default()
                .push(recipe.id.clone());
        }

        self.recipes.insert(recipe.id.clone(), recipe);
    }

    /// Find matching recipes for a query
    pub fn find_matches(&self, query_class: &str, evidence: &[String]) -> Vec<&Recipe> {
        let recipe_ids = self.trigger_index.get(query_class);

        match recipe_ids {
            Some(ids) => ids
                .iter()
                .filter_map(|id| self.recipes.get(id))
                .filter(|r| r.matches(query_class, evidence))
                .collect(),
            None => Vec::new(),
        }
    }

    /// Get recipe by ID
    pub fn get(&self, id: &str) -> Option<&Recipe> {
        self.recipes.get(id)
    }

    /// Get mutable recipe by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Recipe> {
        self.recipes.get_mut(id)
    }

    /// Count recipes by category
    pub fn count_by_category(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for recipe in self.recipes.values() {
            *counts.entry(recipe.category.clone()).or_insert(0) += 1;
        }
        counts
    }

    /// Total recipe count
    pub fn len(&self) -> usize {
        self.recipes.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.recipes.is_empty()
    }
}

/// Minimum reliability to learn a recipe
pub const MIN_LEARN_RELIABILITY: u8 = 85;

/// Should we learn a recipe from this outcome?
pub fn should_learn_recipe(reliability: u8, is_deterministic: bool, already_exists: bool) -> bool {
    if already_exists {
        return false;
    }
    if reliability < MIN_LEARN_RELIABILITY {
        return false;
    }
    // Only learn from deterministic paths (grounded answers)
    is_deterministic
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_builder() {
        let recipe = Recipe::new("vim-syntax", "editor_config", "Enable Vim Syntax Highlighting")
            .with_trigger("configure_editor")
            .with_evidence("tool_exists")
            .with_risk(RecipeRisk::ConfigChange);

        assert_eq!(recipe.triggers, vec!["configure_editor"]);
        assert_eq!(recipe.risk, RecipeRisk::ConfigChange);
    }

    #[test]
    fn test_recipe_matches() {
        let recipe = Recipe::new("test", "test", "Test")
            .with_trigger("memory_usage")
            .with_evidence("memory");

        assert!(recipe.matches("memory_usage", &["memory".to_string()]));
        assert!(!recipe.matches("disk_usage", &["memory".to_string()]));
        assert!(!recipe.matches("memory_usage", &["disk".to_string()]));
    }

    #[test]
    fn test_recipe_store_operations() {
        let mut store = RecipeStore::new();

        let recipe = Recipe::new("test-1", "test", "Test Recipe")
            .with_trigger("test_query");

        store.add(recipe);

        assert_eq!(store.len(), 1);
        assert!(store.get("test-1").is_some());
        assert_eq!(store.find_matches("test_query", &[]).len(), 1);
        assert_eq!(store.find_matches("other_query", &[]).len(), 0);
    }

    #[test]
    fn test_should_learn_recipe() {
        assert!(should_learn_recipe(90, true, false));
        assert!(!should_learn_recipe(80, true, false)); // Too low
        assert!(!should_learn_recipe(90, false, false)); // Not deterministic
        assert!(!should_learn_recipe(90, true, true)); // Already exists
    }

    #[test]
    fn test_risk_requires_confirmation() {
        assert!(!RecipeRisk::ReadOnly.requires_confirmation());
        assert!(!RecipeRisk::ConfigChange.requires_confirmation());
        assert!(RecipeRisk::SystemChange.requires_confirmation());
        assert!(RecipeRisk::Destructive.requires_confirmation());
    }
}
