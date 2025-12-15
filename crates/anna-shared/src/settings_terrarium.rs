// v0.0.777: Settings Terrarium (Phase 353)
// Enclosed terrarium for settings miniature ecosystem

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Terrarium type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TerrariumType {
    /// Desert terrarium
    #[default]
    Desert,
    /// Tropical terrarium
    Tropical,
    /// Woodland terrarium
    Woodland,
    /// Moss terrarium
    Moss,
}

impl std::fmt::Display for TerrariumType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Desert => write!(f, "desert"),
            Self::Tropical => write!(f, "tropical"),
            Self::Woodland => write!(f, "woodland"),
            Self::Moss => write!(f, "moss"),
        }
    }
}

/// Terrarium status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TerrariumStatus {
    /// Building status
    #[default]
    Building,
    /// Sealed status
    Sealed,
    /// Mature status
    Mature,
    /// Renewing status
    Renewing,
}

impl std::fmt::Display for TerrariumStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Building => write!(f, "building"),
            Self::Sealed => write!(f, "sealed"),
            Self::Mature => write!(f, "mature"),
            Self::Renewing => write!(f, "renewing"),
        }
    }
}

/// Terrarium config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrariumConfig {
    /// Name
    pub name: String,
    /// Terrarium type
    pub terrarium_type: TerrariumType,
    /// Status
    pub status: TerrariumStatus,
    /// Max plants
    pub max_plants: usize,
}

impl TerrariumConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            terrarium_type: TerrariumType::Desert,
            status: TerrariumStatus::Building,
            max_plants: 100,
        }
    }

    /// Set type
    pub fn terrarium_type(mut self, tt: TerrariumType) -> Self {
        self.terrarium_type = tt;
        self
    }

    /// Set status
    pub fn status(mut self, s: TerrariumStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max plants
    pub fn max_plants(mut self, max: usize) -> Self {
        self.max_plants = max;
        self
    }
}

impl Default for TerrariumConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

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

/// Terrarium stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TerrariumStats {
    /// Total plants
    pub total_plants: usize,
    /// Established plants
    pub established: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl TerrariumStats {
    /// Update from plants
    pub fn update(&mut self, plants: &[TerrariumPlant], terrarium_type: TerrariumType) {
        self.total_plants = plants.len();
        self.established = plants.iter().filter(|p| p.established).count();
        *self.by_type.entry(terrarium_type.to_string()).or_insert(0) += 1;
    }

    /// Establishment rate
    pub fn establishment_rate(&self) -> f64 {
        if self.total_plants == 0 { 0.0 } else { self.established as f64 / self.total_plants as f64 * 100.0 }
    }
}

/// Settings terrarium
#[derive(Debug, Clone, Default)]
pub struct SettingsTerrarium {
    /// Config
    config: TerrariumConfig,
    /// Plants
    plants: Vec<TerrariumPlant>,
    /// Creators
    creators: Vec<TerrariumCreator>,
    /// Stats
    stats: TerrariumStats,
}

impl SettingsTerrarium {
    /// Create new terrarium system
    pub fn new(config: TerrariumConfig) -> Self {
        Self {
            config,
            plants: Vec::new(),
            creators: Vec::new(),
            stats: TerrariumStats::default(),
        }
    }

    /// Add plant
    pub fn add_plant(&mut self, plant: TerrariumPlant) -> bool {
        if self.plants.len() >= self.config.max_plants {
            return false;
        }
        self.plants.push(plant);
        self.update_stats();
        true
    }

    /// Get plant
    pub fn get_plant(&self, id: &str) -> Option<&TerrariumPlant> {
        self.plants.iter().find(|p| p.id == id)
    }

    /// Get plant mut
    pub fn get_plant_mut(&mut self, id: &str) -> Option<&mut TerrariumPlant> {
        self.plants.iter_mut().find(|p| p.id == id)
    }

    /// Add creator
    pub fn add_creator(&mut self, creator: TerrariumCreator) {
        self.creators.push(creator);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.plants, self.config.terrarium_type);
    }

    /// Get stats
    pub fn stats(&self) -> &TerrariumStats {
        &self.stats
    }

    /// Plant count
    pub fn plant_count(&self) -> usize {
        self.plants.len()
    }
}

/// Terrarium registry
#[derive(Debug, Clone, Default)]
pub struct TerrariumRegistry {
    /// Terrariums by ID
    terrariums: HashMap<String, SettingsTerrarium>,
}

impl TerrariumRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register terrarium
    pub fn register(&mut self, id: impl Into<String>, terrarium: SettingsTerrarium) {
        self.terrariums.insert(id.into(), terrarium);
    }

    /// Unregister terrarium
    pub fn unregister(&mut self, id: &str) -> bool {
        self.terrariums.remove(id).is_some()
    }

    /// Get terrarium
    pub fn get(&self, id: &str) -> Option<&SettingsTerrarium> {
        self.terrariums.get(id)
    }

    /// Get terrarium mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsTerrarium> {
        self.terrariums.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.terrariums.len()
    }
}

/// Format terrarium registry
pub fn format_terrarium_registry(registry: &TerrariumRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Terrarium Registry:\n");
    output.push_str(&format!("  Terrariums: {}\n", registry.count()));
    output
}

/// Check if query is about terrarium
pub fn is_terrarium_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings terrarium") || lower.contains("terrarium settings") || lower.contains("enclosed terrarium")
}

/// Fun fact about terrarium
pub fn terrarium_fun_fact() -> &'static str {
    "Anna's settings terrarium creates miniature ecosystem boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrarium_type_display() {
        assert_eq!(format!("{}", TerrariumType::Desert), "desert");
        assert_eq!(format!("{}", TerrariumType::Tropical), "tropical");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", TerrariumStatus::Building), "building");
        assert_eq!(format!("{}", TerrariumStatus::Mature), "mature");
    }

    #[test]
    fn test_config_new() {
        let c = TerrariumConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = TerrariumConfig::new("test")
            .terrarium_type(TerrariumType::Woodland)
            .status(TerrariumStatus::Sealed);
        assert_eq!(c.terrarium_type, TerrariumType::Woodland);
        assert_eq!(c.status, TerrariumStatus::Sealed);
    }

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

    #[test]
    fn test_stats_update() {
        let mut s = TerrariumStats::default();
        let plant = TerrariumPlant::new("p1", "Title", "Content");
        s.update(&[plant], TerrariumType::Desert);
        assert_eq!(s.total_plants, 1);
        assert_eq!(s.established, 1);
    }

    #[test]
    fn test_terrarium_new() {
        let t = SettingsTerrarium::new(TerrariumConfig::default());
        assert_eq!(t.plant_count(), 0);
    }

    #[test]
    fn test_terrarium_add_plant() {
        let mut t = SettingsTerrarium::new(TerrariumConfig::default());
        t.add_plant(TerrariumPlant::new("p1", "Title", "Content"));
        assert_eq!(t.plant_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = TerrariumRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = TerrariumRegistry::new();
        r.register("t1", SettingsTerrarium::new(TerrariumConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_terrarium_query() {
        assert!(is_terrarium_query("settings terrarium"));
        assert!(!is_terrarium_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = terrarium_fun_fact();
        assert!(fact.contains("terrarium"));
    }
}
