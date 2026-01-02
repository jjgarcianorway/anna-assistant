//! Evidence cache for learning engine (v0.0.427).
//!
//! Rolling cache of:
//! - Probe outputs
//! - Documentation links (Arch Wiki, man pages)
//! - Prior tickets with similar patterns
//!
//! Not directly used to answer users - used as context
//! when generating or refining recipes.

mod cache;
mod evidence_types;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public types
pub use cache::EvidenceCache;
pub use evidence_types::{EvidenceEntry, EvidenceType};
