// v0.0.597: Settings Validator Chain - Utilities Module
// Utility functions for validation chains

use super::chain::ValidationChain;

/// Format validation chain
pub fn format_validator_chain(chain: &ValidationChain) -> String {
    let mut output = String::new();
    output.push_str("Validation Chain:\n");
    output.push_str(&format!("  Validators: {}\n", chain.count()));
    output.push_str(&format!("  Enabled: {}\n", chain.enabled_count()));
    output.push_str(&format!("  Stop on fail: {}\n", chain.stop_on_fail));

    for v in &chain.validators {
        let status = if v.enabled { "✓" } else { "✗" };
        output.push_str(&format!(
            "  {} [{}] {} ({})\n",
            status, v.validator_type, v.name, v.priority
        ));
    }

    output
}

/// Check if query is about validator chain
pub fn is_validator_chain_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("validator")
        || lower.contains("validation chain")
        || lower.contains("validate settings")
}

/// Fun fact about validator chains
pub fn validator_chain_fun_fact() -> &'static str {
    "Anna uses chainable validator pipelines to ensure your settings are always valid!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_validator_chain_query() {
        assert!(is_validator_chain_query("show validators"));
        assert!(!is_validator_chain_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = validator_chain_fun_fact();
        assert!(fact.contains("validator"));
    }
}
