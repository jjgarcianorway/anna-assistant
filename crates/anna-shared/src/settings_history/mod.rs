// v0.0.563: Settings History (Phase 139)
// Tracks changes to settings over time with undo/redo support

mod types;
mod manager;
mod formatting;

#[cfg(test)]
mod tests;

// Re-export public types and functions
pub use types::HistoryEntry;
pub use manager::SettingsHistory;
pub use formatting::{format_history, settings_history_fun_fact};
