// v0.0.771: Settings Conservatory Module
// Glass conservatory for settings preservation

mod types;
mod config;
mod specimen;
mod curator;
mod stats;
mod conservatory;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types to preserve the original API
pub use types::{ConservatoryType, ConservatoryStatus};
pub use config::ConservatoryConfig;
pub use specimen::ConservatorySpecimen;
pub use curator::ConservatoryCurator;
pub use stats::ConservatoryStats;
pub use conservatory::SettingsConservatory;
pub use registry::ConservatoryRegistry;
pub use utils::{format_conservatory_registry, is_conservatory_query, conservatory_fun_fact};
