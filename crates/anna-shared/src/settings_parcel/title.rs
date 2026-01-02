// v0.0.757: Settings Parcel - Title (Phase 333)

use serde::{Deserialize, Serialize};

/// Parcel title
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParcelTitle {
    /// Title ID
    pub id: String,
    /// Title name
    pub title: String,
    /// Content
    pub content: String,
    /// Section number
    pub section: u32,
    /// Cleared
    pub cleared: bool,
}

impl ParcelTitle {
    /// Create new title
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            section: 0,
            cleared: true,
        }
    }

    /// Set section
    pub fn section(mut self, s: u32) -> Self {
        self.section = s;
        self
    }

    /// Make cleared
    pub fn make_cleared(&mut self) {
        self.cleared = true;
    }

    /// Make clouded
    pub fn make_clouded(&mut self) {
        self.cleared = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title_new() {
        let t = ParcelTitle::new("t1", "Title", "Content");
        assert_eq!(t.id, "t1");
    }

    #[test]
    fn test_title_builder() {
        let t = ParcelTitle::new("t1", "Title", "Content")
            .section(1);
        assert_eq!(t.section, 1);
    }

    #[test]
    fn test_title_cleared() {
        let mut t = ParcelTitle::new("t1", "Title", "Content");
        t.make_clouded();
        assert!(!t.cleared);
        t.make_cleared();
        assert!(t.cleared);
    }
}
