//! Real TUI Implementation - Claude-CLI / Codex style interface
//!
//! This module has been refactored into a modular structure under src/tui/
//! for better organization and maintainability. Each component is now in its
//! own focused module (<400 lines each).
//!
//! Module structure:
//! - tui/mod.rs - Re-exports and module organization
//! - tui/event_loop.rs - Main entry point and event handling
//! - tui/render.rs - UI drawing functions
//! - tui/input.rs - Input bar and user input handling
//! - tui/action_plan.rs - ActionPlan execution and rendering
//! - tui/llm.rs - LLM integration and reply generation
//! - tui/state.rs - State management utilities
//! - tui/utils.rs - Helper functions and overlays

// Re-export the main entry point for backward compatibility
pub use crate::tui::run;
pub use crate::tui::TuiMessage;
