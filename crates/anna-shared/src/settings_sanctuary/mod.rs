// v0.0.781: Settings Sanctuary (Phase 357)
// Wildlife sanctuary for settings conservation

mod types;
mod config;
mod resident;
mod warden;
mod stats;
mod sanctuary;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use types::{SanctuaryType, SanctuaryStatus};
pub use config::SanctuaryConfig;
pub use resident::SanctuaryResident;
pub use warden::SanctuaryWarden;
pub use stats::SanctuaryStats;
pub use sanctuary::SettingsSanctuary;
pub use registry::SanctuaryRegistry;
pub use utils::{format_sanctuary_registry, is_sanctuary_query, sanctuary_fun_fact};
