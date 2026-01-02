// v0.0.784: Settings Haven (Phase 360)
// Safe haven for settings protection - Keeper module

use serde::{Deserialize, Serialize};

/// Haven keeper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HavenKeeper {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Guest ID
    pub guest_id: String,
}

impl HavenKeeper {
    /// Create new keeper
    pub fn new(key: impl Into<String>, name: impl Into<String>, guest_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            guest_id: guest_id.into(),
        }
    }
}
