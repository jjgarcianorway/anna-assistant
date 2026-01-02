// v0.0.780: Settings Butterfly (Phase 356)
// Butterfly specimens and curators

use serde::{Deserialize, Serialize};

/// Butterfly specimen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButterflySpecimen {
    /// Specimen ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Flight zone
    pub zone: u32,
    /// Flying
    pub flying: bool,
}

impl ButterflySpecimen {
    /// Create new specimen
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            zone: 0,
            flying: true,
        }
    }

    /// Set zone
    pub fn zone(mut self, z: u32) -> Self {
        self.zone = z;
        self
    }

    /// Make flying
    pub fn make_flying(&mut self) {
        self.flying = true;
    }

    /// Make resting
    pub fn make_resting(&mut self) {
        self.flying = false;
    }
}

/// Butterfly curator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButterflyCurator {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Specimen ID
    pub specimen_id: String,
}

impl ButterflyCurator {
    /// Create new curator
    pub fn new(key: impl Into<String>, name: impl Into<String>, specimen_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            specimen_id: specimen_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specimen_new() {
        let s = ButterflySpecimen::new("s1", "Title", "Content");
        assert_eq!(s.id, "s1");
    }

    #[test]
    fn test_specimen_builder() {
        let s = ButterflySpecimen::new("s1", "Title", "Content")
            .zone(1);
        assert_eq!(s.zone, 1);
    }

    #[test]
    fn test_specimen_flying() {
        let mut s = ButterflySpecimen::new("s1", "Title", "Content");
        s.make_resting();
        assert!(!s.flying);
        s.make_flying();
        assert!(s.flying);
    }

    #[test]
    fn test_curator_new() {
        let c = ButterflyCurator::new("key", "name", "s1");
        assert_eq!(c.specimen_id, "s1");
    }
}
