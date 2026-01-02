// v0.0.588: Settings Versioning (Phase 164)
// Version control for settings with history and comparison

mod types;
mod history;
mod formatting;

// Re-export all public types and functions to preserve the API
pub use types::{ChangeType, VersionChange, SettingsVersion};
pub use history::VersionHistory;
pub use formatting::{
    format_version,
    format_history,
    is_versioning_query,
    settings_versioning_fun_fact,
};
