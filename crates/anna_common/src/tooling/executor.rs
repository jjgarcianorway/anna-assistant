//! Tool Executor - v6.59.0
//!
//! Single executor that runs actions through the tool catalog.
//! All commands must pass through this executor.

use super::actions::{Action, ActionResult, execute_action, map_query_to_actions};
use super::catalog::{ToolId, ToolResult, tool_catalog, get_tool, run_tool};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Errors that can occur during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionError {
    /// Tool is not in the catalog (programming error)
    ToolNotInCatalog { tool_id: String },
    /// Tool binary is not installed on the system
    ToolNotInstalled { binary: String, suggestion: String },
    /// Tool execution failed
    ExecutionFailed { tool: String, error: String },
    /// No actions could be mapped from the query
    NoActionsMatched { query: String },
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ToolNotInCatalog { tool_id } => {
                write!(f, "Internal error: tool '{}' is not in catalog", tool_id)
            }
            Self::ToolNotInstalled { binary, suggestion } => {
                write!(f, "The '{}' command is not installed. {}", binary, suggestion)
            }
            Self::ExecutionFailed { tool, error } => {
                write!(f, "'{}' failed: {}", tool, error)
            }
            Self::NoActionsMatched { query } => {
                write!(f, "I don't understand how to answer: {}", query)
            }
        }
    }
}

/// Result of executing a natural language query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryExecutionResult {
    pub query: String,
    pub actions_executed: Vec<String>,
    pub action_results: Vec<ActionResult>,
    pub success: bool,
    pub combined_output: String,
    pub errors: Vec<String>,
}

impl QueryExecutionResult {
    /// Get a summary suitable for LLM interpretation
    pub fn for_llm_summary(&self) -> String {
        let mut summary = String::new();

        summary.push_str(&format!("Query: {}\n", self.query));
        summary.push_str(&format!("Actions: {}\n", self.actions_executed.join(", ")));
        summary.push_str(&format!("Success: {}\n\n", self.success));

        if !self.combined_output.is_empty() {
            summary.push_str("Command Output:\n");
            summary.push_str(&self.combined_output);
            summary.push('\n');
        }

        if !self.errors.is_empty() {
            summary.push_str("\nErrors:\n");
            for err in &self.errors {
                summary.push_str(&format!("- {}\n", err));
            }
        }

        summary
    }
}

/// The main tool executor
pub struct ToolExecutor {
    catalog_map: HashMap<ToolId, super::catalog::ToolSpec>,
}

impl Default for ToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolExecutor {
    /// Create a new executor
    pub fn new() -> Self {
        let catalog_map = tool_catalog()
            .into_iter()
            .map(|spec| (spec.id, spec))
            .collect();

        Self { catalog_map }
    }

    /// Execute a natural language query
    pub fn execute_query(&self, query: &str) -> QueryExecutionResult {
        let actions = map_query_to_actions(query);

        if actions.is_empty() {
            // Check for special queries that need text responses
            let q = query.to_lowercase();
            if q.contains("upgrade") && (q.contains("brain") || q.contains("llm")) {
                return self.handle_upgrade_brain_query(query);
            }

            return QueryExecutionResult {
                query: query.to_string(),
                actions_executed: vec![],
                action_results: vec![],
                success: false,
                combined_output: String::new(),
                errors: vec![format!(
                    "I couldn't determine what system information you need. \
                     Try asking about RAM, CPU, GPU, disk space, updates, or services."
                )],
            };
        }

        let mut action_results = Vec::new();
        let mut errors = Vec::new();
        let mut combined_output = String::new();
        let mut all_success = true;

        for action in &actions {
            let result = execute_action(action);

            if result.success {
                combined_output.push_str(&result.combined_stdout());
                combined_output.push('\n');
            } else if let Some(ref err) = result.error_message {
                errors.push(err.clone());
                all_success = false;
            }

            action_results.push(result);
        }

        QueryExecutionResult {
            query: query.to_string(),
            actions_executed: actions.iter().map(|a| format!("{:?}", a)).collect(),
            action_results,
            success: all_success || !combined_output.trim().is_empty(),
            combined_output: combined_output.trim().to_string(),
            errors,
        }
    }

    /// Handle special "upgrade brain" queries with a helpful text response
    fn handle_upgrade_brain_query(&self, query: &str) -> QueryExecutionResult {
        let response = "\
Anna uses a local LLM (typically via Ollama) for natural language understanding.

To upgrade or change the LLM model:

1. List available models: ollama list
2. Pull a new model: ollama pull mistral (or gemma2, llama3, etc.)
3. Configure Anna to use it by updating the LLM config

Current configuration can be checked with: annactl status

Note: Anna doesn't automatically upgrade her own brain - that's a decision you make!";

        QueryExecutionResult {
            query: query.to_string(),
            actions_executed: vec!["TextResponse".to_string()],
            action_results: vec![],
            success: true,
            combined_output: response.to_string(),
            errors: vec![],
        }
    }

    /// Validate that all required tools are in the catalog
    /// Returns Ok if all tools are present, Err with missing tools otherwise
    pub fn validate_catalog(&self) -> Result<(), Vec<ToolId>> {
        let expected = vec![
            ToolId::FreeMem,
            ToolId::LsCpu,
            ToolId::LsPci,
            ToolId::DfHuman,
            ToolId::PacmanQuery,
            ToolId::IpAddrShow,
            ToolId::SystemctlFailed,
            ToolId::JournalctlErrors,
            ToolId::Uptime,
            ToolId::UnameAll,
        ];

        let missing: Vec<_> = expected
            .into_iter()
            .filter(|id| !self.catalog_map.contains_key(id))
            .collect();

        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }

    /// Run a tool by ID (for health checks and direct tool access)
    pub fn run_tool_by_id(&self, tool_id: ToolId) -> Result<ToolResult, ExecutionError> {
        let spec = self.catalog_map.get(&tool_id).ok_or_else(|| {
            ExecutionError::ToolNotInCatalog {
                tool_id: tool_id.name().to_string(),
            }
        })?;

        let result = run_tool(spec);

        if result.is_not_installed() {
            return Err(ExecutionError::ToolNotInstalled {
                binary: spec.binary.to_string(),
                suggestion: format!(
                    "Install {} to enable this feature.",
                    spec.binary
                ),
            });
        }

        if !result.success {
            return Err(ExecutionError::ExecutionFailed {
                tool: spec.binary.to_string(),
                error: result.stderr.clone(),
            });
        }

        Ok(result)
    }

    /// Check if a specific tool is available
    pub fn is_tool_available(&self, tool_id: ToolId) -> bool {
        if let Some(spec) = self.catalog_map.get(&tool_id) {
            let result = run_tool(spec);
            result.success || !result.is_not_installed()
        } else {
            false
        }
    }

    /// Get catalog statistics
    pub fn catalog_stats(&self) -> (usize, usize) {
        let total = self.catalog_map.len();
        let required = self.catalog_map.values().filter(|s| s.required).count();
        (total, required)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let exec = ToolExecutor::new();
        assert!(exec.catalog_map.len() > 20); // We have ~35 tools
    }

    #[test]
    fn test_validate_catalog() {
        let exec = ToolExecutor::new();
        let result = exec.validate_catalog();
        assert!(result.is_ok(), "Missing tools: {:?}", result.err());
    }

    #[test]
    fn test_execute_ram_query() {
        let exec = ToolExecutor::new();
        let result = exec.execute_query("how much RAM do I have");

        // On a real system, this should work
        if result.success {
            assert!(
                result.combined_output.contains("Mem:") ||
                result.combined_output.contains("total"),
                "RAM output: {}",
                result.combined_output
            );
        }
    }

    #[test]
    fn test_execute_cpu_query() {
        let exec = ToolExecutor::new();
        let result = exec.execute_query("what CPU model do I have");

        if result.success {
            assert!(
                result.combined_output.contains("CPU") ||
                result.combined_output.contains("Architecture") ||
                result.combined_output.contains("model"),
                "CPU output: {}",
                result.combined_output
            );
        }
    }

    #[test]
    fn test_execute_disk_query() {
        let exec = ToolExecutor::new();
        let result = exec.execute_query("how much free disk space do I have");

        if result.success {
            assert!(
                result.combined_output.contains("Filesystem") ||
                result.combined_output.contains("/"),
                "Disk output: {}",
                result.combined_output
            );
        }
    }

    #[test]
    fn test_execute_upgrade_brain_query() {
        let exec = ToolExecutor::new();
        let result = exec.execute_query("upgrade your brain");

        assert!(result.success);
        assert!(result.combined_output.contains("Ollama"));
        assert!(result.combined_output.contains("ollama pull"));
    }

    #[test]
    fn test_run_tool_by_id() {
        let exec = ToolExecutor::new();

        let result = exec.run_tool_by_id(ToolId::Uptime);
        assert!(result.is_ok() || matches!(result, Err(ExecutionError::ToolNotInstalled { .. })));

        if let Ok(r) = result {
            assert!(r.success);
            assert!(r.stdout.contains("up") || r.stdout.contains("load"));
        }
    }

    #[test]
    fn test_for_llm_summary() {
        let exec = ToolExecutor::new();
        let result = exec.execute_query("what is my uptime");
        let summary = result.for_llm_summary();

        assert!(summary.contains("Query:"));
        assert!(summary.contains("Actions:"));
    }
}
