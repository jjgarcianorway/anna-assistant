// v0.0.727: Settings Treaty (Phase 303)
// International agreement for settings governance

mod helpers;
mod registry;
#[cfg(test)]
mod tests;
mod treaty;
mod types;

// Re-export all public types and functions to maintain the public API
pub use helpers::{format_treaty_registry, is_treaty_query, treaty_fun_fact};
pub use registry::TreatyRegistry;
pub use treaty::SettingsTreaty;
pub use types::{
    TreatyConfig, TreatyProvision, TreatySignatory, TreatyStats, TreatyStatus, TreatyType,
};
