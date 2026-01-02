// v0.0.726: Settings Covenant (Phase 302)
// Binding agreement for settings governance - Helper Functions

use super::registry::CovenantRegistry;

/// Format covenant registry
pub fn format_covenant_registry(registry: &CovenantRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Covenant Registry:\n");
    output.push_str(&format!("  Covenants: {}\n", registry.count()));
    output
}

/// Check if query is about covenant
pub fn is_covenant_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings covenant") || lower.contains("covenant settings") || lower.contains("binding agreement")
}

/// Fun fact about covenant
pub fn covenant_fun_fact() -> &'static str {
    "Anna's settings covenant establishes binding governance agreements!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_covenant_query() {
        assert!(is_covenant_query("settings covenant"));
        assert!(!is_covenant_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = covenant_fun_fact();
        assert!(fact.contains("covenant"));
    }
}
