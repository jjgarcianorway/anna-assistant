// v0.0.768: Settings Garden (Phase 344)
// Garden data models

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{GardenType, GardenStatus};

/// Garden config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GardenConfig {
    /// Name
    pub name: String,
    /// Garden type
    pub garden_type: GardenType,
    /// Status
    pub status: GardenStatus,
    /// Max plants
    pub max_plants: usize,
}

impl GardenConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            garden_type: GardenType::Flower,
            status: GardenStatus::Planned,
            max_plants: 100,
        }
    }

    /// Set type
    pub fn garden_type(mut self, gt: GardenType) -> Self {
        self.garden_type = gt;
        self
    }

    /// Set status
    pub fn status(mut self, s: GardenStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max plants
    pub fn max_plants(mut self, max: usize) -> Self {
        self.max_plants = max;
        self
    }
}

impl Default for GardenConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Garden plant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GardenPlant {
    /// Plant ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Bed number
    pub bed: u32,
    /// Thriving
    pub thriving: bool,
}

impl GardenPlant {
    /// Create new plant
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            bed: 0,
            thriving: true,
        }
    }

    /// Set bed
    pub fn bed(mut self, b: u32) -> Self {
        self.bed = b;
        self
    }

    /// Make thriving
    pub fn make_thriving(&mut self) {
        self.thriving = true;
    }

    /// Make wilting
    pub fn make_wilting(&mut self) {
        self.thriving = false;
    }
}

/// Garden gardener
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GardenGardener {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Plant ID
    pub plant_id: String,
}

impl GardenGardener {
    /// Create new gardener
    pub fn new(key: impl Into<String>, name: impl Into<String>, plant_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            plant_id: plant_id.into(),
        }
    }
}

/// Garden stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GardenStats {
    /// Total plants
    pub total_plants: usize,
    /// Thriving plants
    pub thriving: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl GardenStats {
    /// Update from plants
    pub fn update(&mut self, plants: &[GardenPlant], garden_type: GardenType) {
        self.total_plants = plants.len();
        self.thriving = plants.iter().filter(|p| p.thriving).count();
        *self.by_type.entry(garden_type.to_string()).or_insert(0) += 1;
    }

    /// Thriving rate
    pub fn thriving_rate(&self) -> f64 {
        if self.total_plants == 0 { 0.0 } else { self.thriving as f64 / self.total_plants as f64 * 100.0 }
    }
}
