// v0.0.781: Settings Sanctuary (Phase 357)
// Wildlife sanctuary for settings conservation - Warden

use serde::{Deserialize, Serialize};

/// Sanctuary warden
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctuaryWarden {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Resident ID
    pub resident_id: String,
}

impl SanctuaryWarden {
    /// Create new warden
    pub fn new(key: impl Into<String>, name: impl Into<String>, resident_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            resident_id: resident_id.into(),
        }
    }
}
