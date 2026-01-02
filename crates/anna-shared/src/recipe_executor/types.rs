//! Recipe executor types - execution results and context.

use crate::doc_snippet::DocSnippet;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of executing a recipe
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Whether execution succeeded
    pub success: bool,
    /// Final rendered answer
    pub answer: String,
    /// Step outputs collected
    pub step_outputs: HashMap<String, StepOutput>,
    /// Any errors encountered
    pub errors: Vec<String>,
    /// Audit trail
    pub audit: Vec<AuditEntry>,
    /// Doc sources used
    pub sources: Vec<DocSnippet>,
}

impl ExecutionResult {
    /// Create a new execution result
    pub fn new() -> Self {
        Self {
            success: true,
            answer: String::new(),
            step_outputs: HashMap::new(),
            errors: vec![],
            audit: vec![],
            sources: vec![],
        }
    }
}

impl Default for ExecutionResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Output from a single step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepOutput {
    pub step_id: String,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
}

/// Audit trail entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: u64,
    pub step_id: String,
    pub action: String,
    pub details: String,
    pub backup_path: Option<String>,
}

/// Execution context with parameters and outputs
#[derive(Debug, Clone, Default)]
pub struct ExecutionContext {
    /// Recipe parameters (filled in from query)
    pub params: HashMap<String, String>,
    /// Step outputs so far
    pub outputs: HashMap<String, StepOutput>,
    /// Whether to skip confirmations (auto mode)
    pub auto_confirm: bool,
    /// Ticket ID for audit
    pub ticket_id: Option<String>,
    /// Recipe ID being executed
    pub recipe_id: Option<String>,
}
