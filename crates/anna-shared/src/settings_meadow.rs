// v0.0.763: Settings Meadow (Phase 339)
// Grassland meadow for settings grazing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Meadow type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MeadowType {
    /// Hay meadow
    #[default]
    Hay,
    /// Water meadow
    Water,
    /// Alpine meadow
    Alpine,
    /// Wildflower meadow
    Wildflower,
}

impl std::fmt::Display for MeadowType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hay => write!(f, "hay"),
            Self::Water => write!(f, "water"),
            Self::Alpine => write!(f, "alpine"),
            Self::Wildflower => write!(f, "wildflower"),
        }
    }
}

/// Meadow status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MeadowStatus {
    /// Resting status
    #[default]
    Resting,
    /// Grazing status
    Grazing,
    /// Mowing status
    Mowing,
    /// Recovering status
    Recovering,
}

impl std::fmt::Display for MeadowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resting => write!(f, "resting"),
            Self::Grazing => write!(f, "grazing"),
            Self::Mowing => write!(f, "mowing"),
            Self::Recovering => write!(f, "recovering"),
        }
    }
}

/// Meadow config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeadowConfig {
    /// Name
    pub name: String,
    /// Meadow type
    pub meadow_type: MeadowType,
    /// Status
    pub status: MeadowStatus,
    /// Max grasses
    pub max_grasses: usize,
}

impl MeadowConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            meadow_type: MeadowType::Hay,
            status: MeadowStatus::Resting,
            max_grasses: 100,
        }
    }

    /// Set type
    pub fn meadow_type(mut self, mt: MeadowType) -> Self {
        self.meadow_type = mt;
        self
    }

    /// Set status
    pub fn status(mut self, s: MeadowStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max grasses
    pub fn max_grasses(mut self, max: usize) -> Self {
        self.max_grasses = max;
        self
    }
}

impl Default for MeadowConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Meadow grass
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeadowGrass {
    /// Grass ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Sward number
    pub sward: u32,
    /// Lush
    pub lush: bool,
}

impl MeadowGrass {
    /// Create new grass
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            sward: 0,
            lush: true,
        }
    }

    /// Set sward
    pub fn sward(mut self, s: u32) -> Self {
        self.sward = s;
        self
    }

    /// Make lush
    pub fn make_lush(&mut self) {
        self.lush = true;
    }

    /// Make sparse
    pub fn make_sparse(&mut self) {
        self.lush = false;
    }
}

/// Meadow keeper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeadowKeeper {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Grass ID
    pub grass_id: String,
}

impl MeadowKeeper {
    /// Create new keeper
    pub fn new(key: impl Into<String>, name: impl Into<String>, grass_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            grass_id: grass_id.into(),
        }
    }
}

/// Meadow stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MeadowStats {
    /// Total grasses
    pub total_grasses: usize,
    /// Lush grasses
    pub lush: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl MeadowStats {
    /// Update from grasses
    pub fn update(&mut self, grasses: &[MeadowGrass], meadow_type: MeadowType) {
        self.total_grasses = grasses.len();
        self.lush = grasses.iter().filter(|g| g.lush).count();
        *self.by_type.entry(meadow_type.to_string()).or_insert(0) += 1;
    }

    /// Lush rate
    pub fn lush_rate(&self) -> f64 {
        if self.total_grasses == 0 { 0.0 } else { self.lush as f64 / self.total_grasses as f64 * 100.0 }
    }
}

/// Settings meadow
#[derive(Debug, Clone, Default)]
pub struct SettingsMeadow {
    /// Config
    config: MeadowConfig,
    /// Grasses
    grasses: Vec<MeadowGrass>,
    /// Keepers
    keepers: Vec<MeadowKeeper>,
    /// Stats
    stats: MeadowStats,
}

impl SettingsMeadow {
    /// Create new meadow system
    pub fn new(config: MeadowConfig) -> Self {
        Self {
            config,
            grasses: Vec::new(),
            keepers: Vec::new(),
            stats: MeadowStats::default(),
        }
    }

    /// Add grass
    pub fn add_grass(&mut self, grass: MeadowGrass) -> bool {
        if self.grasses.len() >= self.config.max_grasses {
            return false;
        }
        self.grasses.push(grass);
        self.update_stats();
        true
    }

    /// Get grass
    pub fn get_grass(&self, id: &str) -> Option<&MeadowGrass> {
        self.grasses.iter().find(|g| g.id == id)
    }

    /// Get grass mut
    pub fn get_grass_mut(&mut self, id: &str) -> Option<&mut MeadowGrass> {
        self.grasses.iter_mut().find(|g| g.id == id)
    }

    /// Add keeper
    pub fn add_keeper(&mut self, keeper: MeadowKeeper) {
        self.keepers.push(keeper);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.grasses, self.config.meadow_type);
    }

    /// Get stats
    pub fn stats(&self) -> &MeadowStats {
        &self.stats
    }

    /// Grass count
    pub fn grass_count(&self) -> usize {
        self.grasses.len()
    }
}

/// Meadow registry
#[derive(Debug, Clone, Default)]
pub struct MeadowRegistry {
    /// Meadows by ID
    meadows: HashMap<String, SettingsMeadow>,
}

impl MeadowRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register meadow
    pub fn register(&mut self, id: impl Into<String>, meadow: SettingsMeadow) {
        self.meadows.insert(id.into(), meadow);
    }

    /// Unregister meadow
    pub fn unregister(&mut self, id: &str) -> bool {
        self.meadows.remove(id).is_some()
    }

    /// Get meadow
    pub fn get(&self, id: &str) -> Option<&SettingsMeadow> {
        self.meadows.get(id)
    }

    /// Get meadow mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsMeadow> {
        self.meadows.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.meadows.len()
    }
}

/// Format meadow registry
pub fn format_meadow_registry(registry: &MeadowRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Meadow Registry:\n");
    output.push_str(&format!("  Meadows: {}\n", registry.count()));
    output
}

/// Check if query is about meadow
pub fn is_meadow_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings meadow") || lower.contains("meadow settings") || lower.contains("grassland meadow")
}

/// Fun fact about meadow
pub fn meadow_fun_fact() -> &'static str {
    "Anna's settings meadow establishes grazing boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meadow_type_display() {
        assert_eq!(format!("{}", MeadowType::Hay), "hay");
        assert_eq!(format!("{}", MeadowType::Alpine), "alpine");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", MeadowStatus::Resting), "resting");
        assert_eq!(format!("{}", MeadowStatus::Grazing), "grazing");
    }

    #[test]
    fn test_config_new() {
        let c = MeadowConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = MeadowConfig::new("test")
            .meadow_type(MeadowType::Wildflower)
            .status(MeadowStatus::Mowing);
        assert_eq!(c.meadow_type, MeadowType::Wildflower);
        assert_eq!(c.status, MeadowStatus::Mowing);
    }

    #[test]
    fn test_grass_new() {
        let g = MeadowGrass::new("g1", "Title", "Content");
        assert_eq!(g.id, "g1");
    }

    #[test]
    fn test_grass_builder() {
        let g = MeadowGrass::new("g1", "Title", "Content")
            .sward(1);
        assert_eq!(g.sward, 1);
    }

    #[test]
    fn test_grass_lush() {
        let mut g = MeadowGrass::new("g1", "Title", "Content");
        g.make_sparse();
        assert!(!g.lush);
        g.make_lush();
        assert!(g.lush);
    }

    #[test]
    fn test_keeper_new() {
        let k = MeadowKeeper::new("key", "name", "g1");
        assert_eq!(k.grass_id, "g1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = MeadowStats::default();
        let grass = MeadowGrass::new("g1", "Title", "Content");
        s.update(&[grass], MeadowType::Hay);
        assert_eq!(s.total_grasses, 1);
        assert_eq!(s.lush, 1);
    }

    #[test]
    fn test_meadow_new() {
        let m = SettingsMeadow::new(MeadowConfig::default());
        assert_eq!(m.grass_count(), 0);
    }

    #[test]
    fn test_meadow_add_grass() {
        let mut m = SettingsMeadow::new(MeadowConfig::default());
        m.add_grass(MeadowGrass::new("g1", "Title", "Content"));
        assert_eq!(m.grass_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = MeadowRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = MeadowRegistry::new();
        r.register("m1", SettingsMeadow::new(MeadowConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_meadow_query() {
        assert!(is_meadow_query("settings meadow"));
        assert!(!is_meadow_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = meadow_fun_fact();
        assert!(fact.contains("meadow"));
    }
}
