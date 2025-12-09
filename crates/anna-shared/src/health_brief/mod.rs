//! Health Brief - Relevant-only health status for "how is my computer" queries (v0.0.207).
//!
//! Unlike full system reports, HealthBrief only shows actionable items:
//! - Disk warnings (>85%)
//! - Memory pressure (>90%)
//! - Failed/degraded services
//! - High CPU consumers
//!
//! If everything is OK, the brief simply says "all good".
//!
//! v0.0.207: Modularized into domain-focused submodules.

mod brief;
mod severity;
mod tests;
mod types;

// Re-export all types and functions
pub use brief::HealthBrief;
pub use severity::{disk_severity, memory_severity};
pub use types::{BriefItem, BriefItemKind, BriefSeverity};
