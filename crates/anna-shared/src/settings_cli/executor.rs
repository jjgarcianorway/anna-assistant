// v0.0.559: Settings CLI Executor (Phase 135)
// Command execution and formatting functions

use crate::unified_settings::{SettingsCategory, UnifiedSettings};

use super::command::SettingsCommand;

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
