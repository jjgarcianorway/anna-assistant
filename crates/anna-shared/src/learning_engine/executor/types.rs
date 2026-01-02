//! Execution result types for recipe executor.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of recipe execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Recipe ID that was executed
    pub recipe_id: String,
    /// Whether execution was successful
    pub success: bool,
    /// Short answer
    pub short_answer: String,
    /// Detailed answer
    pub detailed_answer: String,
    /// Probe results
    pub probe_results: HashMap<String, ProbeResult>,
    /// Variables extracted
    pub variables: HashMap<String, String>,
    /// Execution time in milliseconds
    pub execution_ms: u64,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Whether this was a recipe-based resolution (no LLM)
    pub recipe_based: bool,
}

impl ExecutionResult {
    /// Create a successful result
    pub fn success(recipe_id: &str, short: &str, detailed: &str) -> Self {
        Self {
            recipe_id: recipe_id.to_string(),
            success: true,
            short_answer: short.to_string(),
            detailed_answer: detailed.to_string(),
            probe_results: HashMap::new(),
            variables: HashMap::new(),
            execution_ms: 0,
            error: None,
            recipe_based: true,
        }
    }

    /// Create a failed result
    pub fn failure(recipe_id: &str, error: &str) -> Self {
        Self {
            recipe_id: recipe_id.to_string(),
            success: false,
            short_answer: String::new(),
            detailed_answer: String::new(),
            probe_results: HashMap::new(),
            variables: HashMap::new(),
            execution_ms: 0,
            error: Some(error.to_string()),
            recipe_based: true,
        }
    }

    /// Add a probe result
    pub fn with_probe(mut self, id: &str, result: ProbeResult) -> Self {
        self.probe_results.insert(id.to_string(), result);
        self
    }

    /// Set execution time
    pub fn with_time(mut self, ms: u64) -> Self {
        self.execution_ms = ms;
        self
    }
}

/// Result of a single probe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    /// Probe ID
    pub probe_id: String,
    /// Whether probe succeeded
    pub success: bool,
    /// Output from probe
    pub output: String,
    /// Execution time in ms
    pub duration_ms: u64,
    /// Error if failed
    pub error: Option<String>,
}

impl ProbeResult {
    /// Create a successful probe result
    pub fn ok(probe_id: &str, output: &str, duration_ms: u64) -> Self {
        Self {
            probe_id: probe_id.to_string(),
            success: true,
            output: output.to_string(),
            duration_ms,
            error: None,
        }
    }

    /// Create a failed probe result
    pub fn failed(probe_id: &str, error: &str) -> Self {
        Self {
            probe_id: probe_id.to_string(),
            success: false,
            output: String::new(),
            duration_ms: 0,
            error: Some(error.to_string()),
        }
    }
}
