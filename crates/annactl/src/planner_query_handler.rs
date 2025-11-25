//! Planner Query Handler - Routes queries through Planner → Executor → Interpreter
//!
//! v6.41.0: This is the new core architecture for handling inspection queries.
//! Instead of hard-coded handlers, we use LLM-driven planning and execution.
//!
//! v6.42.0: Real LLM integration with Planner and Interpreter.

use anna_common::{
    executor_core, interpreter_core, planner_core, trace_renderer,
    llm_client::{HttpLlmClient, LlmConfig, LlmClient},
    tool_inventory::ToolInventory,
    telemetry::SystemTelemetry,
};
use anyhow::Result;

/// Check if query should use planner core
pub fn should_use_planner(query: &str) -> bool {
    let query_lower = query.to_lowercase();

    // v6.42.0: Expanded patterns for planner core
    let patterns = [
        // Package queries
        "do i have",
        "is installed",
        "have installed",
        "what packages",
        "list packages",
        "show packages",

        // Hardware queries
        "does my cpu",
        "does my gpu",
        "cpu feature",
        "cpu flag",
        "hardware support",

        // DE/WM queries
        "what de",
        "what wm",
        "desktop environment",
        "window manager",
        "which de",
        "which wm",

        // System queries
        "file manager",
        "browser",
        "text editor",
    ];

    patterns.iter().any(|pattern| query_lower.contains(pattern))
}

/// Handle query using Planner → Executor → Interpreter core
/// v6.42.0: Now with real LLM integration
pub async fn handle_with_planner(
    query: &str,
    telemetry: &SystemTelemetry,
    llm_config: Option<&LlmConfig>,
) -> Result<String> {

    // Step 1: Interpret user intent
    let intent = planner_core::interpret_intent(query);

    // Step 2: Detect available tools
    let tool_inventory = ToolInventory::detect();

    // Step 3: Build system signals JSON
    let system_signals = build_system_signals(telemetry);

    // Step 4: Generate command plan (using LLM or fallback)
    let plan = if let Some(config) = llm_config {
        if config.enabled {
            // Try LLM-backed planning
            match HttpLlmClient::new(config.clone()) {
                Ok(client) => {
                    let planner = planner_core::LlmPlanner::new(&client, tool_inventory.clone());
                    match planner.plan(&intent, &system_signals) {
                        Ok(p) => p,
                        Err(_e) => {
                            // LLM planning failed, use fallback
                            planner_core::fallback_plan(&intent, &tool_inventory)
                        }
                    }
                }
                Err(_e) => {
                    // Failed to create LLM client, use fallback
                    planner_core::fallback_plan(&intent, &tool_inventory)
                }
            }
        } else {
            planner_core::fallback_plan(&intent, &tool_inventory)
        }
    } else {
        planner_core::fallback_plan(&intent, &tool_inventory)
    };

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
