//! Timeline - Human-readable internal dialogue reconstruction.
//!
//! Phase 11: Converts specialist activity into coherent, chronological dialogue.
//!
//! This module provides:
//! - `types`: Core timeline and entry types
//! - `narrator`: Converts entries to human-readable dialogue
//! - `builder`: Constructs timelines from events
//! - `redaction`: Rules for hiding internal-only data
//! - `replay`: Deterministic replay of completed timelines
//! - `streaming`: Hooks for incremental dialogue output

pub mod builder;
pub mod narrator;
pub mod redaction;
pub mod replay;
pub mod streaming;
pub mod types;

// Re-export key types
pub use builder::TimelineBuilder;
pub use narrator::{format_timeline, narrate_timeline, DialogueLine};
pub use redaction::{redact_text, redact_timeline, RedactionMode};
pub use replay::{verify_deterministic_replay, ReplayFingerprint, ReplaySession};
pub use streaming::{DialogueConsumer, DialogueStream, SharedDialogueStream, StreamEvent};
pub use types::{ActionType, DialogueTimeline, EntryKind, TimelineEntry};
