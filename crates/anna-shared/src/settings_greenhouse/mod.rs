// v0.0.770: Settings Greenhouse (Phase 346)
// Controlled greenhouse for settings cultivation

mod types;
mod config;
mod crop;
mod stats;
mod greenhouse;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types and functions to preserve the original API
pub use types::{GreenhouseType, GreenhouseStatus};
pub use config::GreenhouseConfig;
pub use crop::{GreenhouseCrop, GreenhouseGrower};
pub use stats::GreenhouseStats;
pub use greenhouse::SettingsGreenhouse;
pub use registry::GreenhouseRegistry;
pub use utils::{format_greenhouse_registry, is_greenhouse_query, greenhouse_fun_fact};
