// v0.0.729: Settings Compact (Phase 305)
// Compact term

use serde::{Deserialize, Serialize};

/// Compact term
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactTerm {
    /// Term ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Article number
    pub article: u32,
    /// Binding
    pub binding: bool,
}

impl CompactTerm {
    /// Create new term
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            article: 0,
            binding: true,
        }
    }

    /// Set article
    pub fn article(mut self, a: u32) -> Self {
        self.article = a;
        self
    }

    /// Make binding
    pub fn make_binding(&mut self) {
        self.binding = true;
    }

    /// Make non-binding
    pub fn make_non_binding(&mut self) {
        self.binding = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_term_new() {
        let t = CompactTerm::new("t1", "Title", "Content");
        assert_eq!(t.id, "t1");
    }

    #[test]
    fn test_term_builder() {
        let t = CompactTerm::new("t1", "Title", "Content")
            .article(1);
        assert_eq!(t.article, 1);
    }

    #[test]
    fn test_term_binding() {
        let mut t = CompactTerm::new("t1", "Title", "Content");
        t.make_non_binding();
        assert!(!t.binding);
        t.make_binding();
        assert!(t.binding);
    }
}
