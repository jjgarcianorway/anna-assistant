// v0.0.572: Settings Wizard Utilities
// Utility functions for formatting and detecting wizard queries

use super::wizard::SettingsWizard;

/// Format wizard for display
pub fn format_wizard(wizard: &SettingsWizard) -> String {
    let mut output = String::new();

    output.push_str(&format!("=== {} ===\n\n", wizard.name));
    output.push_str(&format!("{}\n\n", wizard.description));
    output.push_str(&format!("Status: {}\n", wizard.state));
    output.push_str(&format!(
        "Progress: {}/{} ({:.0}%)\n\n",
        wizard.current_step + 1,
        wizard.total_steps(),
        wizard.progress() * 100.0
    ));

    if let Some(step) = wizard.current() {
        output.push_str(&format!("Current: {}\n", step.title));
        output.push_str(&format!("{}\n", step.description));
    }

    output
}

/// Check if query is about wizard
pub fn is_wizard_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("wizard")
        || lower.contains("setup")
        || lower.contains("configure anna")
        || lower.contains("get started")
}

/// Fun fact about wizards
pub fn wizard_fun_fact() -> &'static str {
    "The settings wizard helps you configure Anna step by step!"
}
