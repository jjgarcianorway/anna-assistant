//! Anna Control - CLI client for Anna Assistant
//!
//! Beta.146: Modular architecture - main.rs is now a thin wrapper
//!
//! The application logic has been moved to:
//! - cli.rs - Command-line argument parsing
//! - runtime.rs - Application dispatch and mode selection
//! - llm_query_handler.rs - Natural language query handling

use anyhow::Result;

// Module declarations
pub mod errors;

// Beta.146: Core architecture modules
mod cli;
mod runtime;
mod llm_query_handler;

// Feature modules
mod adaptive_help;
mod action_executor;
mod autonomy_command;
mod chronos_commands;
mod collective_commands;
mod conscience_commands;
mod consensus_commands;
mod context_detection;
mod dialogue_v3_json;
mod discard_command;
mod empathy_commands;
mod first_run;
mod health;
mod health_commands;
mod help_commands;
mod historian_cli;
mod init_command;
mod install_command;
mod intent_router;
mod internal_dialogue;
mod json_types;
mod learning_commands;
mod llm_integration;
mod llm_wizard;
mod logging;
mod mirror_commands;
mod model_catalog;
mod model_setup_wizard;
mod monitor_setup;
mod output;
mod personality_commands;
mod predictive_hints;
mod query_handler;
mod recipe_formatter;
mod repair;
mod repl;
mod report_command;
mod report_display;
mod rpc_client;
mod runtime_prompt;
mod sentinel_cli;
mod startup_summary;
mod status_command;
mod suggestion_display;
mod suggestions;
mod system_prompt_v2;
mod system_prompt_v3_json;
mod system_query;
mod systemd;
pub mod tui;
pub mod tui_state;
pub mod tui_v2;
mod upgrade_command;
mod version_banner;

/// Application entry point
///
/// Beta.146: Delegates to runtime::run() for all application logic.
/// This keeps main.rs minimal and focused on being an entry point.
#[tokio::main]
async fn main() -> Result<()> {
    runtime::run().await
}
