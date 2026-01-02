//! Recipe step types and result structures (v0.0.423).
//!
//! Core data structures for recipe steps and their execution results.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::RecipeRiskLevel;

/// A single step in a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RecipeStep {
    /// Explain something to the user (no action)
    Explain {
        /// Explanation text
        text: String,
        /// Optional citation
        citation: Option<String>,
    },
    /// Show a command without running it
    ShowCommand {
        /// Command to show
        command: String,
        /// Description of what it does
        description: String,
    },
    /// Run a command
    RunCommand {
        /// Command to execute
        command: String,
        /// Description
        description: String,
        /// Risk level override
        risk: Option<RecipeRiskLevel>,
        /// Whether to capture output
        capture_output: bool,
        /// Variable name to store output
        output_var: Option<String>,
    },
    /// Run a probe and store result
    RunProbe {
        /// Probe command
        probe: String,
        /// Variable name to store result
        output_var: String,
        /// Description
        description: String,
    },
    /// Append content to a file
    AppendToFile {
        /// File path
        path: String,
        /// Content to append
        content: String,
        /// Whether to create backup
        backup: bool,
    },
    /// Replace pattern in file
    ReplaceInFile {
        /// File path
        path: String,
        /// Pattern to find (regex)
        pattern: String,
        /// Replacement text
        replacement: String,
        /// Whether to create backup
        backup: bool,
    },
    /// Create a file with content
    CreateFile {
        /// File path
        path: String,
        /// File content
        content: String,
        /// Overwrite if exists
        overwrite: bool,
    },
    /// Call another recipe
    CallSubRecipe {
        /// Recipe ID to call
        recipe_id: String,
        /// Variables to pass
        variables: HashMap<String, String>,
    },
    /// Conditional step (if/then/else)
    Conditional {
        /// Condition to evaluate
        condition: super::RecipeCondition,
        /// Steps if condition is true
        then_steps: Vec<RecipeStep>,
        /// Steps if condition is false
        else_steps: Vec<RecipeStep>,
    },
}

/// Result of executing a step
#[derive(Debug, Clone)]
pub struct StepResult {
    /// Whether step succeeded
    pub success: bool,
    /// Human-readable message
    pub message: String,
    /// Command output (if any)
    pub output: Option<String>,
    /// Citation (if any)
    pub citation: Option<String>,
    /// Execution time in ms
    pub duration_ms: u64,
}

impl StepResult {
    pub fn ok(message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            output: None,
            citation: None,
            duration_ms: 0,
        }
    }

    pub fn fail(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            output: None,
            citation: None,
            duration_ms: 0,
        }
    }

    pub fn with_output(mut self, output: &str) -> Self {
        self.output = Some(output.to_string());
        self
    }

    pub fn with_citation(mut self, citation: Option<String>) -> Self {
        self.citation = citation;
        self
    }

    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }
}
