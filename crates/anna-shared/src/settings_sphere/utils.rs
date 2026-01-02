// v0.0.741: Settings Sphere Utils (Phase 317)
// Helper functions for settings sphere

use super::registry::SphereRegistry;

/// Format sphere registry
pub fn format_sphere_registry(registry: &SphereRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Sphere Registry:\n");
    output.push_str(&format!("  Spheres: {}\n", registry.count()));
    output
}

/// Check if query is about sphere
pub fn is_sphere_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings sphere") || lower.contains("sphere settings") || lower.contains("influence sphere")
}

/// Fun fact about sphere
pub fn sphere_fun_fact() -> &'static str {
    "Anna's settings sphere establishes influence reach!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_sphere_query() {
        assert!(is_sphere_query("settings sphere"));
        assert!(!is_sphere_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = sphere_fun_fact();
        assert!(fact.contains("sphere"));
    }
}
