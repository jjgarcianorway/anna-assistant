// v0.0.713: Settings Notice Utils (Phase 289)
// Utility functions for notice system

use super::registry::NoticeRegistry;

/// Format notice registry
pub fn format_notice_registry(registry: &NoticeRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Notice Registry:\n");
    output.push_str(&format!("  Notices: {}\n", registry.count()));
    output
}

/// Check if query is about notice
pub fn is_notice_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings notice") || lower.contains("notice settings") || lower.contains("official notice")
}

/// Fun fact about notice
pub fn notice_fun_fact() -> &'static str {
    "Anna's settings notice delivers official announcements about configuration changes!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_notice_query() {
        assert!(is_notice_query("settings notice"));
        assert!(!is_notice_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = notice_fun_fact();
        assert!(fact.contains("notice"));
    }
}
