// v0.0.765: Settings Grove (Phase 341)
// Grove tree structure

use serde::{Deserialize, Serialize};

/// Grove tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroveTree {
    /// Tree ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Row number
    pub row: u32,
    /// Healthy
    pub healthy: bool,
}

impl GroveTree {
    /// Create new tree
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            row: 0,
            healthy: true,
        }
    }

    /// Set row
    pub fn row(mut self, r: u32) -> Self {
        self.row = r;
        self
    }

    /// Make healthy
    pub fn make_healthy(&mut self) {
        self.healthy = true;
    }

    /// Make diseased
    pub fn make_diseased(&mut self) {
        self.healthy = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_new() {
        let t = GroveTree::new("t1", "Title", "Content");
        assert_eq!(t.id, "t1");
    }

    #[test]
    fn test_tree_builder() {
        let t = GroveTree::new("t1", "Title", "Content")
            .row(1);
        assert_eq!(t.row, 1);
    }

    #[test]
    fn test_tree_healthy() {
        let mut t = GroveTree::new("t1", "Title", "Content");
        t.make_diseased();
        assert!(!t.healthy);
        t.make_healthy();
        assert!(t.healthy);
    }
}
