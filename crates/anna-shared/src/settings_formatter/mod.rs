// v0.0.644: Settings Formatter Module (Phase 220)
// Formatter for converting settings values to display formats

pub mod formatter;
pub mod registry;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export all public items to maintain the original API
pub use formatter::SettingsFormatter;
pub use registry::{
    format_formatter_registry, formatter_fun_fact, is_formatter_query, SettingsFormatterRegistry,
};
pub use types::{FormatResult, FormatStyle, FormatType, FormatterConfig, FormatterStats};
