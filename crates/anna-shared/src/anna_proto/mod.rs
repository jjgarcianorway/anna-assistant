//! Anna Protocol v1 - Robust Model Communication (v0.0.436).
//!
//! Provides unbreakable typed JSON communication between Anna and models.
//! Key principles:
//! - Parsing never times out - only model calls can timeout
//! - Non-JSON or partial output is recoverable
//! - Every call returns valid payload or clean failure
//! - Stats only count success when valid payload exists

pub mod decoder;
pub mod envelope;
mod envelope_actions;
mod envelope_claims;
mod envelope_errors;
mod envelope_types;
pub mod fallback;
mod fallback_builder;
mod fallback_types;
pub mod framing;
pub mod prompts;
pub mod stats;
pub mod streaming;
pub mod tests;

pub use decoder::{DecodeError, DecodeResult, ProtoDecoder};
pub use envelope::{
    Action, ActionPayload, ActionType, Claim, EnvelopeValidation, ErrorCode, EvidenceKind,
    EvidenceRef, ModelError, ModelResultEnvelope, ModelRole, RiskLevel,
};
pub use fallback::{
    EvidenceFallback, FallbackResponse, GatheredEvidence, MAX_FALLBACK_CONFIDENCE,
};
pub use framing::{extract_framed_content, PROTO_END, PROTO_START, PROTO_VERSION};
pub use prompts::{junior_prompt, protocol_suffix, senior_prompt, translator_prompt};
pub use stats::{outcome_from_decode, PeriodStats, StatsSummary, TicketOutcome};
pub use streaming::{ProgressFrame, ProgressType, StreamBuffer, StreamDisplay, StreamState};

/// Protocol version.
pub const VERSION: &str = "1";

/// Default timeouts in milliseconds.
pub const DEFAULT_TRANSLATOR_TIMEOUT_MS: u64 = 8_000;
pub const DEFAULT_JUNIOR_TIMEOUT_MS: u64 = 12_000;
pub const DEFAULT_SENIOR_TIMEOUT_MS: u64 = 20_000;

/// Maximum JSON repair attempts.
pub const MAX_REPAIR_ATTEMPTS: usize = 2;
