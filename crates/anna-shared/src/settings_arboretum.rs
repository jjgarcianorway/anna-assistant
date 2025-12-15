// v0.0.772: Settings Arboretum (Phase 348)
// Tree arboretum for settings dendrology

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Arboretum type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ArboretumType {
    /// Public arboretum
    #[default]
    Public,
    /// University arboretum
    University,
    /// Memorial arboretum
    Memorial,
    /// Research arboretum
    Research,
}

impl std::fmt::Display for ArboretumType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public => write!(f, "public"),
            Self::University => write!(f, "university"),
            Self::Memorial => write!(f, "memorial"),
            Self::Research => write!(f, "research"),
        }
    }
}

/// Arboretum status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ArboretumStatus {
    /// Open status
    #[default]
    Open,
    /// Planting status
    Planting,
    /// Surveying status
    Surveying,
    /// Closed status
    Closed,
}

impl std::fmt::Display for ArboretumStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::Planting => write!(f, "planting"),
            Self::Surveying => write!(f, "surveying"),
            Self::Closed => write!(f, "closed"),
        }
    }
}

/// Arboretum config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArboretumConfig {
    /// Name
    pub name: String,
    /// Arboretum type
    pub arboretum_type: ArboretumType,
    /// Status
    pub status: ArboretumStatus,
    /// Max specimens
    pub max_specimens: usize,
}

impl ArboretumConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            arboretum_type: ArboretumType::Public,
            status: ArboretumStatus::Open,
            max_specimens: 100,
        }
    }

    /// Set type
    pub fn arboretum_type(mut self, at: ArboretumType) -> Self {
        self.arboretum_type = at;
        self
    }

    /// Set status
    pub fn status(mut self, s: ArboretumStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max specimens
    pub fn max_specimens(mut self, max: usize) -> Self {
        self.max_specimens = max;
        self
    }
}

impl Default for ArboretumConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Arboretum specimen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArboretumSpecimen {
    /// Specimen ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Plot number
    pub plot: u32,
    /// Cataloged
    pub cataloged: bool,
}

impl ArboretumSpecimen {
    /// Create new specimen
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            plot: 0,
            cataloged: true,
        }
    }

    /// Set plot
    pub fn plot(mut self, p: u32) -> Self {
        self.plot = p;
        self
    }

    /// Make cataloged
    pub fn make_cataloged(&mut self) {
        self.cataloged = true;
    }

    /// Make uncataloged
    pub fn make_uncataloged(&mut self) {
        self.cataloged = false;
    }
}

/// Arboretum dendrologist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArboretumDendrologist {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Specimen ID
    pub specimen_id: String,
}

impl ArboretumDendrologist {
    /// Create new dendrologist
    pub fn new(key: impl Into<String>, name: impl Into<String>, specimen_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            specimen_id: specimen_id.into(),
        }
    }
}

/// Arboretum stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArboretumStats {
    /// Total specimens
    pub total_specimens: usize,
    /// Cataloged specimens
    pub cataloged: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ArboretumStats {
    /// Update from specimens
    pub fn update(&mut self, specimens: &[ArboretumSpecimen], arboretum_type: ArboretumType) {
        self.total_specimens = specimens.len();
        self.cataloged = specimens.iter().filter(|s| s.cataloged).count();
        *self.by_type.entry(arboretum_type.to_string()).or_insert(0) += 1;
    }

    /// Catalog rate
    pub fn catalog_rate(&self) -> f64 {
        if self.total_specimens == 0 { 0.0 } else { self.cataloged as f64 / self.total_specimens as f64 * 100.0 }
    }
}

/// Settings arboretum
#[derive(Debug, Clone, Default)]
pub struct SettingsArboretum {
    /// Config
    config: ArboretumConfig,
    /// Specimens
    specimens: Vec<ArboretumSpecimen>,
    /// Dendrologists
    dendrologists: Vec<ArboretumDendrologist>,
    /// Stats
    stats: ArboretumStats,
}

impl SettingsArboretum {
    /// Create new arboretum system
    pub fn new(config: ArboretumConfig) -> Self {
        Self {
            config,
            specimens: Vec::new(),
            dendrologists: Vec::new(),
            stats: ArboretumStats::default(),
        }
    }

    /// Add specimen
    pub fn add_specimen(&mut self, specimen: ArboretumSpecimen) -> bool {
        if self.specimens.len() >= self.config.max_specimens {
            return false;
        }
        self.specimens.push(specimen);
        self.update_stats();
        true
    }

    /// Get specimen
    pub fn get_specimen(&self, id: &str) -> Option<&ArboretumSpecimen> {
        self.specimens.iter().find(|s| s.id == id)
    }

    /// Get specimen mut
    pub fn get_specimen_mut(&mut self, id: &str) -> Option<&mut ArboretumSpecimen> {
        self.specimens.iter_mut().find(|s| s.id == id)
    }

    /// Add dendrologist
    pub fn add_dendrologist(&mut self, dendrologist: ArboretumDendrologist) {
        self.dendrologists.push(dendrologist);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.specimens, self.config.arboretum_type);
    }

    /// Get stats
    pub fn stats(&self) -> &ArboretumStats {
        &self.stats
    }

    /// Specimen count
    pub fn specimen_count(&self) -> usize {
        self.specimens.len()
    }
}

/// Arboretum registry
#[derive(Debug, Clone, Default)]
pub struct ArboretumRegistry {
    /// Arboretums by ID
    arboretums: HashMap<String, SettingsArboretum>,
}

impl ArboretumRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register arboretum
    pub fn register(&mut self, id: impl Into<String>, arboretum: SettingsArboretum) {
        self.arboretums.insert(id.into(), arboretum);
    }

    /// Unregister arboretum
    pub fn unregister(&mut self, id: &str) -> bool {
        self.arboretums.remove(id).is_some()
    }

    /// Get arboretum
    pub fn get(&self, id: &str) -> Option<&SettingsArboretum> {
        self.arboretums.get(id)
    }

    /// Get arboretum mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsArboretum> {
        self.arboretums.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.arboretums.len()
    }
}

/// Format arboretum registry
pub fn format_arboretum_registry(registry: &ArboretumRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Arboretum Registry:\n");
    output.push_str(&format!("  Arboretums: {}\n", registry.count()));
    output
}

/// Check if query is about arboretum
pub fn is_arboretum_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings arboretum") || lower.contains("arboretum settings") || lower.contains("tree arboretum")
}

/// Fun fact about arboretum
pub fn arboretum_fun_fact() -> &'static str {
    "Anna's settings arboretum catalogs dendrology boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arboretum_type_display() {
        assert_eq!(format!("{}", ArboretumType::Public), "public");
        assert_eq!(format!("{}", ArboretumType::University), "university");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ArboretumStatus::Open), "open");
        assert_eq!(format!("{}", ArboretumStatus::Closed), "closed");
    }

    #[test]
    fn test_config_new() {
        let c = ArboretumConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ArboretumConfig::new("test")
            .arboretum_type(ArboretumType::Memorial)
            .status(ArboretumStatus::Planting);
        assert_eq!(c.arboretum_type, ArboretumType::Memorial);
        assert_eq!(c.status, ArboretumStatus::Planting);
    }

    #[test]
    fn test_specimen_new() {
        let s = ArboretumSpecimen::new("s1", "Title", "Content");
        assert_eq!(s.id, "s1");
    }

    #[test]
    fn test_specimen_builder() {
        let s = ArboretumSpecimen::new("s1", "Title", "Content")
            .plot(1);
        assert_eq!(s.plot, 1);
    }

    #[test]
    fn test_specimen_cataloged() {
        let mut s = ArboretumSpecimen::new("s1", "Title", "Content");
        s.make_uncataloged();
        assert!(!s.cataloged);
        s.make_cataloged();
        assert!(s.cataloged);
    }

    #[test]
    fn test_dendrologist_new() {
        let d = ArboretumDendrologist::new("key", "name", "s1");
        assert_eq!(d.specimen_id, "s1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = ArboretumStats::default();
        let specimen = ArboretumSpecimen::new("s1", "Title", "Content");
        s.update(&[specimen], ArboretumType::Public);
        assert_eq!(s.total_specimens, 1);
        assert_eq!(s.cataloged, 1);
    }

    #[test]
    fn test_arboretum_new() {
        let a = SettingsArboretum::new(ArboretumConfig::default());
        assert_eq!(a.specimen_count(), 0);
    }

    #[test]
    fn test_arboretum_add_specimen() {
        let mut a = SettingsArboretum::new(ArboretumConfig::default());
        a.add_specimen(ArboretumSpecimen::new("s1", "Title", "Content"));
        assert_eq!(a.specimen_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = ArboretumRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ArboretumRegistry::new();
        r.register("a1", SettingsArboretum::new(ArboretumConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_arboretum_query() {
        assert!(is_arboretum_query("settings arboretum"));
        assert!(!is_arboretum_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = arboretum_fun_fact();
        assert!(fact.contains("arboretum"));
    }
}
