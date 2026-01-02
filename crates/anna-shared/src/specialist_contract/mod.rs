//! Specialist JSON contract (v0.0.419).
//!
//! This defines the STRICT schema that all specialists must output.
//! Specialists ONLY output JSON - no prose, no roleplay, no excuses.
//! The personality layer (Sofia, Tomas, etc.) is handled by the renderer.
//!
//! Key principles:
//! - answer.short MUST directly answer the user's question
//! - evidence[] MUST back up every claim
//! - evidence_references[] MUST list IDs of knowledge items used
//! - citations[] MUST provide provenance for all knowledge used
//! - can_answer MUST be false if insufficient evidence
//! - discovery.new_probes/recipes is how Anna learns new capabilities
//! - Specialists NEVER speak to the user, only return structured data
//!
//! v0.0.419: Added KnowledgeCitation for provenance tracking

mod citation;
mod discovery;
mod response;
mod types;

#[cfg(test)]
mod tests;

// Re-export all public items
pub use citation::{CitationKind, KnowledgeCitation};
pub use discovery::{Discovery, ProbeProposal, RecipeProposal, RiskLevel};
pub use response::SpecialistResponse;
pub use types::{
    Answer, Evidence, InternalAction, Mood, NextSteps, ResponseStatus, Severity,
    SpecialistInput, SpecialistIntent, StaffView, UserAction,
};
