// v0.0.787: Settings Enclave (Phase 363)
// Enclave steward management

use serde::{Deserialize, Serialize};

/// Enclave steward
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnclaveSteward {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Member ID
    pub member_id: String,
}

impl EnclaveSteward {
    /// Create new steward
    pub fn new(key: impl Into<String>, name: impl Into<String>, member_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            member_id: member_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steward_new() {
        let s = EnclaveSteward::new("key", "name", "m1");
        assert_eq!(s.member_id, "m1");
    }
}
