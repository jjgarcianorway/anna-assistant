// v0.0.729: Settings Compact (Phase 305)
// Compact member

use serde::{Deserialize, Serialize};

/// Compact member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactMember {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Term ID
    pub term_id: String,
}

impl CompactMember {
    /// Create new member
    pub fn new(key: impl Into<String>, name: impl Into<String>, term_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            term_id: term_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_member_new() {
        let m = CompactMember::new("key", "name", "t1");
        assert_eq!(m.term_id, "t1");
    }
}
