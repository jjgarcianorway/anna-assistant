// v0.0.559: Settings CLI Tests (Phase 135)
// Test cases for settings CLI interface

#[cfg(test)]
mod tests {
    use crate::unified_settings::{SettingsCategory, UnifiedSettings};

    use super::super::command::SettingsCommand;
    use super::super::executor::execute_command;
    use super::super::helpers::{is_settings_command, parse_settings_command, settings_cli_fun_fact};
    use super::super::parse_result::ParseResult;

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
