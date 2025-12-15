// v0.0.780: Settings Butterfly (Phase 356)
// Butterfly house for settings lepidopterology

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Butterfly type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ButterflyType {
    /// Tropical butterfly
    #[default]
    Tropical,
    /// Native butterfly
    Native,
    /// Monarch butterfly
    Monarch,
    /// Conservation butterfly
    Conservation,
}

impl std::fmt::Display for ButterflyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tropical => write!(f, "tropical"),
            Self::Native => write!(f, "native"),
            Self::Monarch => write!(f, "monarch"),
            Self::Conservation => write!(f, "conservation"),
        }
    }
}

/// Butterfly status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ButterflyStatus {
    /// Active status
    #[default]
    Active,
    /// Emerging status
    Emerging,
    /// Breeding status
    Breeding,
    /// Migrating status
    Migrating,
}

impl std::fmt::Display for ButterflyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Emerging => write!(f, "emerging"),
            Self::Breeding => write!(f, "breeding"),
            Self::Migrating => write!(f, "migrating"),
        }
    }
}

/// Butterfly config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButterflyConfig {
    /// Name
    pub name: String,
    /// Butterfly type
    pub butterfly_type: ButterflyType,
    /// Status
    pub status: ButterflyStatus,
    /// Max specimens
    pub max_specimens: usize,
}

impl ButterflyConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            butterfly_type: ButterflyType::Tropical,
            status: ButterflyStatus::Active,
            max_specimens: 100,
        }
    }

    /// Set type
    pub fn butterfly_type(mut self, bt: ButterflyType) -> Self {
        self.butterfly_type = bt;
        self
    }

    /// Set status
    pub fn status(mut self, s: ButterflyStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max specimens
    pub fn max_specimens(mut self, max: usize) -> Self {
        self.max_specimens = max;
        self
    }
}

impl Default for ButterflyConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

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

/// Butterfly stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ButterflyStats {
    /// Total specimens
    pub total_specimens: usize,
    /// Flying specimens
    pub flying: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ButterflyStats {
    /// Update from specimens
    pub fn update(&mut self, specimens: &[ButterflySpecimen], butterfly_type: ButterflyType) {
        self.total_specimens = specimens.len();
        self.flying = specimens.iter().filter(|s| s.flying).count();
        *self.by_type.entry(butterfly_type.to_string()).or_insert(0) += 1;
    }

    /// Flight rate
    pub fn flight_rate(&self) -> f64 {
        if self.total_specimens == 0 { 0.0 } else { self.flying as f64 / self.total_specimens as f64 * 100.0 }
    }
}

/// Settings butterfly
#[derive(Debug, Clone, Default)]
pub struct SettingsButterfly {
    /// Config
    config: ButterflyConfig,
    /// Specimens
    specimens: Vec<ButterflySpecimen>,
    /// Curators
    curators: Vec<ButterflyCurator>,
    /// Stats
    stats: ButterflyStats,
}

impl SettingsButterfly {
    /// Create new butterfly system
    pub fn new(config: ButterflyConfig) -> Self {
        Self {
            config,
            specimens: Vec::new(),
            curators: Vec::new(),
            stats: ButterflyStats::default(),
        }
    }

    /// Add specimen
    pub fn add_specimen(&mut self, specimen: ButterflySpecimen) -> bool {
        if self.specimens.len() >= self.config.max_specimens {
            return false;
        }
        self.specimens.push(specimen);
        self.update_stats();
        true
    }

    /// Get specimen
    pub fn get_specimen(&self, id: &str) -> Option<&ButterflySpecimen> {
        self.specimens.iter().find(|s| s.id == id)
    }

    /// Get specimen mut
    pub fn get_specimen_mut(&mut self, id: &str) -> Option<&mut ButterflySpecimen> {
        self.specimens.iter_mut().find(|s| s.id == id)
    }

    /// Add curator
    pub fn add_curator(&mut self, curator: ButterflyCurator) {
        self.curators.push(curator);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.specimens, self.config.butterfly_type);
    }

    /// Get stats
    pub fn stats(&self) -> &ButterflyStats {
        &self.stats
    }

    /// Specimen count
    pub fn specimen_count(&self) -> usize {
        self.specimens.len()
    }
}

/// Butterfly registry
#[derive(Debug, Clone, Default)]
pub struct ButterflyRegistry {
    /// Butterflies by ID
    butterflies: HashMap<String, SettingsButterfly>,
}

impl ButterflyRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register butterfly
    pub fn register(&mut self, id: impl Into<String>, butterfly: SettingsButterfly) {
        self.butterflies.insert(id.into(), butterfly);
    }

    /// Unregister butterfly
    pub fn unregister(&mut self, id: &str) -> bool {
        self.butterflies.remove(id).is_some()
    }

    /// Get butterfly
    pub fn get(&self, id: &str) -> Option<&SettingsButterfly> {
        self.butterflies.get(id)
    }

    /// Get butterfly mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsButterfly> {
        self.butterflies.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.butterflies.len()
    }
}

/// Format butterfly registry
pub fn format_butterfly_registry(registry: &ButterflyRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Butterfly Registry:\n");
    output.push_str(&format!("  Butterflies: {}\n", registry.count()));
    output
}

/// Check if query is about butterfly
pub fn is_butterfly_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings butterfly") || lower.contains("butterfly settings") || lower.contains("butterfly house")
}

/// Fun fact about butterfly
pub fn butterfly_fun_fact() -> &'static str {
    "Anna's settings butterfly flutters with lepidopterology boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_butterfly_type_display() {
        assert_eq!(format!("{}", ButterflyType::Tropical), "tropical");
        assert_eq!(format!("{}", ButterflyType::Native), "native");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ButterflyStatus::Active), "active");
        assert_eq!(format!("{}", ButterflyStatus::Emerging), "emerging");
    }

    #[test]
    fn test_config_new() {
        let c = ButterflyConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ButterflyConfig::new("test")
            .butterfly_type(ButterflyType::Monarch)
            .status(ButterflyStatus::Breeding);
        assert_eq!(c.butterfly_type, ButterflyType::Monarch);
        assert_eq!(c.status, ButterflyStatus::Breeding);
    }

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

    #[test]
    fn test_stats_update() {
        let mut s = ButterflyStats::default();
        let specimen = ButterflySpecimen::new("s1", "Title", "Content");
        s.update(&[specimen], ButterflyType::Tropical);
        assert_eq!(s.total_specimens, 1);
        assert_eq!(s.flying, 1);
    }

    #[test]
    fn test_butterfly_new() {
        let b = SettingsButterfly::new(ButterflyConfig::default());
        assert_eq!(b.specimen_count(), 0);
    }

    #[test]
    fn test_butterfly_add_specimen() {
        let mut b = SettingsButterfly::new(ButterflyConfig::default());
        b.add_specimen(ButterflySpecimen::new("s1", "Title", "Content"));
        assert_eq!(b.specimen_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = ButterflyRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ButterflyRegistry::new();
        r.register("b1", SettingsButterfly::new(ButterflyConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_butterfly_query() {
        assert!(is_butterfly_query("settings butterfly"));
        assert!(!is_butterfly_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = butterfly_fun_fact();
        assert!(fact.contains("butterfly"));
    }
}
