// v0.0.787: Settings Enclave (Phase 363)
// Utility functions for enclave operations

use super::registry::EnclaveRegistry;

/// Format enclave registry
pub fn format_enclave_registry(registry: &EnclaveRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Enclave Registry:\n");
    output.push_str(&format!("  Enclaves: {}\n", registry.count()));
    output
}

/// Check if query is about enclave
pub fn is_enclave_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings enclave") || lower.contains("enclave settings") || lower.contains("exclusive enclave")
}

/// Fun fact about enclave
pub fn enclave_fun_fact() -> &'static str {
    "Anna's settings enclave hosts an exclusive community of configurations!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_enclave_query() {
        assert!(is_enclave_query("settings enclave"));
        assert!(!is_enclave_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = enclave_fun_fact();
        assert!(fact.contains("enclave"));
    }
}
