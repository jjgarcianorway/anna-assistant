//! Transcript Renderer - Cinematic and Debug mode rendering (v0.0.413).
//!
//! Renders TranscriptSegments to terminal output with proper styling.
//! Supports both Hollywood IT department view and developer debug view.

mod config;
mod formatters;
mod segment_renderers;
mod transcript;

// Re-export public API
pub use config::RenderConfig;
pub use formatters::{format_answer_with_evidence, format_error_with_context};
pub use transcript::{render_segment, render_transcript};
