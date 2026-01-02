// v0.0.586: Settings Cache Module (Phase 162)
// Modular structure for settings cache functionality

mod types;
mod cache;
mod helpers;

#[cfg(test)]
mod tests;

// Re-export public API
pub use types::{CacheEntry, CacheState, CacheStats, EvictionPolicy};
pub use cache::SettingsCache;
pub use helpers::{format_cache, is_cache_query, settings_cache_fun_fact};
