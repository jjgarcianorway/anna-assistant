// v0.0.569: Settings Templates - Helpers (Phase 145)
// Helper functions for template operations and formatting

use crate::unified_settings::{SettingsCategory, UnifiedSettings};
use super::manager::TemplateManager;

/// Apply a single category from source to target
pub fn apply_category(target: &mut UnifiedSettings, source: &UnifiedSettings, category: SettingsCategory) {
    match category {
        SettingsCategory::Personality => target.personality = source.personality.clone(),
        SettingsCategory::Risk => target.risk = source.risk.clone(),
        SettingsCategory::Learning => target.learning = source.learning.clone(),
        SettingsCategory::Escalation => target.escalation = source.escalation.clone(),
        SettingsCategory::Verbosity => target.verbosity = source.verbosity.clone(),
        SettingsCategory::Confirmation => target.confirmation = source.confirmation.clone(),
        SettingsCategory::Timeout => target.timeout = source.timeout.clone(),
        SettingsCategory::OutputStyle => target.output_style = source.output_style.clone(),
        SettingsCategory::Privacy => target.privacy = source.privacy.clone(),
        SettingsCategory::Backup => target.backup = source.backup.clone(),
        SettingsCategory::Update => target.update = source.update.clone(),
        SettingsCategory::Model => target.model = source.model.clone(),
        SettingsCategory::Unknown => {}
    }
}

/// Format templates for display
pub fn format_templates(manager: &TemplateManager) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Templates ===\n\n");

    if manager.count() == 0 {
        output.push_str("No templates available.\n");
        return output;
    }

    // Built-in templates
    let builtin = manager.builtin();
    if !builtin.is_empty() {
        output.push_str("Built-in Templates:\n");
        for t in builtin {
            output.push_str(&format!(
                "  • {} - {} [{}]\n",
                t.meta.name, t.meta.description, t.meta.use_case
            ));
        }
        output.push('\n');
    }

    // User templates
    let user = manager.user_templates();
    if !user.is_empty() {
        output.push_str("User Templates:\n");
        for t in user {
            output.push_str(&format!(
                "  • {} - {} [{}] (used {} times)\n",
                t.meta.name, t.meta.description, t.meta.use_case, t.usage_count
            ));
        }
    }

    output
}

/// Check if query is about templates
pub fn is_template_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("template")
        || lower.contains("create template")
        || lower.contains("apply template")
        || lower.contains("settings template")
}

/// Fun fact about templates
pub fn template_fun_fact() -> &'static str {
    "Settings templates let you quickly switch between different configurations!"
}
