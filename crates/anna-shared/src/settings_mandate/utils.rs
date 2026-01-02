// v0.0.721: Settings Mandate Utilities (Phase 297)
// Helper functions for mandate system

use super::registry::MandateRegistry;

/// Format mandate registry
pub fn format_mandate_registry(registry: &MandateRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Mandate Registry:\n");
    output.push_str(&format!("  Mandates: {}\n", registry.count()));
    output
}

/// Check if query is about mandate
pub fn is_mandate_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings mandate") || lower.contains("mandate settings") || lower.contains("compliance mandate")
}

/// Fun fact about mandate
pub fn mandate_fun_fact() -> &'static str {
    "Anna's settings mandate ensures configuration compliance with authoritative requirements!"
}
