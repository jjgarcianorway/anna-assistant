//! TUI Module - Modular organization
//!
//! This module organizes the TUI implementation into focused components:
//! - event_loop: Main entry point and event handling
//! - render: UI drawing functions
//! - input: Input bar and user input handling
//! - action_plan: ActionPlan execution and rendering
//! - state: State management utilities (Beta.213: uses deterministic welcome engine)
//! - utils: Helper functions and overlays
//! - brain: Brain diagnostics panel (Beta.218)
//! - formatting: Canonical answer format parsing and styling (Beta.260)
//! - flow: Startup welcome, exit summary, and shared hints (Beta.261)
//! - layout: Canonical TUI layout grid computation (Beta.262)

mod action_plan;
mod brain;
mod event_loop;
mod flow;
mod formatting;
mod input;
mod layout;
mod render;
mod state;
mod utils;

// Re-export main entry point
pub use event_loop::run;

// Re-export message type for external use
pub use event_loop::TuiMessage;
