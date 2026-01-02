// v0.0.738: Settings Confederation Article
// Article structure for confederation

use serde::{Deserialize, Serialize};

/// Confederation article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfederationArticle {
    /// Article ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Clause number
    pub clause: u32,
    /// Voluntary
    pub voluntary: bool,
}

impl ConfederationArticle {
    /// Create new article
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            clause: 0,
            voluntary: true,
        }
    }

    /// Set clause
    pub fn clause(mut self, c: u32) -> Self {
        self.clause = c;
        self
    }

    /// Make voluntary
    pub fn make_voluntary(&mut self) {
        self.voluntary = true;
    }

    /// Make mandatory
    pub fn make_mandatory(&mut self) {
        self.voluntary = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_article_new() {
        let a = ConfederationArticle::new("a1", "Title", "Content");
        assert_eq!(a.id, "a1");
    }

    #[test]
    fn test_article_builder() {
        let a = ConfederationArticle::new("a1", "Title", "Content")
            .clause(1);
        assert_eq!(a.clause, 1);
    }

    #[test]
    fn test_article_voluntary() {
        let mut a = ConfederationArticle::new("a1", "Title", "Content");
        a.make_mandatory();
        assert!(!a.voluntary);
        a.make_voluntary();
        assert!(a.voluntary);
    }
}
