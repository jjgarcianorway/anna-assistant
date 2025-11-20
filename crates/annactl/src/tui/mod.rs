//! TUI Module - Modular organization
//!
//! This module organizes the TUI implementation into focused components:
//! - event_loop: Main entry point and event handling
//! - render: UI drawing functions
//! - input: Input bar and user input handling
//! - action_plan: ActionPlan execution and rendering
//! - llm: LLM integration and reply generation
//! - state: State management utilities
//! - utils: Helper functions and overlays

mod action_plan;
mod event_loop;
mod input;
mod llm;
mod render;
mod state;
mod utils;

// Re-export main entry point
pub use event_loop::run;

// Re-export message type for external use
pub use event_loop::TuiMessage;
