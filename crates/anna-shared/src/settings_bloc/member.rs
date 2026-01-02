// v0.0.740: Settings Bloc Member (Phase 316)
// Bloc member management

use serde::{Deserialize, Serialize};

/// Bloc member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlocMember {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Policy ID
    pub policy_id: String,
}

impl BlocMember {
    /// Create new member
    pub fn new(key: impl Into<String>, name: impl Into<String>, policy_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            policy_id: policy_id.into(),
        }
    }
}
