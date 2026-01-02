//! Dialogue Renderer - Phase 89
//!
//! Renders specialist conversations in natural language for fly-on-the-wall display.
//! VISION.md: "Show natural language dialog between players"
//! "User reads the whole communication like a fly on the wall"

mod types;
mod renderers;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use types::{Dialogue, DialogueMood, DialogueTurn, Speaker};

// Re-export all public functions
pub use renderers::{render_dialogue, render_dialogue_compact, render_dialogue_plain};
pub use utils::{dialogue_fun_fact, is_dialogue_query};
