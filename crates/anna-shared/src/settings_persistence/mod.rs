// v0.0.555: Settings Persistence (Phase 131)
// Handles saving/loading unified settings to/from disk

mod backup;
mod error;
mod format;
mod manager;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public API to preserve existing interface
pub use error::{SettingsError, SettingsResult};
pub use format::SettingsFormat;
pub use manager::SettingsPersistence;
pub use utils::{format_persistence_status, is_persistence_available, settings_persistence_fun_fact};
