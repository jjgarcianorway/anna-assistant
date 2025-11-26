//! Planner Query Handler - Routes queries through Planner → Executor → Interpreter
//!
//! v6.41.0: This is the new core architecture for handling inspection queries.
//! Instead of hard-coded handlers, we use LLM-driven planning and execution.
//!
//! v6.42.0: Real LLM integration with Planner and Interpreter.
//!
//! v6.59.0: FAST PATH using tooling::ToolExecutor for common queries.
//! The LLM is NO LONGER allowed to generate arbitrary shell commands.
//! Instead, queries first go through the typed Action vocabulary.
//! Only complex queries that can't be handled by Actions fall back to LLM interpretation.

use anna_common::{
    executor_core, interpreter_core, planner_core, trace_renderer,
    llm_client::{HttpLlmClient, LlmConfig, LlmClient},
    tool_inventory::ToolInventory,
    telemetry::SystemTelemetry,
    tooling::ToolExecutor,
};
use anyhow::Result;

/// Check if query should use planner core
pub fn should_use_planner(query: &str) -> bool {
    let query_lower = query.to_lowercase();

    // v6.42.0: Expanded patterns for planner core
    // v6.55.1: Fixed patterns to avoid false positives

    // Package-specific queries that need planner
    let package_patterns = [
        "do i have",      // "do i have steam installed"
        "installed",      // "is firefox installed"
        "what packages",
        "list packages",
        "show packages",
    ];

    // Hardware queries
    let hardware_patterns = [
        "does my cpu",
        "does my gpu",
        "cpu feature",
        "cpu flag",
        "cpu support",
        "hardware support",
    ];

    // DE/WM queries
    let de_patterns = [
        "what de",
        "what wm",
        "desktop environment",
        "window manager",
        "which de",
        "which wm",
    ];

    // System queries
    let system_patterns = [
        "file manager",
        "browser",
        "text editor",
    ];

    // Exclusion patterns - these should NOT use planner (simple telemetry queries)
    let exclusion_patterns = [
        "how much ram",
        "how much memory",
        "system status",
        "cpu usage",
        "memory usage",
        "disk usage",
        "disk space",
    ];

    // Check exclusions first
    if exclusion_patterns.iter().any(|p| query_lower.contains(p)) {
        return false;
    }

    // Check all pattern groups
    package_patterns.iter().any(|p| query_lower.contains(p))
        || hardware_patterns.iter().any(|p| query_lower.contains(p))
        || de_patterns.iter().any(|p| query_lower.contains(p))
        || system_patterns.iter().any(|p| query_lower.contains(p))
}

/// Handle query using Planner → Executor → Interpreter core
/// v6.42.0: Now with real LLM integration
/// v6.59.0: FAST PATH - Try typed Actions first, then fall back to LLM interpretation
pub async fn handle_with_planner(
    query: &str,
    telemetry: &SystemTelemetry,
    llm_config: Option<&LlmConfig>,
) -> Result<String> {

    // === v6.59.0 FAST PATH ===
    // Try the typed Action vocabulary FIRST before asking LLM
    // This prevents LLM from generating arbitrary shell commands
    let executor = ToolExecutor::new();
    let tool_result = executor.execute_query(query);

    if tool_result.success && !tool_result.combined_output.trim().is_empty() {
        // Fast path succeeded! Now interpret the results with LLM if available
        let raw_output = tool_result.combined_output.clone();

        // Use LLM to summarize the output if available
        let answer = if let Some(config) = llm_config {
            if config.enabled {
                match HttpLlmClient::new(config.clone()) {
                    Ok(client) => {
                        summarize_tool_output(&client, query, &raw_output)
                    }
                    Err(_) => {
                        // No LLM - return raw output with formatting
                        format_raw_tool_output(query, &raw_output, &tool_result.actions_executed)
                    }
                }
            } else {
                format_raw_tool_output(query, &raw_output, &tool_result.actions_executed)
            }
        } else {
            format_raw_tool_output(query, &raw_output, &tool_result.actions_executed)
        };

        return Ok(answer);
    }

    // === FALLBACK PATH ===
    // If no actions matched or execution failed, try original planner pipeline
    // But ONLY for complex queries that need LLM reasoning

    // Step 1: Interpret user intent
    let intent = planner_core::interpret_intent(query);

    // Step 2: Detect available tools
    let tool_inventory = ToolInventory::detect();

    // Step 3: Build system signals JSON
    let system_signals = build_system_signals(telemetry);

    // Step 4: Generate command plan (using fallback ONLY - no LLM shell generation)
    // v6.59.0: We NO LONGER let LLM generate arbitrary shell commands
    let plan = planner_core::fallback_plan(&intent, &tool_inventory);

    // Step 4.5: Request approval if needed (v6.43.0)
    if crate::approval_ui::requires_approval(&plan) {
        match crate::approval_ui::request_approval(&plan) {
            Ok(approved) => {
                if !approved {
                    return Ok("Operation cancelled by user.".to_string());
                }
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to get user approval: {}", e));
            }
        }
    }

    // Step 5: Execute the plan
    let exec_result = executor_core::execute_plan(&plan)?;

    // Step 6: Interpret results (using LLM or fallback)
    let answer = if let Some(config) = llm_config {
        if config.enabled {
            // Try LLM-backed interpretation
            match HttpLlmClient::new(config.clone()) {
                Ok(client) => {
                    let interpreter = interpreter_core::LlmInterpreter::new(&client);
                    interpreter.interpret(&intent, &exec_result, &system_signals)
                }
                Err(_e) => {
                    // Failed to create LLM client, use fallback
                    interpreter_core::interpret_without_llm(&intent, &exec_result)
                }
            }
        } else {
            interpreter_core::interpret_without_llm(&intent, &exec_result)
        }
    } else {
        interpreter_core::interpret_without_llm(&intent, &exec_result)
    };

    // Step 7: Render output with trace
    let mut output = String::new();

    // Main answer
    output.push_str(&answer.answer);
    output.push('\n');

    // Show trace (visible thinking traces requirement)
    output.push_str(&trace_renderer::render_trace(&intent, &exec_result, &answer));

    // Source attribution
    output.push_str(&format!("\nSource: {}", answer.source));

    Ok(output)
}

/// Summarize tool output using LLM
fn summarize_tool_output(client: &HttpLlmClient, query: &str, raw_output: &str) -> String {
    let system_prompt = "You are a helpful Linux system assistant. \
        Summarize command output concisely and helpfully.";

    let user_prompt = format!(
        "The user asked: \"{}\"\n\n\
         System output:\n```\n{}\n```\n\n\
         Summarize this briefly.",
        query,
        raw_output.chars().take(2000).collect::<String>()
    );

    let schema = r#"{"type": "object", "properties": {"summary": {"type": "string"}}}"#;

    match client.call_json(system_prompt, &user_prompt, schema) {
        Ok(json) => {
            json.get("summary")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format_raw_tool_output(query, raw_output, &[]))
        }
        Err(_) => format_raw_tool_output(query, raw_output, &[]),
    }
}

/// Format raw tool output when LLM is not available
fn format_raw_tool_output(query: &str, raw_output: &str, actions: &[String]) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&format!("📋  Query: {}\n\n", query));

    // Actions taken
    if !actions.is_empty() {
        output.push_str("🔧  Actions: ");
        output.push_str(&actions.join(", "));
        output.push_str("\n\n");
    }

    // Output
    output.push_str("📊  Result:\n");
    output.push_str(raw_output);

    output
}

/// Build system signals JSON from telemetry
fn build_system_signals(_telemetry: &SystemTelemetry) -> serde_json::Value {
    // v6.42.0: Simplified for now - full telemetry integration later
    serde_json::json!({
        "system": "arch_linux",
        "context": "system_query"
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_use_planner() {
        assert!(should_use_planner("do I have games installed?"));
        assert!(should_use_planner("what DE am I running?"));
        assert!(should_use_planner("does my CPU have SSE?"));
        assert!(should_use_planner("do I have any file manager?"));

        // Should not use planner
        assert!(!should_use_planner("how much ram do I have?"));
        assert!(!should_use_planner("show me system status"));
    }

    #[test]
    fn test_expanded_patterns() {
        // v6.42.0: Test expanded query patterns
        assert!(should_use_planner("do I have steam installed?"));
        assert!(should_use_planner("is firefox installed?"));
        assert!(should_use_planner("what packages do I have?"));
        assert!(should_use_planner("does my CPU support AVX2?"));
        assert!(should_use_planner("which file manager am I using?"));

        // These should still not match
        assert!(!should_use_planner("how much ram do I have?"));
        assert!(!should_use_planner("show me system status"));
        assert!(!should_use_planner("what is the weather?"));
    }
}
