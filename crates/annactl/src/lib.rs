//! Annactl library - exposes modules for testing
//!
//! Phase 0.3e: Export modules for integration tests
//! Phase 3.8: Export context_detection for adaptive CLI tests

pub mod context_detection;
pub mod errors;
pub mod internal_dialogue; // Beta.55: LLM internal dialogue
pub mod llm_integration; // Beta.55: LLM integration
pub mod logging;
pub mod output;
pub mod rpc_client; // RPC client for daemon communication
pub mod tui; // Phase 5.7: TUI REPL with ratatui
