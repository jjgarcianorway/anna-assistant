//! Evidence Gating (Part C) - v0.0.439.
//!
//! Rules:
//! - If required_evidence probes are missing or failed: do not call specialist.
//! - If probes already provide a direct factual answer: do not call specialist.
//! - Only call specialists when synthesis is needed (why, root cause, recommendations).

mod extractors;
mod gate;
mod types;

// Re-export all public types
pub use gate::EvidenceGate;
pub use types::{DirectAnswer, EvidenceStatus, GateDecision, ProbeResult};
