//! Anna Protocol v1 - Robust Model Communication (v0.0.436).
//!
//! Provides unbreakable typed JSON communication between Anna and models.
//! Key principles:
//! - Parsing never times out - only model calls can timeout
//! - Non-JSON or partial output is recoverable
//! - Every call returns valid payload or clean failure
//! - Stats only count success when valid payload exists

pub mod framing;
pub mod decoder;
pub mod envelope;
pub mod fallback;
pub mod stats;
pub mod prompts;
pub mod streaming;
pub mod tests;

pub use framing::{PROTO_START, PROTO_END, PROTO_VERSION, extract_framed_content};
pub use decoder::{ProtoDecoder, DecodeResult, DecodeError};
pub use envelope::{
    ModelResultEnvelope, Claim, Action, ActionType, ActionPayload,
    EvidenceRef, EvidenceKind, ModelError, ErrorCode, ModelRole,
};
pub use fallback::{EvidenceFallback, FallbackResponse, GatheredEvidence};
pub use prompts::{protocol_suffix, junior_prompt, senior_prompt, translator_prompt};
pub use stats::{TicketOutcome, PeriodStats, StatsSummary, outcome_from_decode};
pub use streaming::{StreamBuffer, StreamState, ProgressFrame};

/// Protocol version.
pub const VERSION: &str = "1";

/// Default timeouts in milliseconds.
pub const DEFAULT_TRANSLATOR_TIMEOUT_MS: u64 = 8_000;
pub const DEFAULT_JUNIOR_TIMEOUT_MS: u64 = 12_000;
pub const DEFAULT_SENIOR_TIMEOUT_MS: u64 = 20_000;

/// Maximum JSON repair attempts.
pub const MAX_REPAIR_ATTEMPTS: usize = 2;
