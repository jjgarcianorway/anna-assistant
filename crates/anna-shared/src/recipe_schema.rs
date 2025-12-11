//! Recipe data model for Anna's learning system.
//! v0.0.418: Full recipe schema with matcher, plan steps, and metrics.
//!
//! Recipes are declarative JSON objects that describe:
//! - The INTENT they serve
//! - How to detect when they apply (matcher)
//! - Preconditions (probes or simple checks)
//! - A PLAN: steps the non-LLM engine can execute
//! - Success criteria and rollback behavior
//! - Origin/citations and metrics

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A learned recipe that can be executed without LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    /// Unique identifier (e.g., "vim_enable_syntax")
    pub id: String,
    /// Recipe version (incremented on updates)
    pub version: u32,
    /// Domain: desktop, storage, network, etc.
    pub domain: String,
    /// Canonical intent this recipe serves
    pub intent: String,
    /// Pattern describing what this recipe handles
    pub pattern: RecipePattern,
    /// Matcher configuration for runtime lookup
    pub matcher: RecipeMatcher,
    /// Preconditions that must be true before execution
    pub preconditions: Vec<Precondition>,
    /// Plan steps to execute
    pub plan: Vec<PlanStep>,
    /// Whether to require user confirmation
    pub confirmation_policy: ConfirmationPolicy,
    /// Success criteria and rollback behavior
    pub success_criteria: SuccessCriteria,
    /// Documentation citations (Arch Wiki, man pages)
    pub citations: Vec<String>,
    /// Usage metrics
    pub metrics: RecipeMetrics,
    /// Recipe status
    pub status: RecipeStatus,
    /// When this recipe was created
    pub created_at: DateTime<Utc>,
    /// When this recipe was last updated
    pub updated_at: DateTime<Utc>,
}

/// Pattern describing what user goal this recipe handles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipePattern {
    /// Human-readable description of user goal
    pub user_goal: String,
    /// Extracted slots/parameters (e.g., editor="vim", feature="syntax")
    #[serde(default)]
    pub slots: HashMap<String, String>,
}

/// Matcher configuration for finding applicable recipes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeMatcher {
    /// Keywords that MUST be present
    pub required_keywords: Vec<String>,
    /// Keywords that boost score if present
    #[serde(default)]
    pub optional_keywords: Vec<String>,
    /// Keywords that disqualify this recipe
    #[serde(default)]
    pub negative_keywords: Vec<String>,
    /// Minimum confidence to use this recipe (0.0-1.0)
    pub min_confidence: f32,
    /// Intent must match exactly (if set)
    #[serde(default)]
    pub exact_intent: Option<String>,
}

/// Precondition that must be true before recipe execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Precondition {
    /// Check if a tool/command exists
    ToolExists { tool: String },
    /// Check if a file exists
    FileExists { path: String },
    /// Check if a directory exists
    DirExists { path: String },
    /// Check probe result contains expected value
    ProbeContains { probe: String, contains: String },
    /// Check probe result matches regex
    ProbeMatches { probe: String, pattern: String },
    /// Check systemd service exists
    ServiceExists { service: String },
    /// Custom probe check
    ProbeCheck { probe: String, condition: String },
}

/// A step in the recipe plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PlanStep {
    /// Explain something to the user (no action)
    Explain { message: String },
    /// Backup a file before modification
    BackupFile { path: String },
    /// Append a line to a file
    AppendLine { path: String, line: String },
    /// Prepend a line to a file
    PrependLine { path: String, line: String },
    /// Replace a line matching pattern
    ReplaceLine {
        path: String,
        pattern: String,
        replacement: String,
    },
    /// Ensure a line exists (add if missing)
    EnsureLine { path: String, line: String },
    /// Remove lines matching pattern
    RemoveLines { path: String, pattern: String },
    /// Run a command (read-only, for verification)
    VerifyCommand { command: String, expect_success: bool },
    /// Run a command that changes system state
    RunCommand {
        command: String,
        description: String,
        rollback_command: Option<String>,
    },
    /// Enable a systemd service
    EnableService { service: String, start: bool },
    /// Disable a systemd service
    DisableService { service: String, stop: bool },
    /// Restart a systemd service
    RestartService { service: String },
    /// Create a directory
    CreateDir { path: String, mode: Option<String> },
    /// Create or overwrite a file
    WriteFile {
        path: String,
        content: String,
        mode: Option<String>,
    },
    /// Set environment variable (in shell config)
    SetEnvVar {
        name: String,
        value: String,
        shell_config: String,
    },
}

impl PlanStep {
    /// Get the step type name
    pub fn type_name(&self) -> &'static str {
        match self {
            PlanStep::Explain { .. } => "explain",
            PlanStep::BackupFile { .. } => "backup_file",
            PlanStep::AppendLine { .. } => "append_line",
            PlanStep::PrependLine { .. } => "prepend_line",
            PlanStep::ReplaceLine { .. } => "replace_line",
            PlanStep::EnsureLine { .. } => "ensure_line",
            PlanStep::RemoveLines { .. } => "remove_lines",
            PlanStep::VerifyCommand { .. } => "verify_command",
            PlanStep::RunCommand { .. } => "run_command",
            PlanStep::EnableService { .. } => "enable_service",
            PlanStep::DisableService { .. } => "disable_service",
            PlanStep::RestartService { .. } => "restart_service",
            PlanStep::CreateDir { .. } => "create_dir",
            PlanStep::WriteFile { .. } => "write_file",
            PlanStep::SetEnvVar { .. } => "set_env_var",
        }
    }

    /// Check if this step modifies the system
    pub fn is_mutating(&self) -> bool {
        !matches!(self, PlanStep::Explain { .. } | PlanStep::VerifyCommand { .. })
    }
}

/// Confirmation policy for recipe execution.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationPolicy {
    /// Always require user confirmation
    Require,
    /// Ask for confirmation for mutating steps only
    MutatingOnly,
    /// Never ask (dangerous, only for safe read-only recipes)
    Never,
}

impl Default for ConfirmationPolicy {
    fn default() -> Self {
        Self::Require
    }
}

/// Success criteria for recipe execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriteria {
    /// Step types that must succeed
    #[serde(default)]
    pub must_succeed: Vec<String>,
    /// Whether to rollback on any failure
    #[serde(default = "default_true")]
    pub rollback_on_failure: bool,
    /// Optional verification command to run after plan
    #[serde(default)]
    pub post_verification: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Default for SuccessCriteria {
    fn default() -> Self {
        Self {
            must_succeed: vec![],
            rollback_on_failure: true,
            post_verification: None,
        }
    }
}

/// Usage metrics for a recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeMetrics {
    /// Times this recipe was used
    pub times_used: u32,
    /// Times this recipe failed
    pub times_failed: u32,
    /// Last time this recipe was used
    #[serde(default)]
    pub last_used_at: Option<DateTime<Utc>>,
    /// Average user rating (if collected)
    #[serde(default)]
    pub avg_user_rating: Option<f32>,
    /// Recent success rate (last N uses)
    #[serde(default)]
    pub recent_success_rate: Option<f32>,
}

impl Default for RecipeMetrics {
    fn default() -> Self {
        Self {
            times_used: 0,
            times_failed: 0,
            last_used_at: None,
            avg_user_rating: None,
            recent_success_rate: None,
        }
    }
}

/// Recipe status.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecipeStatus {
    /// Recipe is active and can be used
    Active,
    /// Recipe needs review (e.g., deprecated commands)
    NeedsReview,
    /// Recipe is disabled due to failures
    Disabled,
    /// Recipe is deprecated (superseded by another)
    Deprecated,
}

impl Default for RecipeStatus {
    fn default() -> Self {
        Self::Active
    }
}

impl Recipe {
    /// Create a new recipe with default metrics
    pub fn new(
        id: String,
        domain: String,
        intent: String,
        pattern: RecipePattern,
        matcher: RecipeMatcher,
        plan: Vec<PlanStep>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            version: 1,
            domain,
            intent,
            pattern,
            matcher,
            preconditions: vec![],
            plan,
            confirmation_policy: ConfirmationPolicy::Require,
            success_criteria: SuccessCriteria::default(),
            citations: vec![],
            metrics: RecipeMetrics::default(),
            status: RecipeStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if recipe is usable
    pub fn is_usable(&self) -> bool {
        self.status == RecipeStatus::Active
    }

    /// Record a successful use
    pub fn record_success(&mut self) {
        self.metrics.times_used += 1;
        self.metrics.last_used_at = Some(Utc::now());
        self.update_success_rate();
    }

    /// Record a failed use
    pub fn record_failure(&mut self) {
        self.metrics.times_used += 1;
        self.metrics.times_failed += 1;
        self.metrics.last_used_at = Some(Utc::now());
        self.update_success_rate();
        // Auto-disable if failure rate too high
        if self.should_auto_disable() {
            self.status = RecipeStatus::Disabled;
        }
    }

    fn update_success_rate(&mut self) {
        if self.metrics.times_used > 0 {
            let successes = self.metrics.times_used - self.metrics.times_failed;
            self.metrics.recent_success_rate =
                Some(successes as f32 / self.metrics.times_used as f32);
        }
    }

    fn should_auto_disable(&self) -> bool {
        // Disable if 3+ failures in last 10 uses
        self.metrics.times_used >= 10
            && self.metrics.times_failed >= 3
            && self.metrics.recent_success_rate.unwrap_or(1.0) < 0.7
    }

    /// Check if recipe has mutating steps
    pub fn has_mutating_steps(&self) -> bool {
        self.plan.iter().any(|s| s.is_mutating())
    }

    /// Get files this recipe touches
    pub fn touched_files(&self) -> Vec<String> {
        self.plan
            .iter()
            .filter_map(|step| match step {
                PlanStep::BackupFile { path }
                | PlanStep::AppendLine { path, .. }
                | PlanStep::PrependLine { path, .. }
                | PlanStep::ReplaceLine { path, .. }
                | PlanStep::EnsureLine { path, .. }
                | PlanStep::RemoveLines { path, .. }
                | PlanStep::WriteFile { path, .. } => Some(path.clone()),
                _ => None,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_creation() {
        let recipe = Recipe::new(
            "test_recipe".into(),
            "desktop".into(),
            "configure_editor".into(),
            RecipePattern {
                user_goal: "enable syntax highlighting".into(),
                slots: HashMap::new(),
            },
            RecipeMatcher {
                required_keywords: vec!["vim".into(), "syntax".into()],
                optional_keywords: vec![],
                negative_keywords: vec![],
                min_confidence: 0.8,
                exact_intent: None,
            },
            vec![PlanStep::Explain {
                message: "Test".into(),
            }],
        );
        assert!(recipe.is_usable());
        assert_eq!(recipe.version, 1);
    }

    #[test]
    fn test_auto_disable() {
        let mut recipe = Recipe::new(
            "test".into(),
            "test".into(),
            "test".into(),
            RecipePattern {
                user_goal: "test".into(),
                slots: HashMap::new(),
            },
            RecipeMatcher {
                required_keywords: vec![],
                optional_keywords: vec![],
                negative_keywords: vec![],
                min_confidence: 0.8,
                exact_intent: None,
            },
            vec![],
        );
        // Simulate 10 uses with 4 failures
        for _ in 0..6 {
            recipe.record_success();
        }
        for _ in 0..4 {
            recipe.record_failure();
        }
        assert_eq!(recipe.status, RecipeStatus::Disabled);
    }
}
