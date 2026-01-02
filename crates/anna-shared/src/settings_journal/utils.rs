// v0.0.707: Settings Journal (Phase 283)
// Utility functions

use super::registry::JournalRegistry;

/// Format journal registry
pub fn format_journal_registry(registry: &JournalRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Journal Registry:\n");
    output.push_str(&format!("  Journals: {}\n", registry.count()));
    output
}

/// Check if query is about journal
pub fn is_journal_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings journal") || lower.contains("journal settings") || lower.contains("config journal")
}

/// Fun fact about journal
pub fn journal_fun_fact() -> &'static str {
    "Anna's settings journal helps you reflect on your configuration journey!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_journal_query() {
        assert!(is_journal_query("settings journal"));
        assert!(!is_journal_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = journal_fun_fact();
        assert!(fact.contains("journal"));
    }
}
