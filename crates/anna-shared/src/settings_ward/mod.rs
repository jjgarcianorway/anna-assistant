// v0.0.752: Settings Ward Module (Phase 328)
// Electoral ward for settings representation

mod types;
mod config;
mod motion;
mod stats;
mod ward;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public types
pub use types::{WardType, WardStatus};
pub use config::WardConfig;
pub use motion::{WardMotion, WardDelegate};
pub use stats::WardStats;
pub use ward::SettingsWard;
pub use registry::WardRegistry;
pub use utils::{format_ward_registry, is_ward_query, ward_fun_fact};
