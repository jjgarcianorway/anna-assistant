//! Deterministic system report generation (v0.0.189).
//!
//! All output derived from typed evidence atoms only.
//! No LLM speculation - purely threshold-driven health checks.
//!
//! v0.0.189: Modularized into domain-focused submodules.

mod builders;
mod formatters;
mod helpers;
mod tests;
mod types;

// Re-export main types and functions
pub use builders::{build_executive_summary, build_health_checks, build_inventory};
pub use formatters::{format_markdown, format_text};
pub use helpers::{format_bytes, sanitize_mount};
pub use types::{HealthItem, HealthSeverity, ReportEvidence, SystemInventory, SystemReport};
