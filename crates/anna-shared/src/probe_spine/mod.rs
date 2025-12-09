//! Probe spine: deterministic tool selection and evidence requirements (v0.0.193).
//!
//! Prevents "no probes, no evidence, but claims anyway" scenarios.
//!
//! v0.0.193: Modularized into domain-focused submodules.

mod commands;
mod enforcement;
mod reduction;
mod types;

// Re-export all types and functions
pub use commands::{probe_to_command, probes_for_evidence};
pub use enforcement::{enforce_minimum_probes, enforce_spine_probes, ProbeSpineDecision};
pub use reduction::{query_wants_errors, query_wants_warnings, reduce_probes};
pub use types::{EvidenceKind, ProbeId, RouteCapability, Urgency};
