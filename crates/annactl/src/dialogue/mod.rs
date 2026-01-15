//! Dialogue - Terminal dialogue rendering for Phase 11.
//!
//! Provides terminal output for human-readable internal dialogue.

pub mod consumer;
pub mod renderer;

pub use consumer::TerminalConsumer;
pub use renderer::{render_line, render_timeline, render_resolution, render_failure};
