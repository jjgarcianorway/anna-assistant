// v0.0.785: Settings Retreat - Guide (Phase 361)

use serde::{Deserialize, Serialize};

/// Retreat guide
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetreatGuide {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Visitor ID
    pub visitor_id: String,
}

impl RetreatGuide {
    /// Create new guide
    pub fn new(key: impl Into<String>, name: impl Into<String>, visitor_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            visitor_id: visitor_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guide_new() {
        let g = RetreatGuide::new("key", "name", "v1");
        assert_eq!(g.visitor_id, "v1");
    }
}
