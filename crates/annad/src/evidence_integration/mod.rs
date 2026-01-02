//! Evidence Integration - Bridge between evidence pipeline and specialist (v0.0.410).
//!
//! This module integrates the evidence engine into the specialist pipeline:
//! 1. Converts translator output to evidence request
//! 2. Runs evidence pipeline (probes, docs, knowledge)
//! 3. Formats evidence bundle for specialist consumption
//!
//! The goal: Specialist sees structured evidence, not raw chaos.

mod converters;
mod formatting;
mod integration;
mod probes;
mod tags;
mod types;

// Re-export public API
pub use formatting::{
    build_enhanced_specialist_input, evidence_to_probe_map, format_evidence_for_prompt,
};
pub use integration::build_evidence_for_specialist;
pub use types::EvidenceIntegrationResult;
