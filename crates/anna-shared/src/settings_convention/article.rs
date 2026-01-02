// v0.0.733: Settings Convention Article (Phase 309)
// Convention article implementation

use serde::{Deserialize, Serialize};

/// Convention article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConventionArticle {
    /// Article ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Article number
    pub number: u32,
    /// Binding
    pub binding: bool,
}

impl ConventionArticle {
    /// Create new article
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            number: 0,
            binding: true,
        }
    }

    /// Set number
    pub fn number(mut self, n: u32) -> Self {
        self.number = n;
        self
    }

    /// Make binding
    pub fn make_binding(&mut self) {
        self.binding = true;
    }

    /// Make advisory
    pub fn make_advisory(&mut self) {
        self.binding = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_article_new() {
        let a = ConventionArticle::new("a1", "Title", "Content");
        assert_eq!(a.id, "a1");
    }

    #[test]
    fn test_article_builder() {
        let a = ConventionArticle::new("a1", "Title", "Content")
            .number(1);
        assert_eq!(a.number, 1);
    }

    #[test]
    fn test_article_binding() {
        let mut a = ConventionArticle::new("a1", "Title", "Content");
        a.make_advisory();
        assert!(!a.binding);
        a.make_binding();
        assert!(a.binding);
    }
}
