// v0.0.585: Settings Logging (Phase 161)
// Structured logging for settings operations
// Modularized for maintainability

mod log_level;
mod log_target;
mod log_entry;
mod log_filter;
mod logger;
mod utils;
#[cfg(test)]
mod tests;

// Re-export all public types and functions to preserve API
pub use log_level::LogLevel;
pub use log_target::LogTarget;
pub use log_entry::LogEntry;
pub use log_filter::LogFilter;
pub use logger::SettingsLogger;
pub use utils::{format_logs, is_logging_query, settings_logging_fun_fact};
