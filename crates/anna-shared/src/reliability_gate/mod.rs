//! Hard Reliability Gate (v0.0.447).
//!
//! CORE PRINCIPLE: Anna NEVER answers without evidence.
//!
//! Gate checks (ALL must pass):
//! 1. PROBE COVERAGE - Every claim backed by probe evidence
//! 2. QUESTION MATCH - Answer directly matches user's question
//! 3. DOMAIN CONSISTENCY - Domain matches probes and answer
//! 4. NO HALLUCINATED ENTITIES - Every noun exists in probe output
//! 5. PARSE SUCCESS - LLM output parsed deterministically
//! 6. CONFIDENCE THRESHOLD - Score >= 0.85
//!
//! If ANY check fails → abort answer → produce controlled failure response.

pub mod answer_contract;
pub mod claim_evidence;
pub mod deterministic;
pub mod gate;

#[cfg(test)]
pub mod tests;

// Re-exports
pub use answer_contract::{detect_generic_content, AnswerContract, AnswerShape, ContractViolation};
pub use claim_evidence::{ClaimType, EvidenceBinding, EvidenceType, StrictClaim, StrictEvidence};
pub use deterministic::{DeterministicPolicy, DeterministicRoute, QueryDomain};
pub use gate::{GateCheck, GateInput, GateOutcome, GateResult, ReliabilityGate};

/// Module version
pub const RELIABILITY_GATE_VERSION: &str = "0.0.447";
