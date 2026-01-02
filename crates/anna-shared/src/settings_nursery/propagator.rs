// v0.0.769: Settings Nursery - Propagator (Phase 345)

use serde::{Deserialize, Serialize};

/// Nursery propagator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NurseryPropagator {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Seedling ID
    pub seedling_id: String,
}

impl NurseryPropagator {
    /// Create new propagator
    pub fn new(key: impl Into<String>, name: impl Into<String>, seedling_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            seedling_id: seedling_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_propagator_new() {
        let p = NurseryPropagator::new("key", "name", "s1");
        assert_eq!(p.seedling_id, "s1");
    }
}
