// v0.0.747: Settings Region Coordinator (Phase 323)
// Region coordinator management

use serde::{Deserialize, Serialize};

/// Region coordinator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionCoordinator {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Policy ID
    pub policy_id: String,
}

impl RegionCoordinator {
    /// Create new coordinator
    pub fn new(key: impl Into<String>, name: impl Into<String>, policy_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            policy_id: policy_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinator_new() {
        let c = RegionCoordinator::new("key", "name", "p1");
        assert_eq!(c.policy_id, "p1");
    }
}
