//! Annactl library - exposes modules for testing
//!
//! Phase 0.3e: Export modules for integration tests
//! Phase 3.8: Export context_detection for adaptive CLI tests

pub mod action_plan_executor; // Beta.147: ActionPlan execution engine
pub mod context_detection;
pub mod context_engine; // Version 150: Contextual awareness and proactive monitoring
pub mod dialogue_v3_json; // Beta.145: V3 JSON dialogue runner
pub mod errors;
pub mod health; // Modular health command implementations
pub mod internal_dialogue; // Beta.55: LLM internal dialogue
pub mod llm_integration; // Beta.55: LLM integration
pub mod logging;
pub mod output;
pub mod predictive_hints; // Predictive hint system
pub mod query_handler; // Beta.144: Unified 3-tier query architecture
pub mod recipe_formatter; // Beta.90: Recipe answer formatting
pub mod recipes; // Beta.151: Deterministic ActionPlan recipes
pub mod rpc_client; // RPC client for daemon communication
pub mod system_prompt_v2; // Beta.142: Strict reasoning discipline prompts
pub mod system_prompt_v3_json; // Beta.143: JSON runtime contract
pub mod system_query; // Beta.91: System telemetry queries
pub mod system_report; // Version 150: Unified system report generator
pub mod systemd; // Systemd service management
pub mod telemetry_truth; // Version 150: Telemetry truth enforcement
mod tui; // Beta.125: Modular TUI implementation (src/tui/ directory)
pub mod tui_old; // Phase 5.7: TUI REPL with ratatui (deprecated, kept for reference)
pub mod tui_state; // Beta.90: TUI state management
pub mod tui_v2; // Beta.90: Real TUI implementation (re-exports from tui/)
pub mod unified_query_handler; // Version 149: Unified query path for CLI and TUI
