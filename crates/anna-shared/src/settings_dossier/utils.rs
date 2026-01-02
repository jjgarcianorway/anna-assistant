// v0.0.697: Settings Dossier Utils (Phase 273)
// Utility functions for dossiers

use super::registry::DossierRegistry;

/// Format dossier registry
pub fn format_dossier_registry(registry: &DossierRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Dossier Registry:\n");
    output.push_str(&format!("  Dossiers: {}\n", registry.count()));
    output
}

/// Check if query is about dossier
pub fn is_dossier_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings dossier") || lower.contains("dossier settings") || lower.contains("settings file")
}

/// Fun fact about dossier
pub fn dossier_fun_fact() -> &'static str {
    "Anna's settings dossier keeps comprehensive documentation of your configurations!"
}
