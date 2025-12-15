// v0.0.771: Settings Conservatory (Phase 347)
// Glass conservatory for settings preservation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Conservatory type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ConservatoryType {
    /// Victorian conservatory
    #[default]
    Victorian,
    /// Modern conservatory
    Modern,
    /// Lean-to conservatory
    LeanTo,
    /// Edwardian conservatory
    Edwardian,
}

impl std::fmt::Display for ConservatoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Victorian => write!(f, "victorian"),
            Self::Modern => write!(f, "modern"),
            Self::LeanTo => write!(f, "lean-to"),
            Self::Edwardian => write!(f, "edwardian"),
        }
    }
}

/// Conservatory status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ConservatoryStatus {
    /// Open status
    #[default]
    Open,
    /// Closed status
    Closed,
    /// Ventilating status
    Ventilating,
    /// Renovation status
    Renovation,
}

impl std::fmt::Display for ConservatoryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::Closed => write!(f, "closed"),
            Self::Ventilating => write!(f, "ventilating"),
            Self::Renovation => write!(f, "renovation"),
        }
    }
}

/// Conservatory config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConservatoryConfig {
    /// Name
    pub name: String,
    /// Conservatory type
    pub conservatory_type: ConservatoryType,
    /// Status
    pub status: ConservatoryStatus,
    /// Max specimens
    pub max_specimens: usize,
}

impl ConservatoryConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            conservatory_type: ConservatoryType::Victorian,
            status: ConservatoryStatus::Open,
            max_specimens: 100,
        }
    }

    /// Set type
    pub fn conservatory_type(mut self, ct: ConservatoryType) -> Self {
        self.conservatory_type = ct;
        self
    }

    /// Set status
    pub fn status(mut self, s: ConservatoryStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max specimens
    pub fn max_specimens(mut self, max: usize) -> Self {
        self.max_specimens = max;
        self
    }
}

impl Default for ConservatoryConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Conservatory specimen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConservatorySpecimen {
    /// Specimen ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Section number
    pub section: u32,
    /// Preserved
    pub preserved: bool,
}

impl ConservatorySpecimen {
    /// Create new specimen
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            section: 0,
            preserved: true,
        }
    }

    /// Set section
    pub fn section(mut self, s: u32) -> Self {
        self.section = s;
        self
    }

    /// Make preserved
    pub fn make_preserved(&mut self) {
        self.preserved = true;
    }

    /// Make damaged
    pub fn make_damaged(&mut self) {
        self.preserved = false;
    }
}

/// Conservatory curator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConservatoryCurator {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Specimen ID
    pub specimen_id: String,
}

impl ConservatoryCurator {
    /// Create new curator
    pub fn new(key: impl Into<String>, name: impl Into<String>, specimen_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            specimen_id: specimen_id.into(),
        }
    }
}

/// Conservatory stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConservatoryStats {
    /// Total specimens
    pub total_specimens: usize,
    /// Preserved specimens
    pub preserved: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ConservatoryStats {
    /// Update from specimens
    pub fn update(&mut self, specimens: &[ConservatorySpecimen], conservatory_type: ConservatoryType) {
        self.total_specimens = specimens.len();
        self.preserved = specimens.iter().filter(|s| s.preserved).count();
        *self.by_type.entry(conservatory_type.to_string()).or_insert(0) += 1;
    }

    /// Preservation rate
    pub fn preservation_rate(&self) -> f64 {
        if self.total_specimens == 0 { 0.0 } else { self.preserved as f64 / self.total_specimens as f64 * 100.0 }
    }
}

/// Settings conservatory
#[derive(Debug, Clone, Default)]
pub struct SettingsConservatory {
    /// Config
    config: ConservatoryConfig,
    /// Specimens
    specimens: Vec<ConservatorySpecimen>,
    /// Curators
    curators: Vec<ConservatoryCurator>,
    /// Stats
    stats: ConservatoryStats,
}

impl SettingsConservatory {
    /// Create new conservatory system
    pub fn new(config: ConservatoryConfig) -> Self {
        Self {
            config,
            specimens: Vec::new(),
            curators: Vec::new(),
            stats: ConservatoryStats::default(),
        }
    }

    /// Add specimen
    pub fn add_specimen(&mut self, specimen: ConservatorySpecimen) -> bool {
        if self.specimens.len() >= self.config.max_specimens {
            return false;
        }
        self.specimens.push(specimen);
        self.update_stats();
        true
    }

    /// Get specimen
    pub fn get_specimen(&self, id: &str) -> Option<&ConservatorySpecimen> {
        self.specimens.iter().find(|s| s.id == id)
    }

    /// Get specimen mut
    pub fn get_specimen_mut(&mut self, id: &str) -> Option<&mut ConservatorySpecimen> {
        self.specimens.iter_mut().find(|s| s.id == id)
    }

    /// Add curator
    pub fn add_curator(&mut self, curator: ConservatoryCurator) {
        self.curators.push(curator);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.specimens, self.config.conservatory_type);
    }

    /// Get stats
    pub fn stats(&self) -> &ConservatoryStats {
        &self.stats
    }

    /// Specimen count
    pub fn specimen_count(&self) -> usize {
        self.specimens.len()
    }
}

/// Conservatory registry
#[derive(Debug, Clone, Default)]
pub struct ConservatoryRegistry {
    /// Conservatories by ID
    conservatories: HashMap<String, SettingsConservatory>,
}

impl ConservatoryRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register conservatory
    pub fn register(&mut self, id: impl Into<String>, conservatory: SettingsConservatory) {
        self.conservatories.insert(id.into(), conservatory);
    }

    /// Unregister conservatory
    pub fn unregister(&mut self, id: &str) -> bool {
        self.conservatories.remove(id).is_some()
    }

    /// Get conservatory
    pub fn get(&self, id: &str) -> Option<&SettingsConservatory> {
        self.conservatories.get(id)
    }

    /// Get conservatory mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsConservatory> {
        self.conservatories.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.conservatories.len()
    }
}

/// Format conservatory registry
pub fn format_conservatory_registry(registry: &ConservatoryRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Conservatory Registry:\n");
    output.push_str(&format!("  Conservatories: {}\n", registry.count()));
    output
}

/// Check if query is about conservatory
pub fn is_conservatory_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings conservatory") || lower.contains("conservatory settings") || lower.contains("glass conservatory")
}

/// Fun fact about conservatory
pub fn conservatory_fun_fact() -> &'static str {
    "Anna's settings conservatory preserves configuration boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conservatory_type_display() {
        assert_eq!(format!("{}", ConservatoryType::Victorian), "victorian");
        assert_eq!(format!("{}", ConservatoryType::Modern), "modern");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ConservatoryStatus::Open), "open");
        assert_eq!(format!("{}", ConservatoryStatus::Renovation), "renovation");
    }

    #[test]
    fn test_config_new() {
        let c = ConservatoryConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ConservatoryConfig::new("test")
            .conservatory_type(ConservatoryType::Edwardian)
            .status(ConservatoryStatus::Ventilating);
        assert_eq!(c.conservatory_type, ConservatoryType::Edwardian);
        assert_eq!(c.status, ConservatoryStatus::Ventilating);
    }

    #[test]
    fn test_specimen_new() {
        let s = ConservatorySpecimen::new("s1", "Title", "Content");
        assert_eq!(s.id, "s1");
    }

    #[test]
    fn test_specimen_builder() {
        let s = ConservatorySpecimen::new("s1", "Title", "Content")
            .section(1);
        assert_eq!(s.section, 1);
    }

    #[test]
    fn test_specimen_preserved() {
        let mut s = ConservatorySpecimen::new("s1", "Title", "Content");
        s.make_damaged();
        assert!(!s.preserved);
        s.make_preserved();
        assert!(s.preserved);
    }

    #[test]
    fn test_curator_new() {
        let c = ConservatoryCurator::new("key", "name", "s1");
        assert_eq!(c.specimen_id, "s1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = ConservatoryStats::default();
        let specimen = ConservatorySpecimen::new("s1", "Title", "Content");
        s.update(&[specimen], ConservatoryType::Victorian);
        assert_eq!(s.total_specimens, 1);
        assert_eq!(s.preserved, 1);
    }

    #[test]
    fn test_conservatory_new() {
        let c = SettingsConservatory::new(ConservatoryConfig::default());
        assert_eq!(c.specimen_count(), 0);
    }

    #[test]
    fn test_conservatory_add_specimen() {
        let mut c = SettingsConservatory::new(ConservatoryConfig::default());
        c.add_specimen(ConservatorySpecimen::new("s1", "Title", "Content"));
        assert_eq!(c.specimen_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = ConservatoryRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ConservatoryRegistry::new();
        r.register("c1", SettingsConservatory::new(ConservatoryConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_conservatory_query() {
        assert!(is_conservatory_query("settings conservatory"));
        assert!(!is_conservatory_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = conservatory_fun_fact();
        assert!(fact.contains("conservatory"));
    }
}
