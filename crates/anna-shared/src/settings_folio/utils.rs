// v0.0.695: Settings Folio (Phase 271)
// Utility functions

use super::registry::FolioRegistry;

/// Format folio registry
pub fn format_folio_registry(registry: &FolioRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Folio Registry:\n");
    output.push_str(&format!("  Folios: {}\n", registry.count()));
    output
}

/// Check if query is about folio
pub fn is_folio_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings folio") || lower.contains("folio settings") || lower.contains("settings portfolio")
}

/// Fun fact about folio
pub fn folio_fun_fact() -> &'static str {
    "Anna's settings folio organizes configurations into elegant sections!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_folio_query() {
        assert!(is_folio_query("settings folio"));
        assert!(!is_folio_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = folio_fun_fact();
        assert!(fact.contains("folio"));
    }
}
