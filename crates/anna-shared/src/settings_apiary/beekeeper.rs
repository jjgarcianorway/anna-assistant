// v0.0.779: Settings Apiary - Beekeeper (Phase 355)
// Apiary beekeeper management

use serde::{Deserialize, Serialize};

/// Apiary beekeeper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiaryBeekeeper {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Hive ID
    pub hive_id: String,
}

impl ApiaryBeekeeper {
    /// Create new beekeeper
    pub fn new(key: impl Into<String>, name: impl Into<String>, hive_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            hive_id: hive_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beekeeper_new() {
        let b = ApiaryBeekeeper::new("key", "name", "h1");
        assert_eq!(b.hive_id, "h1");
    }
}
