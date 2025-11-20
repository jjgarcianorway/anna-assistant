//! Annactl library - exposes modules for testing
//!
//! Phase 0.3e: Export modules for integration tests
//! Phase 3.8: Export context_detection for adaptive CLI tests

pub mod context_detection;
pub mod dialogue_v3_json; // Beta.145: V3 JSON dialogue runner
pub mod errors;
pub mod internal_dialogue; // Beta.55: LLM internal dialogue
pub mod system_prompt_v2; // Beta.142: Strict reasoning discipline prompts
pub mod system_prompt_v3_json; // Beta.143: JSON runtime contract
pub mod llm_integration; // Beta.55: LLM integration
pub mod logging;
pub mod output;
pub mod query_handler; // Beta.144: Unified 3-tier query architecture
pub mod recipe_formatter; // Beta.90: Recipe answer formatting
pub mod rpc_client; // RPC client for daemon communication
pub mod system_query; // Beta.91: System telemetry queries
pub mod tui; // Phase 5.7: TUI REPL with ratatui (old)
pub mod tui_state; // Beta.90: TUI state management
pub mod tui_v2; // Beta.90: Real TUI implementation
