// v0.0.530: Knowledge Citation Tracker (Phase 106)
// Tracks citations from authoritative sources (Arch Wiki, man pages, --help) per VISION.md
//
// This module is organized into the following submodules:
// - types: CitationSource and CitationReliability enums
// - record: CitationRecord struct and methods
// - tracker: KnowledgeCitationTracker main implementation
// - formatting: Display and formatting functions
// - utils: Utility functions for queries and fun facts

mod formatting;
mod record;
mod tracker;
mod types;
mod utils;

// Re-export all public types and functions to preserve the original API
pub use formatting::{
    format_citation, format_citation_compact, format_citation_oneline, format_tracker_summary,
};
pub use record::CitationRecord;
pub use tracker::KnowledgeCitationTracker;
pub use types::{CitationReliability, CitationSource};
pub use utils::{citation_fun_fact, is_citation_query};
