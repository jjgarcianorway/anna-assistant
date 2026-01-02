// v0.0.744: Settings Realm Decree (Phase 320)
// Realm decree structure

use serde::{Deserialize, Serialize};

/// Realm decree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmDecree {
    /// Decree ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Order number
    pub order: u32,
    /// Royal
    pub royal: bool,
}

impl RealmDecree {
    /// Create new decree
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            order: 0,
            royal: true,
        }
    }

    /// Set order
    pub fn order(mut self, o: u32) -> Self {
        self.order = o;
        self
    }

    /// Make royal
    pub fn make_royal(&mut self) {
        self.royal = true;
    }

    /// Make common
    pub fn make_common(&mut self) {
        self.royal = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decree_new() {
        let d = RealmDecree::new("d1", "Title", "Content");
        assert_eq!(d.id, "d1");
    }

    #[test]
    fn test_decree_builder() {
        let d = RealmDecree::new("d1", "Title", "Content")
            .order(1);
        assert_eq!(d.order, 1);
    }

    #[test]
    fn test_decree_royal() {
        let mut d = RealmDecree::new("d1", "Title", "Content");
        d.make_common();
        assert!(!d.royal);
        d.make_royal();
        assert!(d.royal);
    }
}
