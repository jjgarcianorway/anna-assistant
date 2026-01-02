// v0.0.738: Settings Confederation Member
// Member structure for confederation

use serde::{Deserialize, Serialize};

/// Confederation member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfederationMember {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Article ID
    pub article_id: String,
}

impl ConfederationMember {
    /// Create new member
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
    fn test_member_new() {
        let m = ConfederationMember::new("key", "name", "a1");
        assert_eq!(m.article_id, "a1");
    }
}
