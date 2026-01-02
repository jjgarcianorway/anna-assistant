// v0.0.716: Settings Missive Enclosure (Phase 292)
// Enclosure implementation for missive system

use serde::{Deserialize, Serialize};

/// Missive enclosure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissiveEnclosure {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Letter ID
    pub letter_id: String,
}

impl MissiveEnclosure {
    /// Create new enclosure
    pub fn new(key: impl Into<String>, value: impl Into<String>, letter_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            letter_id: letter_id.into(),
        }
    }
}
