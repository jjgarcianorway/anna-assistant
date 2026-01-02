// v0.0.720: Settings Decree (Phase 296)
// Official decrees for settings governance

// Module declarations
mod types;
mod config;
mod ruling;
mod stats;
mod decree;
mod registry;
mod helpers;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use types::{DecreeType, DecreeBinding};
pub use config::DecreeConfig;
pub use ruling::{DecreeRuling, DecreeClause};
pub use stats::DecreeStats;
pub use decree::SettingsDecree;
pub use registry::DecreeRegistry;
pub use helpers::{format_decree_registry, is_decree_query, decree_fun_fact};
