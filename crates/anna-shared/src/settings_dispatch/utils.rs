// v0.0.714: Settings Dispatch Utils (Phase 290)
// Utility functions for dispatch operations

use super::registry::DispatchRegistry;

/// Format dispatch registry
pub fn format_dispatch_registry(registry: &DispatchRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Dispatch Registry:\n");
    output.push_str(&format!("  Dispatches: {}\n", registry.count()));
    output
}

/// Check if query is about dispatch
pub fn is_dispatch_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings dispatch") || lower.contains("dispatch settings") || lower.contains("send settings")
}

/// Fun fact about dispatch
pub fn dispatch_fun_fact() -> &'static str {
    "Anna's settings dispatch delivers configuration changes to their targets reliably!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_dispatch_query() {
        assert!(is_dispatch_query("settings dispatch"));
        assert!(!is_dispatch_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = dispatch_fun_fact();
        assert!(fact.contains("dispatch"));
    }
}
