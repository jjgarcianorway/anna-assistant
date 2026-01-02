// v0.0.563: Settings History (Phase 139) - Formatting
// Format history entries for display

use super::manager::SettingsHistory;

/// Format history for display
pub fn format_history(history: &SettingsHistory, count: usize) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "=== Settings History ({} entries) ===\n\n",
        history.count()
    ));

    if history.is_empty() {
        output.push_str("No history entries.\n");
        return output;
    }

    output.push_str(&format!(
        "Position: {}/{} (undo: {}, redo: {})\n\n",
        history.current_position(),
        history.count(),
        history.undo_count(),
        history.redo_count()
    ));

    for (i, entry) in history.recent(count).iter().enumerate() {
        let age = format_age(entry.age());
        let cat = entry
            .category
            .map(|c| format!(" [{}]", c))
            .unwrap_or_default();
        output.push_str(&format!(
            "{}. {} - {}{}\n",
            i + 1,
            age,
            entry.description,
            cat
        ));
    }

    output
}

/// Format duration as human-readable age
fn format_age(duration: chrono::Duration) -> String {
    let secs = duration.num_seconds();
    if secs < 60 {
        format!("{}s ago", secs)
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86400)
    }
}

/// Fun fact about settings history
pub fn settings_history_fun_fact() -> &'static str {
    "Anna remembers your last 100 settings changes - you can always undo and redo!"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unified_settings::UnifiedSettings;

    #[test]
    fn test_format_history_empty() {
        let history = SettingsHistory::new();
        let output = format_history(&history, 10);
        assert!(output.contains("No history"));
    }

    #[test]
    fn test_format_age() {
        assert_eq!(format_age(chrono::Duration::seconds(30)), "30s ago");
        assert_eq!(format_age(chrono::Duration::minutes(5)), "5m ago");
        assert_eq!(format_age(chrono::Duration::hours(2)), "2h ago");
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_history_fun_fact();
        assert!(fact.contains("100"));
    }
}
