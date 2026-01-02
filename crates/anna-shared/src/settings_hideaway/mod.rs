// v0.0.786: Settings Hideaway (Phase 362)
// Secret hideaway for settings seclusion

mod types;
mod models;
mod hideaway;
mod utils;

#[cfg(test)]
mod tests;

// Re-export types
pub use types::{HideawayType, HideawayStatus};

// Re-export models
pub use models::{HideawayConfig, HideawayOccupant, HideawayGuardian, HideawayStats};

// Re-export hideaway system
pub use hideaway::{SettingsHideaway, HideawayRegistry};

// Re-export utilities
pub use utils::{format_hideaway_registry, is_hideaway_query, hideaway_fun_fact};
