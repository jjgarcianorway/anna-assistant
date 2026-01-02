// v0.0.766: Settings Orchard Picker
// Orchard picker structure

use serde::{Deserialize, Serialize};

/// Orchard picker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchardPicker {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Fruit ID
    pub fruit_id: String,
}

impl OrchardPicker {
    /// Create new picker
    pub fn new(key: impl Into<String>, name: impl Into<String>, fruit_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            fruit_id: fruit_id.into(),
        }
    }
}
