// v0.0.778: Settings Aviary (Phase 354)
// Bird aviary for settings ornithology

mod types;
mod config;
mod bird;
mod keeper;
mod stats;
mod aviary;
mod registry;

#[cfg(test)]
mod tests;

// Re-export all public types and functions
pub use types::{AviaryType, AviaryStatus};
pub use config::AviaryConfig;
pub use bird::AviaryBird;
pub use keeper::AviaryKeeper;
pub use stats::AviaryStats;
pub use aviary::SettingsAviary;
pub use registry::{AviaryRegistry, format_aviary_registry, is_aviary_query, aviary_fun_fact};
