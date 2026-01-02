// v0.0.736: Settings Coalition (Phase 312)
// Temporary coalition for settings governance

mod types;
mod config;
mod agreement;
mod partner;
mod stats;
mod coalition;
mod registry;
mod helpers;

// Re-export all public types and functions
pub use types::{CoalitionType, CoalitionStatus};
pub use config::CoalitionConfig;
pub use agreement::CoalitionAgreement;
pub use partner::CoalitionPartner;
pub use stats::CoalitionStats;
pub use coalition::SettingsCoalition;
pub use registry::CoalitionRegistry;
pub use helpers::{format_coalition_registry, is_coalition_query, coalition_fun_fact};
