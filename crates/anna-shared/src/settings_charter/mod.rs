// v0.0.724: Settings Charter (Phase 300)
// Foundational charter for settings governance

mod types;
mod config;
mod provision;
mod stats;
mod charter;
mod registry;

#[cfg(test)]
mod tests;

// Re-export all public types to preserve the original API
pub use types::{CharterType, CharterStatus};
pub use config::CharterConfig;
pub use provision::{CharterProvision, CharterAmendment};
pub use stats::CharterStats;
pub use charter::SettingsCharter;
pub use registry::{CharterRegistry, format_charter_registry, is_charter_query, charter_fun_fact};
