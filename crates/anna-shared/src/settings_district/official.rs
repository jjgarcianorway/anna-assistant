// v0.0.748: Settings District Official (Phase 324)
// District official management

use serde::{Deserialize, Serialize};

/// District official
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistrictOfficial {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Bylaw ID
    pub bylaw_id: String,
}

impl DistrictOfficial {
    /// Create new official
    pub fn new(key: impl Into<String>, name: impl Into<String>, bylaw_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            bylaw_id: bylaw_id.into(),
        }
    }
}
