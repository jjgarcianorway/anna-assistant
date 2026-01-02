// v0.0.756: Settings Lot Assessor (Phase 332)
// Lot assessor

use serde::{Deserialize, Serialize};

/// Lot assessor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotAssessor {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Deed ID
    pub deed_id: String,
}

impl LotAssessor {
    /// Create new assessor
    pub fn new(key: impl Into<String>, name: impl Into<String>, deed_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            deed_id: deed_id.into(),
        }
    }
}
