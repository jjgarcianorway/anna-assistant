// v0.0.662: Settings Patcher Entry (Phase 238)
// Single patch entry definition

use serde::{Deserialize, Serialize};

use super::types::PatchOperation;

/// Single patch entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchEntry {
    /// Operation
    pub operation: PatchOperation,
    /// Target key
    pub key: String,
    /// Value (for add/replace)
    pub value: Option<String>,
    /// Source key (for copy/move)
    pub source_key: Option<String>,
}

impl PatchEntry {
    /// Create add patch
    pub fn add(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            operation: PatchOperation::Add,
            key: key.into(),
            value: Some(value.into()),
            source_key: None,
        }
    }

    /// Create remove patch
    pub fn remove(key: impl Into<String>) -> Self {
        Self {
            operation: PatchOperation::Remove,
            key: key.into(),
            value: None,
            source_key: None,
        }
    }

    /// Create replace patch
    pub fn replace(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            operation: PatchOperation::Replace,
            key: key.into(),
            value: Some(value.into()),
            source_key: None,
        }
    }

    /// Create copy patch
    pub fn copy(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            operation: PatchOperation::Copy,
            key: to.into(),
            value: None,
            source_key: Some(from.into()),
        }
    }

    /// Create move patch
    pub fn move_key(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            operation: PatchOperation::Move,
            key: to.into(),
            value: None,
            source_key: Some(from.into()),
        }
    }
}
