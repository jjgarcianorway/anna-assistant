//! Research Plan and Citations - v0.0.443.
//!
//! Research plan generated BEFORE specialist:
//! - Required facts
//! - Probes to run
//! - Sources to fetch (man, help, wiki)
//!
//! Citations included in every answer:
//! - Sources (documentation)
//! - Evidence (from this machine)

// Re-export types from submodules
pub use super::research_builder::CitationBuilder;
pub use super::research_types::{
    CitedAnswer, Citation, ResearchConstraints, ResearchPlan, ResearchResult,
};

/// Truncate string to max length.
pub fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}
