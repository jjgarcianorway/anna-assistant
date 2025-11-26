//! Anna Control - CLI client for Anna Assistant
//!
//! v6.57.0: Brutal cleanup - single pipeline architecture
//!
//! All queries flow through the unified pipeline:
//! planner_core → executor_core → interpreter_core → trace_renderer
//!
//! NO legacy handlers, NO hardcoded recipes, NO shortcut paths.

use anyhow::Result;

// Module declarations
pub mod errors;

// Core architecture modules
mod cli;
mod debug;
pub mod llm;
mod llm_query_handler;
mod runtime;
pub mod state;
pub mod telemetry;

// Feature modules
mod action_executor;
pub mod action_plan_executor;
mod brain_command;
mod context_engine;
// REMOVED in 6.57.0: plan_command - depended on legacy orchestrator
// REMOVED in 6.57.0: selftest_command - depended on legacy orchestrator
mod diagnostic_formatter;
// REMOVED in 6.57.0: sysadmin_answers - hardcoded answer templates
// REMOVED in 6.57.0: deterministic_answers - bypassed the pipeline
// REMOVED in v7.0.0: planner_query_handler - replaced by query_handler_v7
pub mod query_handler_v7; // v7.0.0: Clean brain architecture
pub mod query_handler_v8; // v8.0.0: Pure LLM-driven architecture
pub mod query_handler_v10; // v10.0.0: Evidence-based architecture
mod approval_ui;
mod net_diagnostics;
mod dialogue_v3_json;
mod first_run;
mod hardware_questions;
mod health;
mod help_commands;
mod historian_cli;
mod intent_router;
mod internal_dialogue;
// REMOVED in 6.57.0: json_types - depended on deleted caretaker_brain
mod llm_integration;
mod llm_wizard;
mod logging;
mod model_catalog;
mod model_setup_wizard;
mod output;
mod personality_commands;
mod power_diagnostics;
// REMOVED in 6.57.0: query_handler - legacy recipe-based handler
mod reflection_helper;
// REMOVED in 6.57.0: recipe_formatter - legacy recipe formatting
mod repair;
mod rpc_client;
mod runtime_prompt;
mod startup;
mod status_command;
mod status_health;
mod system_prompt_v2;
mod system_prompt_v3_json;
mod system_query;
mod systemd;
mod system_report;
mod telemetry_truth;
mod unified_query_handler;
mod version_banner;

/// Application entry point
///
/// Beta.200: Delegates to runtime::run() for all application logic.
/// Beta.240: Initializes debug flags and prints stats on exit (if enabled)
/// This keeps main.rs minimal and focused on being an entry point.
#[tokio::main]
async fn main() -> Result<()> {
    // Beta.240: Initialize debug flags from environment variables
    debug::init_debug_flags();

    // Run the application
    let result = runtime::run().await;

    // Beta.240: Print RPC stats on exit if debug enabled (developer-only)
    // Note: Stats printing handled per-client in status_command, llm_query_handler, etc.
    // This is just a reminder that debug output is opt-in via ANNA_DEBUG_RPC_STATS=1

    result
}
