// v0.0.744: Settings Realm Config (Phase 320)
// Realm configuration

use serde::{Deserialize, Serialize};
use super::types::{RealmType, RealmStatus};

/// Realm config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmConfig {
    /// Name
    pub name: String,
    /// Realm type
    pub realm_type: RealmType,
    /// Status
    pub status: RealmStatus,
    /// Max decrees
    pub max_decrees: usize,
}

impl RealmConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            realm_type: RealmType::Kingdom,
            status: RealmStatus::Rising,
            max_decrees: 100,
        }
    }

    /// Set type
    pub fn realm_type(mut self, rt: RealmType) -> Self {
        self.realm_type = rt;
        self
    }

    /// Set status
    pub fn status(mut self, s: RealmStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max decrees
    pub fn max_decrees(mut self, max: usize) -> Self {
        self.max_decrees = max;
        self
    }
}

impl Default for RealmConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = RealmConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = RealmConfig::new("test")
            .realm_type(RealmType::Empire)
            .status(RealmStatus::Prosperous);
        assert_eq!(c.realm_type, RealmType::Empire);
        assert_eq!(c.status, RealmStatus::Prosperous);
    }
}
