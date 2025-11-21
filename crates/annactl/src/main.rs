//! Anna Control - CLI client for Anna Assistant
//!
//! Beta.200: Complete architectural redesign - simplified to three commands only
//!
//! The application logic has been moved to:
//! - cli.rs - Command-line argument parsing (3 commands only)
//! - runtime.rs - Application dispatch and mode selection
//! - llm_query_handler.rs - Natural language query handling

use anyhow::Result;

// Module declarations
pub mod errors;

// Beta.200: Core architecture modules (ONLY these remain)
mod cli;
pub mod llm; // Beta.200: Organized LLM functionality
mod llm_query_handler;
mod runtime;
pub mod state; // Beta.200: Application state management
pub mod telemetry; // Beta.200: Telemetry fetching and caching

// Feature modules (KEEP - essential)
mod action_executor;
pub mod action_plan_executor; // ActionPlan execution engine
mod brain_command; // Beta.217c: Standalone brain diagnostic command
mod context_engine; // Contextual awareness and proactive monitoring
mod dialogue_v3_json;
mod first_run;
mod health;
mod help_commands;
mod historian_cli; // Simplified - basic history only
mod intent_router;
mod internal_dialogue;
mod json_types;
mod llm_integration;
mod llm_wizard;
mod logging;
mod model_catalog;
mod model_setup_wizard;
mod output;
mod personality_commands; // Simplified - core traits only
mod query_handler;
mod recipe_formatter;
mod recipes; // Deterministic ActionPlan recipes (77+)
mod repair; // Self-healing capabilities
mod repl;
mod rpc_client;
mod runtime_prompt;
mod startup; // Beta.211: Welcome engine
mod status_command;
mod system_prompt_v2;
mod system_prompt_v3_json;
mod system_query;
mod systemd;
mod system_report; // Unified system report generator
mod telemetry_truth; // Telemetry truth enforcement
pub mod tui;
pub mod tui_state;
pub mod tui_v2;
mod unified_query_handler; // Unified query path for CLI and TUI
mod version_banner;

/// Application entry point
///
/// Beta.200: Delegates to runtime::run() for all application logic.
/// This keeps main.rs minimal and focused on being an entry point.
#[tokio::main]
async fn main() -> Result<()> {
    runtime::run().await
}
