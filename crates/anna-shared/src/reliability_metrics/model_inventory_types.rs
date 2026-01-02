//! Model inventory types and basic implementations.

use serde::{Deserialize, Serialize};

/// Model ownership - who installed this model?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelOwner {
    /// User installed this model (pre-existing or manual pull)
    User,
    /// Anna installed this model (auto-pulled for operation)
    Anna,
    /// Unknown ownership (legacy or unclear)
    Unknown,
}

/// A single model in the inventory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    /// Model name (e.g., "qwen2.5:7b")
    pub name: String,
    /// Normalized name for deduplication (lowercase, no tag variants)
    pub normalized: String,
    /// Owner (user/anna/unknown)
    pub owner: ModelOwner,
    /// Size in MB (if known)
    pub size_mb: Option<u64>,
    /// Quantization (if known)
    pub quantization: Option<String>,
    /// Last used timestamp (Unix ms)
    pub last_used_ms: Option<u64>,
    /// Is this configured for a role?
    pub is_configured: bool,
    /// Role it's configured for (if any)
    pub configured_role: Option<String>,
}

impl ModelEntry {
    /// Create a new model entry.
    pub fn new(name: impl Into<String>, owner: ModelOwner) -> Self {
        let name = name.into();
        let normalized = normalize_model_name(&name);
        Self {
            name,
            normalized,
            owner,
            size_mb: None,
            quantization: None,
            last_used_ms: None,
            is_configured: false,
            configured_role: None,
        }
    }

    /// Set size.
    pub fn with_size(mut self, size_mb: u64) -> Self {
        self.size_mb = Some(size_mb);
        self
    }

    /// Mark as configured for a role.
    pub fn with_role(mut self, role: &str) -> Self {
        self.is_configured = true;
        self.configured_role = Some(role.to_string());
        self
    }
}

/// Models configured for specific roles.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfiguredModels {
    pub translator: Option<String>,
    pub junior: Option<String>,
    pub senior: Option<String>,
}

/// Normalize model name for deduplication.
/// "qwen2.5:7b" and "qwen2.5:7b-instruct" are different
/// but "qwen2.5:7b" and "QWEN2.5:7B" are the same.
pub fn normalize_model_name(name: &str) -> String {
    name.to_lowercase().trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_entry_creation() {
        let entry = ModelEntry::new("qwen2.5:7b", ModelOwner::User);
        assert_eq!(entry.name, "qwen2.5:7b");
        assert_eq!(entry.normalized, "qwen2.5:7b");
        assert_eq!(entry.owner, ModelOwner::User);
        assert!(!entry.is_configured);
    }

    #[test]
    fn test_model_normalization() {
        assert_eq!(normalize_model_name("Qwen2.5:7b"), "qwen2.5:7b");
        assert_eq!(normalize_model_name("QWEN2.5:7B"), "qwen2.5:7b");
        assert_eq!(normalize_model_name("  qwen2.5:7b  "), "qwen2.5:7b");
    }

    #[test]
    fn test_model_entry_with_size() {
        let entry = ModelEntry::new("qwen2.5:7b", ModelOwner::User).with_size(4096);
        assert_eq!(entry.size_mb, Some(4096));
    }

    #[test]
    fn test_model_entry_with_role() {
        let entry = ModelEntry::new("qwen2.5:7b", ModelOwner::User).with_role("translator");
        assert!(entry.is_configured);
        assert_eq!(entry.configured_role, Some("translator".to_string()));
    }
}
