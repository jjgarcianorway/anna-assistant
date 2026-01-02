// v0.0.771: Conservatory Curator
// Curator management for conservatory specimens

use serde::{Deserialize, Serialize};

/// Conservatory curator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConservatoryCurator {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Specimen ID
    pub specimen_id: String,
}

impl ConservatoryCurator {
    /// Create new curator
    pub fn new(key: impl Into<String>, name: impl Into<String>, specimen_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            specimen_id: specimen_id.into(),
        }
    }
}
