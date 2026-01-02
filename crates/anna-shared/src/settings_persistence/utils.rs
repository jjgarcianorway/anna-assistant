// v0.0.555: Settings Persistence - Utility functions (Phase 131)
// Utility functions for settings persistence

use super::manager::SettingsPersistence;

/// Format settings summary for display
pub fn format_persistence_status() -> String {
    let mut output = String::new();

    output.push_str("=== Settings Persistence ===\n\n");

    if let Some(path) = SettingsPersistence::settings_path() {
        output.push_str(&format!("Config path: {}\n", path.display()));
        output.push_str(&format!(
            "Settings exist: {}\n",
            if path.exists() { "Yes" } else { "No" }
        ));
    }

    if let Ok(backups) = SettingsPersistence::list_backups() {
        output.push_str(&format!("Backup count: {}\n", backups.len()));
    }

    output
}

/// Check if settings persistence is available
pub fn is_persistence_available() -> bool {
    SettingsPersistence::settings_path().is_some()
}

/// Fun fact about settings persistence
pub fn settings_persistence_fun_fact() -> &'static str {
    "Anna keeps up to 5 backup copies of your settings - you can always restore to a previous configuration!"
}
