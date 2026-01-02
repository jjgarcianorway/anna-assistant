// v0.0.705: Settings Almanac (Phase 281)
// Yearly almanac of settings information

mod types;
mod config;
mod chapter;
mod stats;
mod almanac;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public API
pub use types::{AlmanacType, AlmanacEdition};
pub use config::AlmanacConfig;
pub use chapter::{AlmanacChapter, AlmanacEntry};
pub use stats::AlmanacStats;
pub use almanac::SettingsAlmanac;
pub use registry::AlmanacRegistry;
pub use utils::{format_almanac_registry, is_almanac_query, almanac_fun_fact};
