// v0.0.775: Settings Aquarium - Inhabitant Module (Phase 351)
// Aquarium inhabitants and aquarists

use serde::{Deserialize, Serialize};

/// Aquarium inhabitant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AquariumInhabitant {
    /// Inhabitant ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Tank number
    pub tank: u32,
    /// Healthy
    pub healthy: bool,
}

impl AquariumInhabitant {
    /// Create new inhabitant
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            tank: 0,
            healthy: true,
        }
    }

    /// Set tank
    pub fn tank(mut self, t: u32) -> Self {
        self.tank = t;
        self
    }

    /// Make healthy
    pub fn make_healthy(&mut self) {
        self.healthy = true;
    }

    /// Make sick
    pub fn make_sick(&mut self) {
        self.healthy = false;
    }
}

/// Aquarium aquarist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AquariumAquarist {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Inhabitant ID
    pub inhabitant_id: String,
}

impl AquariumAquarist {
    /// Create new aquarist
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
        let i = AquariumInhabitant::new("i1", "Title", "Content");
        assert_eq!(i.id, "i1");
    }

    #[test]
    fn test_inhabitant_builder() {
        let i = AquariumInhabitant::new("i1", "Title", "Content")
            .tank(1);
        assert_eq!(i.tank, 1);
    }

    #[test]
    fn test_inhabitant_healthy() {
        let mut i = AquariumInhabitant::new("i1", "Title", "Content");
        i.make_sick();
        assert!(!i.healthy);
        i.make_healthy();
        assert!(i.healthy);
    }

    #[test]
    fn test_aquarist_new() {
        let a = AquariumAquarist::new("key", "name", "i1");
        assert_eq!(a.inhabitant_id, "i1");
    }
}
