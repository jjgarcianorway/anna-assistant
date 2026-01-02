// v0.0.733: Settings Convention Party (Phase 309)
// Convention party implementation

use serde::{Deserialize, Serialize};

/// Convention party
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConventionParty {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Article ID
    pub article_id: String,
}

impl ConventionParty {
    /// Create new party
    pub fn new(key: impl Into<String>, name: impl Into<String>, article_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            article_id: article_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_party_new() {
        let p = ConventionParty::new("key", "name", "a1");
        assert_eq!(p.article_id, "a1");
    }
}
