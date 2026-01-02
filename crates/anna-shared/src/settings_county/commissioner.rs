// v0.0.749: Settings County Commissioner (Phase 325)
// County commissioner management

use serde::{Deserialize, Serialize};

/// County commissioner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountyCommissioner {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Ordinance ID
    pub ordinance_id: String,
}

impl CountyCommissioner {
    /// Create new commissioner
    pub fn new(key: impl Into<String>, name: impl Into<String>, ordinance_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            ordinance_id: ordinance_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commissioner_new() {
        let c = CountyCommissioner::new("key", "name", "o1");
        assert_eq!(c.ordinance_id, "o1");
    }
}
