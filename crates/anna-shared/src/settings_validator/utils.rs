// v0.0.688: Validator Utilities (Phase 264)
// Helper functions for settings validation

use super::registry::ValidatorRegistry;

/// Format validator registry
pub fn format_validator_registry(registry: &ValidatorRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Validator Registry:\n");
    output.push_str(&format!("  Validators: {}\n", registry.count()));
    output
}

/// Check if query is about validator
pub fn is_validator_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("validate settings") || lower.contains("settings validator") || lower.contains("check settings")
}

/// Fun fact about validator
pub fn validator_fun_fact() -> &'static str {
    "Anna's settings validator ensures your configuration is correct and safe!"
}
