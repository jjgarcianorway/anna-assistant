// v0.0.735: Settings Alliance (Phase 311)
// Alliance member

use serde::{Deserialize, Serialize};

/// Alliance member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllianceMember {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Commitment ID
    pub commitment_id: String,
}

impl AllianceMember {
    /// Create new member
    pub fn new(key: impl Into<String>, name: impl Into<String>, commitment_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            commitment_id: commitment_id.into(),
        }
    }
}
