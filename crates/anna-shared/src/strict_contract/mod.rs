//! Strict Specialist Contract (v0.0.415).
//!
//! THE single source of truth for specialist JSON responses.
//! All specialists MUST return exactly this schema. No exceptions.
//!
//! Design principles:
//! - JSON only, no prose outside JSON
//! - Every field has a clear purpose
//! - Validation catches all known failure modes
//! - Metrics are domain-specific and structured
//! - Citations are mandatory for grounded answers

mod budgets;
mod parser;
mod types;
mod validation;

#[cfg(test)]
mod tests;

// Re-export all public items
pub use budgets::TimeBudgets;
pub use parser::{extract_json, parse_lenient, parse_specialist_output};
pub use types::{
    ActionKind, Citation, CitationKind, EvidenceItem, ParseResult, RiskLevel, StrictSpecialistResponse,
    StrictStatus, SuggestedAction,
};
