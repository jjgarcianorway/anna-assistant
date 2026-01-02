// v0.0.716: Settings Missive Module (Phase 292)
// Formal letters about settings changes

mod types;
mod config;
mod letter;
mod enclosure;
mod stats;
mod settings;
mod registry;
mod helpers;

#[cfg(test)]
mod tests;

// Re-export all public types and functions
pub use types::{MissiveType, MissiveDelivery};
pub use config::MissiveConfig;
pub use letter::MissiveLetter;
pub use enclosure::MissiveEnclosure;
pub use stats::MissiveStats;
pub use settings::SettingsMissive;
pub use registry::MissiveRegistry;
pub use helpers::{format_missive_registry, is_missive_query, missive_fun_fact};
