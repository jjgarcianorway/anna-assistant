// v0.0.735: Settings Alliance (Phase 311)
// Formal alliance for settings governance

mod types;
mod config;
mod commitment;
mod member;
mod stats;
mod alliance;
mod registry;

#[cfg(test)]
mod tests;

// Re-export all public types to preserve the original API
pub use types::{AllianceType, AllianceStatus};
pub use config::AllianceConfig;
pub use commitment::AllianceCommitment;
pub use member::AllianceMember;
pub use stats::AllianceStats;
pub use alliance::SettingsAlliance;
pub use registry::{AllianceRegistry, format_alliance_registry, is_alliance_query, alliance_fun_fact};
