// v0.0.744: Settings Realm Vassal (Phase 320)
// Realm vassal structure

use serde::{Deserialize, Serialize};

/// Realm vassal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmVassal {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Decree ID
    pub decree_id: String,
}

impl RealmVassal {
    /// Create new vassal
    pub fn new(key: impl Into<String>, name: impl Into<String>, decree_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            decree_id: decree_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vassal_new() {
        let v = RealmVassal::new("key", "name", "d1");
        assert_eq!(v.decree_id, "d1");
    }
}
