//! Service Desk Theatre - Cinematic narrative rendering (v0.0.226).
//!
//! Transforms technical pipeline events into natural dialogue between
//! named IT department personas. Makes users feel like they're watching
//! a real IT team solve their problems.
//!
//! Two modes:
//! - Normal: Cinematic narrative with named personas
//! - Debug: Full technical pipeline visibility
//!
//! v0.0.87: Enhanced dialogue variety and internal communications.
//! v0.0.226: Modularized into domain-focused submodules.

mod builder;
mod formatting;
#[cfg(test)]
mod tests;
mod types;

// Re-export for backwards compatibility
pub use builder::NarrativeBuilder;
pub use formatting::{describe_check, format_case_id};
pub use types::{NarrativeSegment, Speaker};
