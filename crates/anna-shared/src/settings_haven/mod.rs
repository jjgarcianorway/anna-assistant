// v0.0.784: Settings Haven (Phase 360)
// Safe haven for settings protection - Module definitions

mod types;
mod config;
mod guest;
mod keeper;
mod stats;
mod haven;
mod registry;
mod utils;
#[cfg(test)]
mod tests;

// Re-export public API
pub use types::{HavenType, HavenStatus};
pub use config::HavenConfig;
pub use guest::HavenGuest;
pub use keeper::HavenKeeper;
pub use stats::HavenStats;
pub use haven::SettingsHaven;
pub use registry::{HavenRegistry, format_haven_registry};
pub use utils::{is_haven_query, haven_fun_fact};
