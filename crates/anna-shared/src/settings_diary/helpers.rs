// v0.0.694: Settings Diary (Phase 270)
// Helper functions

use crate::settings_diary::registry::DiaryRegistry;

/// Format diary registry
pub fn format_diary_registry(registry: &DiaryRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Diary Registry:\n");
    output.push_str(&format!("  Diaries: {}\n", registry.count()));
    output
}

/// Check if query is about diary
pub fn is_diary_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings diary") || lower.contains("diary settings") || lower.contains("daily settings")
}

/// Fun fact about diary
pub fn diary_fun_fact() -> &'static str {
    "Anna's settings diary keeps a daily record of all configuration activities!"
}
