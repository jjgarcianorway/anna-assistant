//! Recipe extractor for Anna's learning system.
//! v0.0.418: Extracts recipes from successful tickets.
//!
//! Given an eligible ticket, extracts:
//! - Intent and parameters (pattern)
//! - Plan steps from specialist actions
//! - Preconditions from probes used
//! - Matcher from user query keywords
//! - Citations from knowledge engine

mod extractor;
mod id_generator;
mod matcher;
mod plan;
mod preconditions;
mod types;

// Re-export public types
pub use types::{
    CommandRecord, ExtractionResult, FileEdit, FileEditType, TicketData,
};

// Re-export main extraction function
pub use extractor::extract_recipe;
