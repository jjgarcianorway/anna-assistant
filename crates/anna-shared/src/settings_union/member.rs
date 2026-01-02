// v0.0.739: Settings Union (Phase 315)
// Political union for settings integration - Member

use serde::{Deserialize, Serialize};

/// Union member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnionMember {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Provision ID
    pub provision_id: String,
}

impl UnionMember {
    /// Create new member
    pub fn new(key: impl Into<String>, name: impl Into<String>, provision_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            provision_id: provision_id.into(),
        }
    }
}
