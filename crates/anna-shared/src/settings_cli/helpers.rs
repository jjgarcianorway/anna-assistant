// v0.0.559: Settings CLI Helpers (Phase 135)
// Utility functions for settings command parsing

use super::parse_result::ParseResult;
use super::parser::SettingsParser;

/// Quick parse helper
pub fn parse_settings_command(input: &str) -> ParseResult {
    SettingsParser::new().parse(input)
}

/// Check if input is a settings command
pub fn is_settings_command(input: &str) -> bool {
    let lower = input.to_lowercase();
    lower.contains("setting")
        || lower.contains("config")
        || lower.contains("preference")
        || lower.starts_with("show ")
        || lower.starts_with("reset ")
        || lower.starts_with("export ")
        || lower.starts_with("import ")
}

/// Fun fact about settings CLI
pub fn settings_cli_fun_fact() -> &'static str {
    "You can change any of Anna's 12 settings categories using natural language - no commands to memorize!"
}
