// v0.0.700: Settings Compendium (Phase 276) - Milestone!
// Comprehensive compendium of settings knowledge

mod types;
mod config;
mod volume;
mod entry;
mod stats;
mod compendium;
mod registry;
mod helpers;

#[cfg(test)]
mod tests;

// Re-export all public types to maintain API compatibility
pub use types::{CompendiumType, CompendiumEdition};
pub use config::CompendiumConfig;
pub use volume::{CompendiumVolume, CompendiumArticle};
pub use entry::CompendiumEntry;
pub use stats::CompendiumStats;
pub use compendium::SettingsCompendium;
pub use registry::CompendiumRegistry;
pub use helpers::{format_compendium_registry, is_compendium_query, compendium_fun_fact};
