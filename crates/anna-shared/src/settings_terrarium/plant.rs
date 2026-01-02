// v0.0.777: Settings Terrarium (Phase 353)
// Terrarium plant and creator types

use serde::{Deserialize, Serialize};

/// Terrarium plant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrariumPlant {
    /// Plant ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Layer number
    pub layer: u32,
    /// Established
    pub established: bool,
}

impl TerrariumPlant {
    /// Create new plant
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            layer: 0,
            established: true,
        }
    }

    /// Set layer
    pub fn layer(mut self, l: u32) -> Self {
        self.layer = l;
        self
    }

    /// Make established
    pub fn make_established(&mut self) {
        self.established = true;
    }

    /// Make struggling
    pub fn make_struggling(&mut self) {
        self.established = false;
    }
}

/// Terrarium creator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrariumCreator {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Plant ID
    pub plant_id: String,
}

impl TerrariumCreator {
    /// Create new creator
    pub fn new(key: impl Into<String>, name: impl Into<String>, plant_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            plant_id: plant_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plant_new() {
        let p = TerrariumPlant::new("p1", "Title", "Content");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_plant_builder() {
        let p = TerrariumPlant::new("p1", "Title", "Content")
            .layer(1);
        assert_eq!(p.layer, 1);
    }

    #[test]
    fn test_plant_established() {
        let mut p = TerrariumPlant::new("p1", "Title", "Content");
        p.make_struggling();
        assert!(!p.established);
        p.make_established();
        assert!(p.established);
    }

    #[test]
    fn test_creator_new() {
        let c = TerrariumCreator::new("key", "name", "p1");
        assert_eq!(c.plant_id, "p1");
    }
}
