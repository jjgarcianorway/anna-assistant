// v0.0.559: Settings CLI Interface - Parser (Phase 135)
// Natural language command parser for settings commands

use crate::unified_settings::SettingsCategory;

use super::command::SettingsCommand;
use super::parse_result::ParseResult;

/// Settings command parser
#[derive(Debug, Clone, Default)]
pub struct SettingsParser {
    /// Strict mode (require exact matches)
    pub strict: bool,
}

impl SettingsParser {
    /// Create new parser
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable strict mode
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    /// Parse a natural language command
    pub fn parse(&self, input: &str) -> ParseResult {
        let lower = input.to_lowercase().trim().to_string();

        // Check for show commands
        if let Some(result) = self.parse_show(&lower) {
            return result;
        }

        // Check for reset commands
        if let Some(result) = self.parse_reset(&lower) {
            return result;
        }

        // Check for export commands
        if let Some(result) = self.parse_export(&lower) {
            return result;
        }

        // Check for import commands
        if let Some(result) = self.parse_import(&lower) {
            return result;
        }

        // Check for validate commands
        if lower.contains("validate") || lower.contains("check") && lower.contains("setting") {
            return ParseResult::new(SettingsCommand::Validate, 0.9);
        }

        // Check for help commands
        if lower.contains("help") || lower == "?" {
            return ParseResult::new(SettingsCommand::Help, 1.0);
        }

        // Check for list categories
        if lower.contains("list") && lower.contains("categor") {
            return ParseResult::new(SettingsCommand::ListCategories, 0.9);
        }

        // Otherwise, treat as a change command
        if self.looks_like_change(&lower) {
            return ParseResult::new(SettingsCommand::Change(input.to_string()), 0.7);
        }

        ParseResult::new(SettingsCommand::Unknown(input.to_string()), 0.3)
    }

    /// Parse show commands
    fn parse_show(&self, lower: &str) -> Option<ParseResult> {
        if !lower.contains("show") && !lower.contains("display") && !lower.contains("list") {
            return None;
        }

        // Check for specific category
        let category = self.extract_category(lower);

        Some(ParseResult::new(SettingsCommand::Show(category), 0.9))
    }

    /// Parse reset commands
    fn parse_reset(&self, lower: &str) -> Option<ParseResult> {
        if !lower.contains("reset") && !lower.contains("restore default") {
            return None;
        }

        let category = self.extract_category(lower);

        Some(ParseResult::new(SettingsCommand::Reset(category), 0.9))
    }

    /// Parse export commands
    fn parse_export(&self, lower: &str) -> Option<ParseResult> {
        if !lower.contains("export") && !lower.contains("save to") {
            return None;
        }

        // Try to extract path
        let path = self.extract_path(lower);

        Some(ParseResult::new(SettingsCommand::Export(path), 0.9))
    }

    /// Parse import commands
    fn parse_import(&self, lower: &str) -> Option<ParseResult> {
        if !lower.contains("import") && !lower.contains("load from") {
            return None;
        }

        // Try to extract path
        if let Some(path) = self.extract_path(lower) {
            return Some(ParseResult::new(SettingsCommand::Import(path), 0.9));
        }

        Some(ParseResult::new(
            SettingsCommand::Unknown("Import requires a path".to_string()),
            0.5,
        ))
    }

    /// Extract category from input
    fn extract_category(&self, lower: &str) -> Option<SettingsCategory> {
        if lower.contains("personality") {
            Some(SettingsCategory::Personality)
        } else if lower.contains("risk") {
            Some(SettingsCategory::Risk)
        } else if lower.contains("learning") {
            Some(SettingsCategory::Learning)
        } else if lower.contains("escalation") {
            Some(SettingsCategory::Escalation)
        } else if lower.contains("verbosity") || lower.contains("verbose") {
            Some(SettingsCategory::Verbosity)
        } else if lower.contains("confirmation") || lower.contains("confirm") {
            Some(SettingsCategory::Confirmation)
        } else if lower.contains("timeout") {
            Some(SettingsCategory::Timeout)
        } else if lower.contains("style") || lower.contains("output") {
            Some(SettingsCategory::OutputStyle)
        } else if lower.contains("privacy") {
            Some(SettingsCategory::Privacy)
        } else if lower.contains("backup") {
            Some(SettingsCategory::Backup)
        } else if lower.contains("update") {
            Some(SettingsCategory::Update)
        } else if lower.contains("model") {
            Some(SettingsCategory::Model)
        } else {
            None
        }
    }

    /// Extract path from input
    fn extract_path(&self, lower: &str) -> Option<String> {
        // Look for quoted path
        if let Some(start) = lower.find('"') {
            if let Some(end) = lower[start + 1..].find('"') {
                return Some(lower[start + 1..start + 1 + end].to_string());
            }
        }

        // Look for path-like strings
        for word in lower.split_whitespace() {
            if word.contains('/') || word.contains('.') && !word.starts_with('.') {
                return Some(word.to_string());
            }
        }

        None
    }

    /// Check if input looks like a change command
    fn looks_like_change(&self, lower: &str) -> bool {
        lower.contains("set ")
            || lower.contains("make ")
            || lower.contains("enable ")
            || lower.contains("disable ")
            || lower.contains("turn on")
            || lower.contains("turn off")
            || lower.contains("be more")
            || lower.contains("be less")
            || lower.contains("increase")
            || lower.contains("decrease")
    }
}
