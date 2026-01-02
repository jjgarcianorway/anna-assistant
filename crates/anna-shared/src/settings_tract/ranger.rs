// v0.0.759: Settings Tract Ranger (Phase 335)
// Tract ranger management

use serde::{Deserialize, Serialize};

/// Tract ranger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TractRanger {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Grant ID
    pub grant_id: String,
}

impl TractRanger {
    /// Create new ranger
    pub fn new(key: impl Into<String>, name: impl Into<String>, grant_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            grant_id: grant_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ranger_new() {
        let r = TractRanger::new("key", "name", "g1");
        assert_eq!(r.grant_id, "g1");
    }
}
