//! Recipe store types (v0.0.232).

use serde::{Deserialize, Serialize};

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
