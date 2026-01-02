// v0.0.652: Settings Binder - Helpers
// Helper functions for binder

use super::binder::SettingsBinderRegistry;

/// Format binder registry
pub fn format_binder_registry(registry: &SettingsBinderRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Binder Registry:\n");
    output.push_str(&format!("  Binders: {}\n", registry.count()));
    output
}

/// Check if query is about binder
pub fn is_binder_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("binder") || lower.contains("bind settings") || lower.contains("settings binding")
}

/// Fun fact about binder
pub fn binder_fun_fact() -> &'static str {
    "Anna's settings binders connect configs to runtime objects!"
}
