// v0.0.772: Settings Arboretum Module (Phase 348)
// Tree arboretum for settings dendrology

mod types;
mod config;
mod specimen;
mod stats;
mod arboretum;
mod registry;

#[cfg(test)]
mod tests;

// Re-export all public types to preserve the original API
pub use types::{ArboretumType, ArboretumStatus};
pub use config::ArboretumConfig;
pub use specimen::{ArboretumSpecimen, ArboretumDendrologist};
pub use stats::ArboretumStats;
pub use arboretum::SettingsArboretum;
pub use registry::{ArboretumRegistry, format_arboretum_registry, is_arboretum_query, arboretum_fun_fact};
