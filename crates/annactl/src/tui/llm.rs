//! LLM Integration - TUI reply generation via unified_query_handler
//!
//! Beta.205: Dead code removed - TUI MUST use unified_query_handler for consistency
//!
//! This module provides a single entry point for TUI query processing:
//! - generate_reply_streaming() - Calls handle_unified_query() for all queries
//!
//! ARCHITECTURAL GUARANTEE:
//! Both CLI (llm_query_handler.rs) and TUI (this file) MUST use handle_unified_query()
//! to ensure identical responses for the same question. Any deviation breaks determinism.

use crate::system_query::query_system_telemetry;
use crate::tui_state::AnnaTuiState;
use crate::unified_query_handler::{handle_unified_query, AnswerConfidence, UnifiedQueryResult};
use anna_common::llm::LlmConfig;
use tokio::sync::mpsc;

use super::event_loop::TuiMessage;

/// Generate reply using unified query handler (Beta.149+)
///
/// This ensures TUI and CLI get IDENTICAL responses for the same question.
/// All TUI queries MUST flow through this function to maintain consistency.
pub async fn generate_reply_streaming(
    input: &str,
    state: &AnnaTuiState,
    tx: mpsc::Sender<TuiMessage>,
) -> String {
    // Get telemetry
    let telemetry = match query_system_telemetry() {
        Ok(t) => t,
        Err(e) => {
            return format!("⚠ Error reading system telemetry: {}", e);
        }
    };

    // Get LLM config from state
    let model_name = if state.llm_panel.model_name == "None"
        || state.llm_panel.model_name == "Unknown"
        || state.llm_panel.model_name == "Ollama N/A"
    {
        "llama3.1:8b"
    } else {
        &state.llm_panel.model_name
    };
    let llm_config = LlmConfig::local("http://127.0.0.1:11434/v1", model_name);

    // Use unified query handler (CLI and TUI consistency guarantee)
    match handle_unified_query(input, &telemetry, &llm_config).await {
        Ok(UnifiedQueryResult::DeterministicRecipe {
            recipe_name,
            action_plan,
        }) => {
            // Format recipe output for TUI
            format!(
                "**🎯 Using deterministic recipe: {}**\n\n\
                 ## Analysis\n{}\n\n\
                 ## Goals\n{}\n\n\
                 ## Commands\n{}\n\n\
                 ## Notes\n{}",
                recipe_name,
                action_plan.analysis,
                action_plan
                    .goals
                    .iter()
                    .enumerate()
                    .map(|(i, g)| format!("{}. {}", i + 1, g))
                    .collect::<Vec<_>>()
                    .join("\n"),
                action_plan
                    .command_plan
                    .iter()
                    .enumerate()
                    .map(|(i, cmd)| format!(
                        "{}. {} [Risk: {:?}]\n   $ {}",
                        i + 1,
                        cmd.description,
                        cmd.risk_level,
                        cmd.command
                    ))
                    .collect::<Vec<_>>()
                    .join("\n"),
                action_plan.notes_for_user
            )
        }
        Ok(UnifiedQueryResult::Template { command, output }) => {
            // Format template output for TUI
            format!("**Running:** `{}`\n\n```\n{}\n```", command, output)
        }
        Ok(UnifiedQueryResult::ActionPlan {
            action_plan,
            raw_json: _,
        }) => {
            // Format action plan for TUI
            format!(
                "## Analysis\n{}\n\n\
                 ## Goals\n{}\n\n\
                 ## Commands\n{}\n\n\
                 ## Notes\n{}",
                action_plan.analysis,
                action_plan
                    .goals
                    .iter()
                    .enumerate()
                    .map(|(i, g)| format!("{}. {}", i + 1, g))
                    .collect::<Vec<_>>()
                    .join("\n"),
                action_plan
                    .command_plan
                    .iter()
                    .enumerate()
                    .map(|(i, cmd)| format!(
                        "{}. {} [Risk: {:?}]\n   $ {}",
                        i + 1,
                        cmd.description,
                        cmd.risk_level,
                        cmd.command
                    ))
                    .collect::<Vec<_>>()
                    .join("\n"),
                action_plan.notes_for_user
            )
        }
        Ok(UnifiedQueryResult::ConversationalAnswer {
            answer,
            confidence,
            sources,
        }) => {
            // Format conversational answer for TUI
            let confidence_str = match confidence {
                AnswerConfidence::High => "✅ High",
                AnswerConfidence::Medium => "🟡 Medium",
                AnswerConfidence::Low => "⚠️  Low",
            };

            format!(
                "{}\n\n---\n*Confidence: {} | Sources: {}*",
                answer,
                confidence_str,
                sources.join(", ")
            )
        }
        Err(e) => {
            // Unified handler error
            format!("⚠ Query processing error: {}", e)
        }
    }
}
