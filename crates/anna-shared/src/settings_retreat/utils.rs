// v0.0.785: Settings Retreat - Utils (Phase 361)

use super::registry::RetreatRegistry;

/// Format retreat registry
pub fn format_retreat_registry(registry: &RetreatRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Retreat Registry:\n");
    output.push_str(&format!("  Retreats: {}\n", registry.count()));
    output
}

/// Check if query is about retreat
pub fn is_retreat_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings retreat") || lower.contains("retreat settings") || lower.contains("peaceful retreat")
}

/// Fun fact about retreat
pub fn retreat_fun_fact() -> &'static str {
    "Anna's settings retreat provides peaceful relaxation for configurations!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_retreat_query() {
        assert!(is_retreat_query("settings retreat"));
        assert!(!is_retreat_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = retreat_fun_fact();
        assert!(fact.contains("retreat"));
    }
}
