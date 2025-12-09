//! Transcript rendering for consistent pipeline visibility (v0.0.179).
//!
//! Two modes:
//! - debug OFF: Theatre mode - cinematic IT department experience (v0.0.81)
//! - debug ON: Full troubleshooting view with stages and timings
//!
//! v0.0.88: Removed unused render_clean functions (theatre_render is used instead).
//! v0.0.179: Modularized into domain-focused submodules.

mod answer_source;
mod debug_render;
mod event_renders;
mod helpers;
mod render;
mod tests;

// Re-export main render functions
pub use render::render;

// Keep these for external use and tests
#[allow(unused_imports)]
pub use render::render_with_options;
#[allow(unused_imports)]
pub use helpers::{format_outcome, reliability_color, truncate};
