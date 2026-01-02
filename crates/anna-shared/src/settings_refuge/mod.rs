// v0.0.783: Settings Refuge (Phase 359)
// Wildlife refuge for settings shelter

mod types;
mod config;
mod inhabitant;
mod stats;
mod refuge;
mod registry;
mod utils;

// Re-export all public types and functions
pub use types::{RefugeType, RefugeStatus};
pub use config::RefugeConfig;
pub use inhabitant::{RefugeInhabitant, RefugeWarden};
pub use stats::RefugeStats;
pub use refuge::SettingsRefuge;
pub use registry::RefugeRegistry;
pub use utils::{format_refuge_registry, is_refuge_query, refuge_fun_fact};
