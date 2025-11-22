//! Annactl library - exposes modules for testing
//!
//! Beta.200: Simplified architecture - removed all non-mandated modules
//! Beta.240: Added debug utilities for developer observability

pub mod action_plan_executor; // ActionPlan execution engine
pub mod cli_output; // JSON output formatting for CLI commands
pub mod context_engine; // Contextual awareness and proactive monitoring
pub mod debug; // Beta.240: Debug utilities (developer-only, unstable)
pub mod diagnostic_formatter; // Beta.250: Canonical diagnostic report formatting
pub mod dialogue_v3_json; // V3 JSON dialogue runner
pub mod errors;
pub mod health; // Modular health command implementations
pub mod internal_dialogue; // LLM internal dialogue
pub mod llm_integration; // LLM integration
pub mod logging;
pub mod output; // Answer normalization module (Beta.208)
pub mod startup; // Startup context awareness and welcome reports (Beta.209)
pub mod query_handler; // Unified 3-tier query architecture
pub mod recipe_formatter; // Recipe answer formatting
pub mod recipes; // Deterministic ActionPlan recipes (77+)
pub mod rpc_client; // RPC client for daemon communication
pub mod sysadmin_answers; // Beta.263: Sysadmin answer composer (services, disk, logs)
pub mod system_prompt_v2; // Strict reasoning discipline prompts
pub mod system_prompt_v3_json; // JSON runtime contract
pub mod system_query; // System telemetry queries
pub mod system_report; // Unified system report generator
pub mod systemd; // Systemd service management
pub mod telemetry_truth; // Telemetry truth enforcement
mod tui; // Modular TUI implementation (src/tui/ directory)
// pub mod tui_old; // Removed in Beta.217d cleanup
pub mod tui_state; // TUI state management
pub mod tui_v2; // Real TUI implementation (re-exports from tui/)
pub mod unified_query_handler; // Unified query path for CLI and TUI
