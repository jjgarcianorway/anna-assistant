// v0.0.769: Settings Nursery - Seedling (Phase 345)

use serde::{Deserialize, Serialize};

/// Nursery seedling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NurserySeedling {
    /// Seedling ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Tray number
    pub tray: u32,
    /// Viable
    pub viable: bool,
}

impl NurserySeedling {
    /// Create new seedling
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            tray: 0,
            viable: true,
        }
    }

    /// Set tray
    pub fn tray(mut self, t: u32) -> Self {
        self.tray = t;
        self
    }

    /// Make viable
    pub fn make_viable(&mut self) {
        self.viable = true;
    }

    /// Make unviable
    pub fn make_unviable(&mut self) {
        self.viable = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seedling_new() {
        let s = NurserySeedling::new("s1", "Title", "Content");
        assert_eq!(s.id, "s1");
    }

    #[test]
    fn test_seedling_builder() {
        let s = NurserySeedling::new("s1", "Title", "Content")
            .tray(1);
        assert_eq!(s.tray, 1);
    }

    #[test]
    fn test_seedling_viable() {
        let mut s = NurserySeedling::new("s1", "Title", "Content");
        s.make_unviable();
        assert!(!s.viable);
        s.make_viable();
        assert!(s.viable);
    }
}
