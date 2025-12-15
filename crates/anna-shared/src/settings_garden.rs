// v0.0.768: Settings Garden (Phase 344)
// Cultivated garden for settings horticulture

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Garden type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GardenType {
    /// Flower garden
    #[default]
    Flower,
    /// Vegetable garden
    Vegetable,
    /// Herb garden
    Herb,
    /// Rock garden
    Rock,
}

impl std::fmt::Display for GardenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Flower => write!(f, "flower"),
            Self::Vegetable => write!(f, "vegetable"),
            Self::Herb => write!(f, "herb"),
            Self::Rock => write!(f, "rock"),
        }
    }
}

/// Garden status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GardenStatus {
    /// Planned status
    #[default]
    Planned,
    /// Planted status
    Planted,
    /// Growing status
    Growing,
    /// Blooming status
    Blooming,
}

impl std::fmt::Display for GardenStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Planned => write!(f, "planned"),
            Self::Planted => write!(f, "planted"),
            Self::Growing => write!(f, "growing"),
            Self::Blooming => write!(f, "blooming"),
        }
    }
}

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

/// Settings garden
#[derive(Debug, Clone, Default)]
pub struct SettingsGarden {
    /// Config
    config: GardenConfig,
    /// Plants
    plants: Vec<GardenPlant>,
    /// Gardeners
    gardeners: Vec<GardenGardener>,
    /// Stats
    stats: GardenStats,
}

impl SettingsGarden {
    /// Create new garden system
    pub fn new(config: GardenConfig) -> Self {
        Self {
            config,
            plants: Vec::new(),
            gardeners: Vec::new(),
            stats: GardenStats::default(),
        }
    }

    /// Add plant
    pub fn add_plant(&mut self, plant: GardenPlant) -> bool {
        if self.plants.len() >= self.config.max_plants {
            return false;
        }
        self.plants.push(plant);
        self.update_stats();
        true
    }

    /// Get plant
    pub fn get_plant(&self, id: &str) -> Option<&GardenPlant> {
        self.plants.iter().find(|p| p.id == id)
    }

    /// Get plant mut
    pub fn get_plant_mut(&mut self, id: &str) -> Option<&mut GardenPlant> {
        self.plants.iter_mut().find(|p| p.id == id)
    }

    /// Add gardener
    pub fn add_gardener(&mut self, gardener: GardenGardener) {
        self.gardeners.push(gardener);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.plants, self.config.garden_type);
    }

    /// Get stats
    pub fn stats(&self) -> &GardenStats {
        &self.stats
    }

    /// Plant count
    pub fn plant_count(&self) -> usize {
        self.plants.len()
    }
}

/// Garden registry
#[derive(Debug, Clone, Default)]
pub struct GardenRegistry {
    /// Gardens by ID
    gardens: HashMap<String, SettingsGarden>,
}

impl GardenRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register garden
    pub fn register(&mut self, id: impl Into<String>, garden: SettingsGarden) {
        self.gardens.insert(id.into(), garden);
    }

    /// Unregister garden
    pub fn unregister(&mut self, id: &str) -> bool {
        self.gardens.remove(id).is_some()
    }

    /// Get garden
    pub fn get(&self, id: &str) -> Option<&SettingsGarden> {
        self.gardens.get(id)
    }

    /// Get garden mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsGarden> {
        self.gardens.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.gardens.len()
    }
}

/// Format garden registry
pub fn format_garden_registry(registry: &GardenRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Garden Registry:\n");
    output.push_str(&format!("  Gardens: {}\n", registry.count()));
    output
}

/// Check if query is about garden
pub fn is_garden_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings garden") || lower.contains("garden settings") || lower.contains("cultivated garden")
}

/// Fun fact about garden
pub fn garden_fun_fact() -> &'static str {
    "Anna's settings garden cultivates horticulture boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_garden_type_display() {
        assert_eq!(format!("{}", GardenType::Flower), "flower");
        assert_eq!(format!("{}", GardenType::Vegetable), "vegetable");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", GardenStatus::Planned), "planned");
        assert_eq!(format!("{}", GardenStatus::Blooming), "blooming");
    }

    #[test]
    fn test_config_new() {
        let c = GardenConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = GardenConfig::new("test")
            .garden_type(GardenType::Herb)
            .status(GardenStatus::Growing);
        assert_eq!(c.garden_type, GardenType::Herb);
        assert_eq!(c.status, GardenStatus::Growing);
    }

    #[test]
    fn test_plant_new() {
        let p = GardenPlant::new("p1", "Title", "Content");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_plant_builder() {
        let p = GardenPlant::new("p1", "Title", "Content")
            .bed(1);
        assert_eq!(p.bed, 1);
    }

    #[test]
    fn test_plant_thriving() {
        let mut p = GardenPlant::new("p1", "Title", "Content");
        p.make_wilting();
        assert!(!p.thriving);
        p.make_thriving();
        assert!(p.thriving);
    }

    #[test]
    fn test_gardener_new() {
        let g = GardenGardener::new("key", "name", "p1");
        assert_eq!(g.plant_id, "p1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = GardenStats::default();
        let plant = GardenPlant::new("p1", "Title", "Content");
        s.update(&[plant], GardenType::Flower);
        assert_eq!(s.total_plants, 1);
        assert_eq!(s.thriving, 1);
    }

    #[test]
    fn test_garden_new() {
        let g = SettingsGarden::new(GardenConfig::default());
        assert_eq!(g.plant_count(), 0);
    }

    #[test]
    fn test_garden_add_plant() {
        let mut g = SettingsGarden::new(GardenConfig::default());
        g.add_plant(GardenPlant::new("p1", "Title", "Content"));
        assert_eq!(g.plant_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = GardenRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = GardenRegistry::new();
        r.register("g1", SettingsGarden::new(GardenConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_garden_query() {
        assert!(is_garden_query("settings garden"));
        assert!(!is_garden_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = garden_fun_fact();
        assert!(fact.contains("garden"));
    }
}
