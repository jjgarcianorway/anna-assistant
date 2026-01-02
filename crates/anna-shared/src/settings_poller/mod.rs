// v0.0.637: Settings Poller Module (Phase 213)
// Poller for settings changes with interval support

mod types;
mod watcher;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types to preserve the original API
pub use types::{WatcherType, WatchInterval, WatcherConfig, WatcherStats};
pub use watcher::{WatchEvent, Watcher};
pub use registry::SettingsWatcherRegistry;
pub use utils::{format_watcher_registry, is_watcher_query, watcher_fun_fact};
