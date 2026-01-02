//! Common types for inventory management - v0.0.443.

use serde::{Deserialize, Serialize};

/// Model installation source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstalledBy {
    /// Installed by Anna.
    Anna,
    /// Installed by user (preexisting or manual).
    User,
    /// Unknown (detected but no history).
    Unknown,
}

/// Normalize model name.
pub fn normalize_model_name(name: &str) -> String {
    // Remove trailing :latest, lowercase
    name.trim_end_matches(":latest").to_lowercase().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_model_name() {
        assert_eq!(normalize_model_name("Qwen2.5:7B:latest"), "qwen2.5:7b");
        assert_eq!(normalize_model_name("llama3:8b"), "llama3:8b");
    }
}
