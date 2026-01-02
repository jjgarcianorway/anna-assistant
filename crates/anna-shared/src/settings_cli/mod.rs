// v0.0.559: Settings CLI Interface (Phase 135)
// Provides natural language command parsing for settings management

mod command;
mod executor;
mod helpers;
mod parse_result;
mod parser;

#[cfg(test)]
mod tests;

// Re-export main types
pub use command::SettingsCommand;
pub use executor::execute_command;
pub use helpers::{is_settings_command, parse_settings_command, settings_cli_fun_fact};
pub use parse_result::ParseResult;
pub use parser::SettingsParser;
