//! Recipe execution data types (v0.0.406).

use crate::recipe_file::format::FileRecipe;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Context for recipe execution
#[derive(Debug, Clone)]
pub struct RecipeContext {
    /// User's home directory
    pub home_dir: String,
    /// Current working directory
    pub cwd: String,
    /// User ID (for permission checks)
    pub user_id: Option<String>,
    /// Pre-collected probe outputs (from earlier stages)
    pub probe_outputs: HashMap<String, String>,
    /// Whether to actually execute commands (false = dry run)
    pub execute: bool,
    /// Confirmation callback (returns true if user confirms)
    pub confirm_callback: Option<fn(&str) -> bool>,
}

impl Default for RecipeContext {
    fn default() -> Self {
        Self {
            home_dir: std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()),
            cwd: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "/tmp".to_string()),
            user_id: None,
            probe_outputs: HashMap::new(),
            execute: true,
            confirm_callback: None,
        }
    }
}

/// Result of executing a single step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Step ID
    pub id: String,
    /// Command that was run
    pub command: String,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Exit code
    pub exit_code: i32,
    /// Execution time in milliseconds
    pub duration_ms: u64,
    /// Whether step was skipped (condition not met or dry run)
    pub skipped: bool,
    /// Extracted variables from output
    pub extracted: HashMap<String, String>,
}

/// Result of executing an entire recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Recipe ID that was executed
    pub recipe_id: String,
    /// Individual step results
    pub steps: Vec<StepResult>,
    /// Accumulated variables for template rendering
    pub variables: HashMap<String, String>,
    /// Whether all steps succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Total execution time in milliseconds
    pub total_duration_ms: u64,
    /// Whether confirmation was needed
    pub confirmation_required: bool,
    /// Whether execution was a dry run
    pub dry_run: bool,
}

impl ExecutionResult {
    /// Create a successful empty result
    pub fn empty(recipe_id: String) -> Self {
        Self {
            recipe_id,
            steps: vec![],
            variables: HashMap::new(),
            success: true,
            error: None,
            total_duration_ms: 0,
            confirmation_required: false,
            dry_run: false,
        }
    }

    /// Create a failed result
    pub fn failed(recipe_id: String, error: impl Into<String>) -> Self {
        Self {
            recipe_id,
            steps: vec![],
            variables: HashMap::new(),
            success: false,
            error: Some(error.into()),
            total_duration_ms: 0,
            confirmation_required: false,
            dry_run: false,
        }
    }
}

/// Recipe match result
#[derive(Debug, Clone)]
pub struct RecipeMatchResult {
    /// The matched recipe
    pub recipe: FileRecipe,
    /// Match confidence (0-100)
    pub confidence: u8,
    /// Which criteria matched
    pub matched_criteria: Vec<String>,
}
