//! EvidenceBundle (Part B) - v0.0.441.
//!
//! Core data model for structured evidence.
//!
//! All probes output into this structure:
//! - facts: Atomic, namespaced, typed values
//! - raw: Original probe output
//! - confidence: Per-domain confidence scores
//! - missing: Facts that could not be collected

mod bundle;
mod builder;
mod extractors;
mod fact_value;

// Re-export all public types
pub use bundle::{fact_domain, EvidenceBundle, ProbeError};
pub use builder::EvidenceBundleBuilder;
pub use extractors::{
    extract_blame, extract_boot_time, extract_disk, extract_failed_services, extract_memory,
};
pub use fact_value::FactValue;
