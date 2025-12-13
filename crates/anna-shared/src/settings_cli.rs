// v0.0.559: Settings CLI Interface (Phase 135)
// Provides natural language command parsing for settings management

use serde::{Deserialize, Serialize};

use crate::unified_settings::{SettingsCategory, UnifiedSettings};

/// Settings command type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingsCommand {
    /// Show current settings
    Show(Option<SettingsCategory>),
    /// Change a setting
    Change(String),
    /// Reset settings
    Reset(Option<SettingsCategory>),
    /// Export settings
    Export(Option<String>),
    /// Import settings
    Import(String),
    /// Validate settings
    Validate,
    /// Show help
    Help,
    /// List categories
    ListCategories,
    /// Unknown command
    Unknown(String),
}

impl std::fmt::Display for SettingsCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Show(Some(cat)) => write!(f, "Show {} settings", cat),
            Self::Show(None) => write!(f, "Show all settings"),
            Self::Change(request) => write!(f, "Change: {}", request),
            Self::Reset(Some(cat)) => write!(f, "Reset {} settings", cat),
            Self::Reset(None) => write!(f, "Reset all settings"),
            Self::Export(Some(path)) => write!(f, "Export to {}", path),
            Self::Export(None) => write!(f, "Export settings"),
            Self::Import(path) => write!(f, "Import from {}", path),
            Self::Validate => write!(f, "Validate settings"),
            Self::Help => write!(f, "Show help"),
            Self::ListCategories => write!(f, "List categories"),
            Self::Unknown(cmd) => write!(f, "Unknown: {}", cmd),
        }
    }
}

/// Command parse result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    /// Parsed command
    pub command: SettingsCommand,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Alternative interpretations
    pub alternatives: Vec<SettingsCommand>,
}

impl ParseResult {
    /// Create a new parse result
    pub fn new(command: SettingsCommand, confidence: f32) -> Self {
        Self {
            command,
            confidence,
            alternatives: vec![],
        }
    }

    /// Add an alternative interpretation
    pub fn with_alternative(mut self, alt: SettingsCommand) -> Self {
        self.alternatives.push(alt);
        self
    }

    /// Is this a confident match?
    pub fn is_confident(&self) -> bool {
        self.confidence >= 0.7
    }
}

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

/// Execute a settings command
pub fn execute_command(command: &SettingsCommand, settings: &mut UnifiedSettings) -> String {
    match command {
        SettingsCommand::Show(Some(cat)) => {
            format!("Showing {} settings:\n{}", cat, format_category(settings, *cat))
        }
        SettingsCommand::Show(None) => {
            crate::unified_settings::format_settings_summary(settings)
        }
        SettingsCommand::Change(request) => {
            if let Some(response) = settings.apply_change(request) {
                format!("Settings updated: {}", response)
            } else {
                "Could not apply the requested change.".to_string()
            }
        }
        SettingsCommand::Reset(Some(cat)) => {
            settings.reset_category(*cat);
            format!("{} settings have been reset to defaults.", cat)
        }
        SettingsCommand::Reset(None) => {
            settings.reset_all();
            "All settings have been reset to defaults.".to_string()
        }
        SettingsCommand::Export(path) => {
            let filename = path.clone().unwrap_or_else(|| "settings.json".to_string());
            format!("Settings would be exported to: {}", filename)
        }
        SettingsCommand::Import(path) => {
            format!("Settings would be imported from: {}", path)
        }
        SettingsCommand::Validate => {
            let result = crate::settings_validation::validate_settings(settings);
            crate::settings_validation::format_validation_result(&result)
        }
        SettingsCommand::Help => format_help(),
        SettingsCommand::ListCategories => format_categories(),
        SettingsCommand::Unknown(cmd) => {
            format!("Unknown command: '{}'\nType 'help' for available commands.", cmd)
        }
    }
}

/// Format a specific category's settings
fn format_category(settings: &UnifiedSettings, category: SettingsCategory) -> String {
    match category {
        SettingsCategory::Personality => format!("{:?}", settings.personality),
        SettingsCategory::Risk => format!("{:?}", settings.risk),
        SettingsCategory::Learning => format!("{:?}", settings.learning),
        SettingsCategory::Escalation => format!("{:?}", settings.escalation),
        SettingsCategory::Verbosity => format!("{:?}", settings.verbosity),
        SettingsCategory::Confirmation => format!("{:?}", settings.confirmation),
        SettingsCategory::Timeout => format!("{:?}", settings.timeout),
        SettingsCategory::OutputStyle => format!("{:?}", settings.output_style),
        SettingsCategory::Privacy => format!("{:?}", settings.privacy),
        SettingsCategory::Backup => format!("{:?}", settings.backup),
        SettingsCategory::Update => format!("{:?}", settings.update),
        SettingsCategory::Model => format!("{:?}", settings.model),
        SettingsCategory::Unknown => "Unknown category".to_string(),
    }
}

/// Format help text
fn format_help() -> String {
    let mut output = String::new();
    output.push_str("=== Settings Commands ===\n\n");
    output.push_str("Show settings:\n");
    output.push_str("  'show settings' - Show all settings\n");
    output.push_str("  'show personality' - Show specific category\n\n");
    output.push_str("Change settings:\n");
    output.push_str("  'be more formal' - Change personality\n");
    output.push_str("  'enable learning mode' - Toggle features\n\n");
    output.push_str("Reset settings:\n");
    output.push_str("  'reset settings' - Reset all to defaults\n");
    output.push_str("  'reset privacy' - Reset specific category\n\n");
    output.push_str("Import/Export:\n");
    output.push_str("  'export settings' - Export to file\n");
    output.push_str("  'import from file.json' - Import from file\n\n");
    output.push_str("Other:\n");
    output.push_str("  'validate settings' - Check for issues\n");
    output.push_str("  'list categories' - Show all categories\n");
    output
}

/// Format categories list
fn format_categories() -> String {
    let mut output = String::new();
    output.push_str("=== Settings Categories ===\n\n");
    for cat in UnifiedSettings::categories() {
        output.push_str(&format!("  - {}\n", cat));
    }
    output
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_display() {
        let cmd = SettingsCommand::Show(None);
        assert_eq!(format!("{}", cmd), "Show all settings");

        let cmd = SettingsCommand::Validate;
        assert_eq!(format!("{}", cmd), "Validate settings");
    }

    #[test]
    fn test_parse_result_new() {
        let result = ParseResult::new(SettingsCommand::Help, 1.0);
        assert!(result.is_confident());
        assert!(result.alternatives.is_empty());
    }

    #[test]
    fn test_parse_result_with_alternative() {
        let result = ParseResult::new(SettingsCommand::Help, 1.0)
            .with_alternative(SettingsCommand::ListCategories);
        assert_eq!(result.alternatives.len(), 1);
    }

    #[test]
    fn test_parse_show_all() {
        let result = parse_settings_command("show settings");
        assert!(matches!(result.command, SettingsCommand::Show(None)));
    }

    #[test]
    fn test_parse_show_category() {
        let result = parse_settings_command("show personality settings");
        assert!(matches!(
            result.command,
            SettingsCommand::Show(Some(SettingsCategory::Personality))
        ));
    }

    #[test]
    fn test_parse_reset() {
        let result = parse_settings_command("reset settings");
        assert!(matches!(result.command, SettingsCommand::Reset(None)));
    }

    #[test]
    fn test_parse_reset_category() {
        let result = parse_settings_command("reset privacy settings");
        assert!(matches!(
            result.command,
            SettingsCommand::Reset(Some(SettingsCategory::Privacy))
        ));
    }

    #[test]
    fn test_parse_validate() {
        let result = parse_settings_command("validate settings");
        assert!(matches!(result.command, SettingsCommand::Validate));
    }

    #[test]
    fn test_parse_help() {
        let result = parse_settings_command("help");
        assert!(matches!(result.command, SettingsCommand::Help));
    }

    #[test]
    fn test_parse_change() {
        let result = parse_settings_command("enable learning mode");
        assert!(matches!(result.command, SettingsCommand::Change(_)));
    }

    #[test]
    fn test_parse_export() {
        let result = parse_settings_command("export settings");
        assert!(matches!(result.command, SettingsCommand::Export(_)));
    }

    #[test]
    fn test_is_settings_command() {
        assert!(is_settings_command("show settings"));
        assert!(is_settings_command("configure preferences"));
        assert!(!is_settings_command("install vim"));
    }

    #[test]
    fn test_execute_help() {
        let mut settings = UnifiedSettings::default();
        let output = execute_command(&SettingsCommand::Help, &mut settings);
        assert!(output.contains("Commands"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_cli_fun_fact();
        assert!(fact.contains("natural language"));
    }
}
