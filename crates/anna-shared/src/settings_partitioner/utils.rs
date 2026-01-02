// v0.0.678: Settings Partitioner Utilities
// Helper functions for formatting and queries

use super::registry::PartitionerRegistry;

/// Format partitioner registry
pub fn format_partitioner_registry(registry: &PartitionerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Partitioner Registry:\n");
    output.push_str(&format!("  Partitioners: {}\n", registry.count()));
    output
}

/// Check if query is about partitioner
pub fn is_partitioner_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("partition settings") || lower.contains("settings partitioner") || lower.contains("split settings")
}

/// Fun fact about partitioner
pub fn partitioner_fun_fact() -> &'static str {
    "Anna's settings partitioner splits your settings into logical subsets!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_partitioner_query() {
        assert!(is_partitioner_query("partition settings"));
        assert!(!is_partitioner_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = partitioner_fun_fact();
        assert!(fact.contains("partitioner"));
    }
}
