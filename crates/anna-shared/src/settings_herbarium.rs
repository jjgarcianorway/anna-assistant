// v0.0.774: Settings Herbarium (Phase 350)
// Plant herbarium for settings taxonomy

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Herbarium type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HerbariumType {
    /// University herbarium
    #[default]
    University,
    /// Museum herbarium
    Museum,
    /// National herbarium
    National,
    /// Private herbarium
    Private,
}

impl std::fmt::Display for HerbariumType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::University => write!(f, "university"),
            Self::Museum => write!(f, "museum"),
            Self::National => write!(f, "national"),
            Self::Private => write!(f, "private"),
        }
    }
}

/// Herbarium status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HerbariumStatus {
    /// Active status
    #[default]
    Active,
    /// Cataloging status
    Cataloging,
    /// Digitizing status
    Digitizing,
    /// Archiving status
    Archiving,
}

impl std::fmt::Display for HerbariumStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Cataloging => write!(f, "cataloging"),
            Self::Digitizing => write!(f, "digitizing"),
            Self::Archiving => write!(f, "archiving"),
        }
    }
}

/// Herbarium config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HerbariumConfig {
    /// Name
    pub name: String,
    /// Herbarium type
    pub herbarium_type: HerbariumType,
    /// Status
    pub status: HerbariumStatus,
    /// Max specimens
    pub max_specimens: usize,
}

impl HerbariumConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            herbarium_type: HerbariumType::University,
            status: HerbariumStatus::Active,
            max_specimens: 100,
        }
    }

    /// Set type
    pub fn herbarium_type(mut self, ht: HerbariumType) -> Self {
        self.herbarium_type = ht;
        self
    }

    /// Set status
    pub fn status(mut self, s: HerbariumStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max specimens
    pub fn max_specimens(mut self, max: usize) -> Self {
        self.max_specimens = max;
        self
    }
}

impl Default for HerbariumConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Herbarium specimen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HerbariumSpecimen {
    /// Specimen ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Cabinet number
    pub cabinet: u32,
    /// Mounted
    pub mounted: bool,
}

impl HerbariumSpecimen {
    /// Create new specimen
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            cabinet: 0,
            mounted: true,
        }
    }

    /// Set cabinet
    pub fn cabinet(mut self, c: u32) -> Self {
        self.cabinet = c;
        self
    }

    /// Make mounted
    pub fn make_mounted(&mut self) {
        self.mounted = true;
    }

    /// Make unmounted
    pub fn make_unmounted(&mut self) {
        self.mounted = false;
    }
}

/// Herbarium taxonomist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HerbariumTaxonomist {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Specimen ID
    pub specimen_id: String,
}

impl HerbariumTaxonomist {
    /// Create new taxonomist
    pub fn new(key: impl Into<String>, name: impl Into<String>, specimen_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            specimen_id: specimen_id.into(),
        }
    }
}

/// Herbarium stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HerbariumStats {
    /// Total specimens
    pub total_specimens: usize,
    /// Mounted specimens
    pub mounted: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl HerbariumStats {
    /// Update from specimens
    pub fn update(&mut self, specimens: &[HerbariumSpecimen], herbarium_type: HerbariumType) {
        self.total_specimens = specimens.len();
        self.mounted = specimens.iter().filter(|s| s.mounted).count();
        *self.by_type.entry(herbarium_type.to_string()).or_insert(0) += 1;
    }

    /// Mount rate
    pub fn mount_rate(&self) -> f64 {
        if self.total_specimens == 0 { 0.0 } else { self.mounted as f64 / self.total_specimens as f64 * 100.0 }
    }
}

/// Settings herbarium
#[derive(Debug, Clone, Default)]
pub struct SettingsHerbarium {
    /// Config
    config: HerbariumConfig,
    /// Specimens
    specimens: Vec<HerbariumSpecimen>,
    /// Taxonomists
    taxonomists: Vec<HerbariumTaxonomist>,
    /// Stats
    stats: HerbariumStats,
}

impl SettingsHerbarium {
    /// Create new herbarium system
    pub fn new(config: HerbariumConfig) -> Self {
        Self {
            config,
            specimens: Vec::new(),
            taxonomists: Vec::new(),
            stats: HerbariumStats::default(),
        }
    }

    /// Add specimen
    pub fn add_specimen(&mut self, specimen: HerbariumSpecimen) -> bool {
        if self.specimens.len() >= self.config.max_specimens {
            return false;
        }
        self.specimens.push(specimen);
        self.update_stats();
        true
    }

    /// Get specimen
    pub fn get_specimen(&self, id: &str) -> Option<&HerbariumSpecimen> {
        self.specimens.iter().find(|s| s.id == id)
    }

    /// Get specimen mut
    pub fn get_specimen_mut(&mut self, id: &str) -> Option<&mut HerbariumSpecimen> {
        self.specimens.iter_mut().find(|s| s.id == id)
    }

    /// Add taxonomist
    pub fn add_taxonomist(&mut self, taxonomist: HerbariumTaxonomist) {
        self.taxonomists.push(taxonomist);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.specimens, self.config.herbarium_type);
    }

    /// Get stats
    pub fn stats(&self) -> &HerbariumStats {
        &self.stats
    }

    /// Specimen count
    pub fn specimen_count(&self) -> usize {
        self.specimens.len()
    }
}

/// Herbarium registry
#[derive(Debug, Clone, Default)]
pub struct HerbariumRegistry {
    /// Herbariums by ID
    herbariums: HashMap<String, SettingsHerbarium>,
}

impl HerbariumRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register herbarium
    pub fn register(&mut self, id: impl Into<String>, herbarium: SettingsHerbarium) {
        self.herbariums.insert(id.into(), herbarium);
    }

    /// Unregister herbarium
    pub fn unregister(&mut self, id: &str) -> bool {
        self.herbariums.remove(id).is_some()
    }

    /// Get herbarium
    pub fn get(&self, id: &str) -> Option<&SettingsHerbarium> {
        self.herbariums.get(id)
    }

    /// Get herbarium mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsHerbarium> {
        self.herbariums.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.herbariums.len()
    }
}

/// Format herbarium registry
pub fn format_herbarium_registry(registry: &HerbariumRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Herbarium Registry:\n");
    output.push_str(&format!("  Herbariums: {}\n", registry.count()));
    output
}

/// Check if query is about herbarium
pub fn is_herbarium_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings herbarium") || lower.contains("herbarium settings") || lower.contains("plant herbarium")
}

/// Fun fact about herbarium
pub fn herbarium_fun_fact() -> &'static str {
    "Anna's settings herbarium preserves taxonomy boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_herbarium_type_display() {
        assert_eq!(format!("{}", HerbariumType::University), "university");
        assert_eq!(format!("{}", HerbariumType::Museum), "museum");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", HerbariumStatus::Active), "active");
        assert_eq!(format!("{}", HerbariumStatus::Archiving), "archiving");
    }

    #[test]
    fn test_config_new() {
        let c = HerbariumConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = HerbariumConfig::new("test")
            .herbarium_type(HerbariumType::National)
            .status(HerbariumStatus::Digitizing);
        assert_eq!(c.herbarium_type, HerbariumType::National);
        assert_eq!(c.status, HerbariumStatus::Digitizing);
    }

    #[test]
    fn test_specimen_new() {
        let s = HerbariumSpecimen::new("s1", "Title", "Content");
        assert_eq!(s.id, "s1");
    }

    #[test]
    fn test_specimen_builder() {
        let s = HerbariumSpecimen::new("s1", "Title", "Content")
            .cabinet(1);
        assert_eq!(s.cabinet, 1);
    }

    #[test]
    fn test_specimen_mounted() {
        let mut s = HerbariumSpecimen::new("s1", "Title", "Content");
        s.make_unmounted();
        assert!(!s.mounted);
        s.make_mounted();
        assert!(s.mounted);
    }

    #[test]
    fn test_taxonomist_new() {
        let t = HerbariumTaxonomist::new("key", "name", "s1");
        assert_eq!(t.specimen_id, "s1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = HerbariumStats::default();
        let specimen = HerbariumSpecimen::new("s1", "Title", "Content");
        s.update(&[specimen], HerbariumType::University);
        assert_eq!(s.total_specimens, 1);
        assert_eq!(s.mounted, 1);
    }

    #[test]
    fn test_herbarium_new() {
        let h = SettingsHerbarium::new(HerbariumConfig::default());
        assert_eq!(h.specimen_count(), 0);
    }

    #[test]
    fn test_herbarium_add_specimen() {
        let mut h = SettingsHerbarium::new(HerbariumConfig::default());
        h.add_specimen(HerbariumSpecimen::new("s1", "Title", "Content"));
        assert_eq!(h.specimen_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = HerbariumRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = HerbariumRegistry::new();
        r.register("h1", SettingsHerbarium::new(HerbariumConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_herbarium_query() {
        assert!(is_herbarium_query("settings herbarium"));
        assert!(!is_herbarium_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = herbarium_fun_fact();
        assert!(fact.contains("herbarium"));
    }
}
