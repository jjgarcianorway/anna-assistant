// v0.0.764: Settings Pasture - Herd (Phase 340)

use serde::{Deserialize, Serialize};

/// Pasture herd
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastureHerd {
    /// Herd ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Paddock number
    pub paddock: u32,
    /// Thriving
    pub thriving: bool,
}

impl PastureHerd {
    /// Create new herd
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            paddock: 0,
            thriving: true,
        }
    }

    /// Set paddock
    pub fn paddock(mut self, p: u32) -> Self {
        self.paddock = p;
        self
    }

    /// Make thriving
    pub fn make_thriving(&mut self) {
        self.thriving = true;
    }

    /// Make struggling
    pub fn make_struggling(&mut self) {
        self.thriving = false;
    }
}

/// Pasture herder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastureHerder {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Herd ID
    pub herd_id: String,
}

impl PastureHerder {
    /// Create new herder
    pub fn new(key: impl Into<String>, name: impl Into<String>, herd_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            herd_id: herd_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_herd_new() {
        let h = PastureHerd::new("h1", "Title", "Content");
        assert_eq!(h.id, "h1");
    }

    #[test]
    fn test_herd_builder() {
        let h = PastureHerd::new("h1", "Title", "Content")
            .paddock(1);
        assert_eq!(h.paddock, 1);
    }

    #[test]
    fn test_herd_thriving() {
        let mut h = PastureHerd::new("h1", "Title", "Content");
        h.make_struggling();
        assert!(!h.thriving);
        h.make_thriving();
        assert!(h.thriving);
    }

    #[test]
    fn test_herder_new() {
        let h = PastureHerder::new("key", "name", "h1");
        assert_eq!(h.herd_id, "h1");
    }
}
