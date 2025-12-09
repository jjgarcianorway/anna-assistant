//! Execution trace for auditable request processing (v0.0.184).
//!
//! Provides structured, deterministic trace of which stages ran,
//! which failed, and which path produced the final answer.
//! No timestamps - only enums and counts for reproducibility.
//!
//! v0.0.184: Modularized into domain-focused submodules.

mod evidence;
mod execution;
mod outcomes;
mod probe_stats;
mod tests;

// Re-export main types
pub use evidence::{evidence_kinds_from_probes, evidence_kinds_from_route, EvidenceKind};
pub use execution::ExecutionTrace;
pub use outcomes::{FallbackUsed, ReviewerOutcome, SpecialistOutcome};
pub use probe_stats::ProbeStats;
