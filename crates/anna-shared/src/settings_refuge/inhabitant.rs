// v0.0.783: Settings Refuge - Inhabitant (Phase 359)
// Refuge inhabitants and wardens

use serde::{Deserialize, Serialize};

/// Refuge inhabitant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefugeInhabitant {
    /// Inhabitant ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Shelter number
    pub shelter: u32,
    /// Safe
    pub safe: bool,
}

impl RefugeInhabitant {
    /// Create new inhabitant
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            shelter: 0,
            safe: true,
        }
    }

    /// Set shelter
    pub fn shelter(mut self, s: u32) -> Self {
        self.shelter = s;
        self
    }

    /// Make safe
    pub fn make_safe(&mut self) {
        self.safe = true;
    }

    /// Make vulnerable
    pub fn make_vulnerable(&mut self) {
        self.safe = false;
    }
}

/// Refuge warden
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefugeWarden {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Inhabitant ID
    pub inhabitant_id: String,
}

impl RefugeWarden {
    /// Create new warden
    pub fn new(key: impl Into<String>, name: impl Into<String>, inhabitant_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            inhabitant_id: inhabitant_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inhabitant_new() {
        let i = RefugeInhabitant::new("i1", "Title", "Content");
        assert_eq!(i.id, "i1");
    }

    #[test]
    fn test_inhabitant_builder() {
        let i = RefugeInhabitant::new("i1", "Title", "Content")
            .shelter(1);
        assert_eq!(i.shelter, 1);
    }

    #[test]
    fn test_inhabitant_safety() {
        let mut i = RefugeInhabitant::new("i1", "Title", "Content");
        i.make_vulnerable();
        assert!(!i.safe);
        i.make_safe();
        assert!(i.safe);
    }

    #[test]
    fn test_warden_new() {
        let w = RefugeWarden::new("key", "name", "i1");
        assert_eq!(w.inhabitant_id, "i1");
    }
}
