// v0.0.746: Settings Province - Governor (Phase 322)
// Province governors

use serde::{Deserialize, Serialize};

/// Province governor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvinceGovernor {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Edict ID
    pub edict_id: String,
}

impl ProvinceGovernor {
    /// Create new governor
    pub fn new(key: impl Into<String>, name: impl Into<String>, edict_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            edict_id: edict_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_governor_new() {
        let g = ProvinceGovernor::new("key", "name", "e1");
        assert_eq!(g.edict_id, "e1");
    }
}
