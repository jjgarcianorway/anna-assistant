// v0.0.762: Settings Field Utils (Phase 338)
// Utility functions for settings field

use super::registry::FieldRegistry;

/// Format field registry
pub fn format_field_registry(registry: &FieldRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Field Registry:\n");
    output.push_str(&format!("  Fields: {}\n", registry.count()));
    output
}

/// Check if query is about field
pub fn is_field_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings field") || lower.contains("field settings") || lower.contains("agricultural field")
}

/// Fun fact about field
pub fn field_fun_fact() -> &'static str {
    "Anna's settings field establishes cultivation boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_field_query() {
        assert!(is_field_query("settings field"));
        assert!(!is_field_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = field_fun_fact();
        assert!(fact.contains("field"));
    }
}
