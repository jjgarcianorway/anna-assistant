//! Progress events for request pipeline visibility (v0.0.204).
//!
//! Provides structured progress updates during request processing.
//!
//! INVARIANT: Progress events are telemetry only - never user-facing content.
//! All string fields are capped to prevent content leakage.

mod event;
mod tests;
mod types;

// Re-export all types and functions
pub use event::{ProgressEvent, ProgressEventType};
pub use types::{DiagnosticText, RequestStage, TimeoutConfig, MAX_DIAGNOSTIC_LENGTH};
