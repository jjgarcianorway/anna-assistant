//! Relevant Health View for "how is my computer" queries (v0.0.210).
//!
//! Produces minimal, actionable health summaries - NOT full reports.
//! Only shows critical issues and warnings. Silent when healthy.
//!
//! v0.0.210: Modularized into domain-focused submodules.

mod builder;
mod summary;
mod tests;
mod types;

// Re-export all types and functions
pub use builder::{build_health_summary, has_health_issues};
pub use summary::RelevantHealthSummary;
pub use types::{HealthCategory, HealthChange, HealthItem, HealthSeverity};
