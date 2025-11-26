//! Planner Query Handler - v6.60.0 Pure LLM Orchestration
//!
//! ALL queries go through the LLM orchestration loop:
//! 1. Query + tool catalog -> LLM Planner -> Command plan
//! 2. Command plan -> Executor -> Raw output
//! 3. Raw output -> LLM Interpreter -> Human answer
//!
//! The orchestrator NEVER decides which tools to run.
//! The orchestrator NEVER interprets results.
//! ALL decisions are delegated to the LLM.

use anna_common::{
    llm_client::LlmConfig,
    telemetry::SystemTelemetry,
    tooling::{LlmOrchestrator, get_tool_catalog_for_llm},
};
use anyhow::Result;

/// Check if query should use planner core
/// v6.60.0: ALWAYS returns true - everything goes through LLM orchestration
pub fn should_use_planner(_query: &str) -> bool {
    true // All queries use LLM orchestration
}

/// Handle query using pure LLM orchestration
/// v6.60.0: Complete rewrite - NO hardcoded logic
pub async fn handle_with_planner(
    query: &str,
    _telemetry: &SystemTelemetry,
    llm_config: Option<&LlmConfig>,
) -> Result<String> {
    // Check if LLM is available
    let config = match llm_config {
        Some(c) if c.enabled => c.clone(),
        _ => {
            // No LLM available - return helpful message
            return Ok(format!(
                "LLM is not configured or disabled.\n\n\
                 Anna requires a local LLM (e.g., Ollama) to process queries.\n\n\
                 To enable:\n\
                 1. Install Ollama: curl -fsSL https://ollama.com/install.sh | sh\n\
                 2. Pull a model: ollama pull llama3.2:3b\n\
                 3. Restart Anna\n\n\
                 Query received: \"{}\"",
                query
            ));
        }
    };

    // Create the LLM orchestrator
    let orchestrator = match LlmOrchestrator::new(config) {
        Ok(o) => o,
        Err(e) => {
            return Ok(format!(
                "Failed to connect to LLM: {}\n\n\
                 Make sure Ollama is running: systemctl start ollama\n\
                 Or check if the endpoint is correct.",
                e
            ));
        }
    };

    // Process the query through the full orchestration loop
    let result = orchestrator.process_query(query);

    if result.success {
        Ok(result.answer)
    } else {
        Ok(format!("Query failed: {}", result.answer))
    }
}

/// Get tool catalog description for display/debugging
pub fn get_available_tools() -> String {
    get_tool_catalog_for_llm()
}

/// Build system signals JSON from telemetry (for future use)
fn build_system_signals(_telemetry: &SystemTelemetry) -> serde_json::Value {
    serde_json::json!({
        "system": "arch_linux",
        "context": "system_query"
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_always_uses_planner() {
        assert!(should_use_planner("any query"));
        assert!(should_use_planner("how much RAM"));
        assert!(should_use_planner("what DE am I running"));
    }

    #[test]
    fn test_get_available_tools() {
        let tools = get_available_tools();
        assert!(tools.contains("free_mem"));
        assert!(tools.contains("lscpu"));
    }
}
