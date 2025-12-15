// v0.0.770: Settings Greenhouse (Phase 346)
// Controlled greenhouse for settings cultivation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Greenhouse type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GreenhouseType {
    /// Commercial greenhouse
    #[default]
    Commercial,
    /// Hobby greenhouse
    Hobby,
    /// Research greenhouse
    Research,
    /// Tropical greenhouse
    Tropical,
}

impl std::fmt::Display for GreenhouseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Commercial => write!(f, "commercial"),
            Self::Hobby => write!(f, "hobby"),
            Self::Research => write!(f, "research"),
            Self::Tropical => write!(f, "tropical"),
        }
    }
}

/// Greenhouse status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GreenhouseStatus {
    /// Active status
    #[default]
    Active,
    /// Heating status
    Heating,
    /// Cooling status
    Cooling,
    /// Maintenance status
    Maintenance,
}

impl std::fmt::Display for GreenhouseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Heating => write!(f, "heating"),
            Self::Cooling => write!(f, "cooling"),
            Self::Maintenance => write!(f, "maintenance"),
        }
    }
}

/// Greenhouse config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreenhouseConfig {
    /// Name
    pub name: String,
    /// Greenhouse type
    pub greenhouse_type: GreenhouseType,
    /// Status
    pub status: GreenhouseStatus,
    /// Max crops
    pub max_crops: usize,
}

impl GreenhouseConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            greenhouse_type: GreenhouseType::Commercial,
            status: GreenhouseStatus::Active,
            max_crops: 100,
        }
    }

    /// Set type
    pub fn greenhouse_type(mut self, gt: GreenhouseType) -> Self {
        self.greenhouse_type = gt;
        self
    }

    /// Set status
    pub fn status(mut self, s: GreenhouseStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max crops
    pub fn max_crops(mut self, max: usize) -> Self {
        self.max_crops = max;
        self
    }
}

impl Default for GreenhouseConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

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

/// Greenhouse stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GreenhouseStats {
    /// Total crops
    pub total_crops: usize,
    /// Flourishing crops
    pub flourishing: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl GreenhouseStats {
    /// Update from crops
    pub fn update(&mut self, crops: &[GreenhouseCrop], greenhouse_type: GreenhouseType) {
        self.total_crops = crops.len();
        self.flourishing = crops.iter().filter(|c| c.flourishing).count();
        *self.by_type.entry(greenhouse_type.to_string()).or_insert(0) += 1;
    }

    /// Flourishing rate
    pub fn flourishing_rate(&self) -> f64 {
        if self.total_crops == 0 { 0.0 } else { self.flourishing as f64 / self.total_crops as f64 * 100.0 }
    }
}

/// Settings greenhouse
#[derive(Debug, Clone, Default)]
pub struct SettingsGreenhouse {
    /// Config
    config: GreenhouseConfig,
    /// Crops
    crops: Vec<GreenhouseCrop>,
    /// Growers
    growers: Vec<GreenhouseGrower>,
    /// Stats
    stats: GreenhouseStats,
}

impl SettingsGreenhouse {
    /// Create new greenhouse system
    pub fn new(config: GreenhouseConfig) -> Self {
        Self {
            config,
            crops: Vec::new(),
            growers: Vec::new(),
            stats: GreenhouseStats::default(),
        }
    }

    /// Add crop
    pub fn add_crop(&mut self, crop: GreenhouseCrop) -> bool {
        if self.crops.len() >= self.config.max_crops {
            return false;
        }
        self.crops.push(crop);
        self.update_stats();
        true
    }

    /// Get crop
    pub fn get_crop(&self, id: &str) -> Option<&GreenhouseCrop> {
        self.crops.iter().find(|c| c.id == id)
    }

    /// Get crop mut
    pub fn get_crop_mut(&mut self, id: &str) -> Option<&mut GreenhouseCrop> {
        self.crops.iter_mut().find(|c| c.id == id)
    }

    /// Add grower
    pub fn add_grower(&mut self, grower: GreenhouseGrower) {
        self.growers.push(grower);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.crops, self.config.greenhouse_type);
    }

    /// Get stats
    pub fn stats(&self) -> &GreenhouseStats {
        &self.stats
    }

    /// Crop count
    pub fn crop_count(&self) -> usize {
        self.crops.len()
    }
}

/// Greenhouse registry
#[derive(Debug, Clone, Default)]
pub struct GreenhouseRegistry {
    /// Greenhouses by ID
    greenhouses: HashMap<String, SettingsGreenhouse>,
}

impl GreenhouseRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register greenhouse
    pub fn register(&mut self, id: impl Into<String>, greenhouse: SettingsGreenhouse) {
        self.greenhouses.insert(id.into(), greenhouse);
    }

    /// Unregister greenhouse
    pub fn unregister(&mut self, id: &str) -> bool {
        self.greenhouses.remove(id).is_some()
    }

    /// Get greenhouse
    pub fn get(&self, id: &str) -> Option<&SettingsGreenhouse> {
        self.greenhouses.get(id)
    }

    /// Get greenhouse mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsGreenhouse> {
        self.greenhouses.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.greenhouses.len()
    }
}

/// Format greenhouse registry
pub fn format_greenhouse_registry(registry: &GreenhouseRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Greenhouse Registry:\n");
    output.push_str(&format!("  Greenhouses: {}\n", registry.count()));
    output
}

/// Check if query is about greenhouse
pub fn is_greenhouse_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings greenhouse") || lower.contains("greenhouse settings") || lower.contains("controlled greenhouse")
}

/// Fun fact about greenhouse
pub fn greenhouse_fun_fact() -> &'static str {
    "Anna's settings greenhouse cultivates controlled boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greenhouse_type_display() {
        assert_eq!(format!("{}", GreenhouseType::Commercial), "commercial");
        assert_eq!(format!("{}", GreenhouseType::Hobby), "hobby");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", GreenhouseStatus::Active), "active");
        assert_eq!(format!("{}", GreenhouseStatus::Maintenance), "maintenance");
    }

    #[test]
    fn test_config_new() {
        let c = GreenhouseConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = GreenhouseConfig::new("test")
            .greenhouse_type(GreenhouseType::Research)
            .status(GreenhouseStatus::Heating);
        assert_eq!(c.greenhouse_type, GreenhouseType::Research);
        assert_eq!(c.status, GreenhouseStatus::Heating);
    }

    #[test]
    fn test_crop_new() {
        let c = GreenhouseCrop::new("c1", "Title", "Content");
        assert_eq!(c.id, "c1");
    }

    #[test]
    fn test_crop_builder() {
        let c = GreenhouseCrop::new("c1", "Title", "Content")
            .zone(1);
        assert_eq!(c.zone, 1);
    }

    #[test]
    fn test_crop_flourishing() {
        let mut c = GreenhouseCrop::new("c1", "Title", "Content");
        c.make_struggling();
        assert!(!c.flourishing);
        c.make_flourishing();
        assert!(c.flourishing);
    }

    #[test]
    fn test_grower_new() {
        let g = GreenhouseGrower::new("key", "name", "c1");
        assert_eq!(g.crop_id, "c1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = GreenhouseStats::default();
        let crop = GreenhouseCrop::new("c1", "Title", "Content");
        s.update(&[crop], GreenhouseType::Commercial);
        assert_eq!(s.total_crops, 1);
        assert_eq!(s.flourishing, 1);
    }

    #[test]
    fn test_greenhouse_new() {
        let g = SettingsGreenhouse::new(GreenhouseConfig::default());
        assert_eq!(g.crop_count(), 0);
    }

    #[test]
    fn test_greenhouse_add_crop() {
        let mut g = SettingsGreenhouse::new(GreenhouseConfig::default());
        g.add_crop(GreenhouseCrop::new("c1", "Title", "Content"));
        assert_eq!(g.crop_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = GreenhouseRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = GreenhouseRegistry::new();
        r.register("g1", SettingsGreenhouse::new(GreenhouseConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_greenhouse_query() {
        assert!(is_greenhouse_query("settings greenhouse"));
        assert!(!is_greenhouse_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = greenhouse_fun_fact();
        assert!(fact.contains("greenhouse"));
    }
}
