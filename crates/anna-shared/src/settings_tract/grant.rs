// v0.0.759: Settings Tract Grant (Phase 335)
// Tract grant management

use serde::{Deserialize, Serialize};

/// Tract grant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TractGrant {
    /// Grant ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Range number
    pub range: u32,
    /// Patented
    pub patented: bool,
}

impl TractGrant {
    /// Create new grant
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            range: 0,
            patented: true,
        }
    }

    /// Set range
    pub fn range(mut self, r: u32) -> Self {
        self.range = r;
        self
    }

    /// Make patented
    pub fn make_patented(&mut self) {
        self.patented = true;
    }

    /// Make pending
    pub fn make_pending(&mut self) {
        self.patented = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grant_new() {
        let g = TractGrant::new("g1", "Title", "Content");
        assert_eq!(g.id, "g1");
    }

    #[test]
    fn test_grant_builder() {
        let g = TractGrant::new("g1", "Title", "Content")
            .range(1);
        assert_eq!(g.range, 1);
    }

    #[test]
    fn test_grant_patented() {
        let mut g = TractGrant::new("g1", "Title", "Content");
        g.make_pending();
        assert!(!g.patented);
        g.make_patented();
        assert!(g.patented);
    }
}
