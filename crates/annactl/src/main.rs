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
mod debug; // Beta.240: Debug utilities (developer-only)
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
mod plan_command; // 6.3.0: Arch Wiki-based execution planner
mod selftest_command; // 6.3.1: Built-in capability verification
mod diagnostic_formatter; // Beta.250: Canonical diagnostic report formatting
mod sysadmin_answers; // Beta.263: Sysadmin answer composer (services, disk, logs, network, CPU, memory)
mod net_diagnostics; // Beta.265: Proactive network diagnostics engine
mod dialogue_v3_json;
mod first_run;
mod hardware_questions; // 6.12.1: Knowledge-first hardware answers
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
mod power_diagnostics; // 6.13.0: TLP and power management diagnostics
mod query_handler;
mod reflection_helper; // 6.7.0: Client-side reflection building
mod recipe_formatter;
// 6.3.1: recipes moved to legacy_recipes/ - not compiled in 6.x
// mod recipes; // Deterministic ActionPlan recipes (77+)
mod repair; // Self-healing capabilities
// 6.0.0: repl.rs archived (see tui_legacy/repl.rs)
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
// 6.0.0: TUI modules archived (see tui_legacy/ directory)
mod unified_query_handler; // Unified query path for CLI
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
