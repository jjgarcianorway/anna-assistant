//! Transcript event model for consistent pipeline visibility (v0.0.178).
//!
//! Single source of truth for rendering request/response conversations.
//! Enforces size cap with diagnostic surfacing (COST phase).
//!
//! v0.0.178: Modularized into domain-focused submodules.

mod actor;
mod core;
mod event;
mod event_kind;
mod outcome;

// Re-export all types
pub use actor::Actor;
pub use core::Transcript;
pub use event::TranscriptEvent;
pub use event_kind::TranscriptEventKind;
pub use outcome::StageOutcome;
