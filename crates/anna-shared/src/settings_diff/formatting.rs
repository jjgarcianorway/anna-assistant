// Formatting functions for settings diff

use super::types::SettingsDiff;

/// Format diff for display
pub fn format_diff(diff: &SettingsDiff) -> String {
    let mut output = String::new();

    if diff.is_identical() {
        output.push_str("Settings are identical.\n");
        return output;
    }

    output.push_str(&format!("=== Settings Diff ({} changes) ===\n\n", diff.change_count()));

    for entry in diff.changes_only() {
        output.push_str(&format!(
            "[{}] {}.{}\n",
            entry.diff_type, entry.category, entry.field
        ));
        if let Some(old) = &entry.old_value {
            output.push_str(&format!("  - {}\n", old));
        }
        if let Some(new) = &entry.new_value {
            output.push_str(&format!("  + {}\n", new));
        }
    }

    output
}

/// Fun fact about settings diff
pub fn settings_diff_fun_fact() -> &'static str {
    "Anna can show you exactly what changed between any two configurations!"
}
