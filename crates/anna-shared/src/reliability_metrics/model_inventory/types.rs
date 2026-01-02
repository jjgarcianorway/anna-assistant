//! Model inventory types (v0.0.444).

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
