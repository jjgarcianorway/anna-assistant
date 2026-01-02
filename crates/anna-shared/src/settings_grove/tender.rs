// v0.0.765: Settings Grove (Phase 341)
// Grove tender structure

use serde::{Deserialize, Serialize};

/// Grove tender
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroveTender {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Tree ID
    pub tree_id: String,
}

impl GroveTender {
    /// Create new tender
    pub fn new(key: impl Into<String>, name: impl Into<String>, tree_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            tree_id: tree_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tender_new() {
        let t = GroveTender::new("key", "name", "t1");
        assert_eq!(t.tree_id, "t1");
    }
}
