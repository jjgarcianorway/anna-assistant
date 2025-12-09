//! v0.0.67: Service Desk narrative renderer.
//!
//! Provides clean "movie-terminal" output for debug OFF mode.
//! Non-negotiables:
//! - No icons, no emojis, no raw probe output
//! - No question marks in Anna's final text
//! - No "would you like"
//! - Citations for factual guidance
//!
//! v0.0.203: Modularized into domain-focused submodules.

mod formatting;
mod output;
mod spinner;
mod tests;
mod types;

// Re-export all types and functions
pub use formatting::{determine_risk_level, format_time_delta, generate_case_id};
pub use output::{
    render_case_start, render_citation, render_clarification, render_collecting_evidence,
    render_evidence_collected, render_greeting, render_header, render_internal_notes,
    render_narrative, render_reliability_line, render_resolution, render_uncited,
};
pub use spinner::{ProgressRenderer, Spinner};
pub use types::{RenderPolicy, RiskLevel, UiConfig, Verbosity};
