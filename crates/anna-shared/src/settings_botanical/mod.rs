// v0.0.773: Settings Botanical Module (Phase 349)
// Botanical garden for settings plant science

mod types;
mod config;
mod collection;
mod stats;
mod botanical;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types to preserve the original API
pub use types::{BotanicalType, BotanicalStatus};
pub use config::BotanicalConfig;
pub use collection::{BotanicalCollection, BotanicalBotanist};
pub use stats::BotanicalStats;
pub use botanical::SettingsBotanical;
pub use registry::{BotanicalRegistry, format_botanical_registry};
pub use utils::{is_botanical_query, botanical_fun_fact};
