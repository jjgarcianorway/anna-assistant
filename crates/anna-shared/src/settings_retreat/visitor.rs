// v0.0.785: Settings Retreat - Visitor (Phase 361)

use serde::{Deserialize, Serialize};

/// Retreat visitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetreatVisitor {
    /// Visitor ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Cabin number
    pub cabin: u32,
    /// Relaxed
    pub relaxed: bool,
}

impl RetreatVisitor {
    /// Create new visitor
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            cabin: 0,
            relaxed: true,
        }
    }

    /// Set cabin
    pub fn cabin(mut self, c: u32) -> Self {
        self.cabin = c;
        self
    }

    /// Make relaxed
    pub fn make_relaxed(&mut self) {
        self.relaxed = true;
    }

    /// Make stressed
    pub fn make_stressed(&mut self) {
        self.relaxed = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visitor_new() {
        let v = RetreatVisitor::new("v1", "Title", "Content");
        assert_eq!(v.id, "v1");
    }

    #[test]
    fn test_visitor_builder() {
        let v = RetreatVisitor::new("v1", "Title", "Content")
            .cabin(1);
        assert_eq!(v.cabin, 1);
    }

    #[test]
    fn test_visitor_relaxation() {
        let mut v = RetreatVisitor::new("v1", "Title", "Content");
        v.make_stressed();
        assert!(!v.relaxed);
        v.make_relaxed();
        assert!(v.relaxed);
    }
}
