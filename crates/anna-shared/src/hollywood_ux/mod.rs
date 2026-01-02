//! Hollywood UX - Unified transcript and terminal renderer (v0.0.431).
//!
//! Provides a cinematic IT department experience with:
//! - Consistent box-drawing terminal style
//! - Transcript storage and persistence
//! - Debug vs user mode separation
//! - Streaming spinner integration
//!
//! Design goals:
//! - Give users the feeling of watching a competent IT department at work
//! - Present internal comms, probes, and outcomes in consistent old-school terminal style
//! - Separate "user view" from "debug view" clearly
//! - Work on 80-column terminals with minimal unicode

pub mod renderer;
pub mod renderer_helpers;
pub mod storage;
pub mod streaming;
pub mod styles;
pub mod types;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod renderer_tests;

pub use renderer::*;
pub use renderer_helpers::*;
pub use storage::*;
pub use streaming::*;
pub use styles::*;
pub use types::*;

/// Default terminal width
pub const DEFAULT_WIDTH: usize = 80;

/// Maximum transcripts to keep on disk
pub const MAX_TRANSCRIPTS: usize = 1000;

/// Maximum transcript file size (1MB)
pub const MAX_TRANSCRIPT_SIZE: usize = 1_000_000;

/// Transcript storage directory name
pub const TRANSCRIPTS_DIR: &str = "transcripts";
