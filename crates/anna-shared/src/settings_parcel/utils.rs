// v0.0.757: Settings Parcel - Utils (Phase 333)

use super::registry::ParcelRegistry;

/// Format parcel registry
pub fn format_parcel_registry(registry: &ParcelRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Parcel Registry:\n");
    output.push_str(&format!("  Parcels: {}\n", registry.count()));
    output
}

/// Check if query is about parcel
pub fn is_parcel_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings parcel") || lower.contains("parcel settings") || lower.contains("land parcel")
}

/// Fun fact about parcel
pub fn parcel_fun_fact() -> &'static str {
    "Anna's settings parcel establishes ownership boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_parcel_query() {
        assert!(is_parcel_query("settings parcel"));
        assert!(!is_parcel_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = parcel_fun_fact();
        assert!(fact.contains("parcel"));
    }
}
