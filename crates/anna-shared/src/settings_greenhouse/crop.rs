// v0.0.770: Settings Greenhouse - Crop Module
// Greenhouse crop and grower types

use serde::{Deserialize, Serialize};

/// Greenhouse crop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreenhouseCrop {
    /// Crop ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Zone number
    pub zone: u32,
    /// Flourishing
    pub flourishing: bool,
}

impl GreenhouseCrop {
    /// Create new crop
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            zone: 0,
            flourishing: true,
        }
    }

    /// Set zone
    pub fn zone(mut self, z: u32) -> Self {
        self.zone = z;
        self
    }

    /// Make flourishing
    pub fn make_flourishing(&mut self) {
        self.flourishing = true;
    }

    /// Make struggling
    pub fn make_struggling(&mut self) {
        self.flourishing = false;
    }
}

/// Greenhouse grower
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreenhouseGrower {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Crop ID
    pub crop_id: String,
}

impl GreenhouseGrower {
    /// Create new grower
    pub fn new(key: impl Into<String>, name: impl Into<String>, crop_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            crop_id: crop_id.into(),
        }
    }
}
