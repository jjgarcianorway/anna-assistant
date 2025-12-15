// v0.0.769: Settings Nursery (Phase 345)
// Plant nursery for settings propagation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Nursery type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NurseryType {
    /// Retail nursery
    #[default]
    Retail,
    /// Wholesale nursery
    Wholesale,
    /// Specialty nursery
    Specialty,
    /// Research nursery
    Research,
}

impl std::fmt::Display for NurseryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Retail => write!(f, "retail"),
            Self::Wholesale => write!(f, "wholesale"),
            Self::Specialty => write!(f, "specialty"),
            Self::Research => write!(f, "research"),
        }
    }
}

/// Nursery status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NurseryStatus {
    /// Seeding status
    #[default]
    Seeding,
    /// Growing status
    Growing,
    /// Ready status
    Ready,
    /// Dormant status
    Dormant,
}

impl std::fmt::Display for NurseryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Seeding => write!(f, "seeding"),
            Self::Growing => write!(f, "growing"),
            Self::Ready => write!(f, "ready"),
            Self::Dormant => write!(f, "dormant"),
        }
    }
}

/// Nursery config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NurseryConfig {
    /// Name
    pub name: String,
    /// Nursery type
    pub nursery_type: NurseryType,
    /// Status
    pub status: NurseryStatus,
    /// Max seedlings
    pub max_seedlings: usize,
}

impl NurseryConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            nursery_type: NurseryType::Retail,
            status: NurseryStatus::Seeding,
            max_seedlings: 100,
        }
    }

    /// Set type
    pub fn nursery_type(mut self, nt: NurseryType) -> Self {
        self.nursery_type = nt;
        self
    }

    /// Set status
    pub fn status(mut self, s: NurseryStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max seedlings
    pub fn max_seedlings(mut self, max: usize) -> Self {
        self.max_seedlings = max;
        self
    }
}

impl Default for NurseryConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Nursery seedling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NurserySeedling {
    /// Seedling ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Tray number
    pub tray: u32,
    /// Viable
    pub viable: bool,
}

impl NurserySeedling {
    /// Create new seedling
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            tray: 0,
            viable: true,
        }
    }

    /// Set tray
    pub fn tray(mut self, t: u32) -> Self {
        self.tray = t;
        self
    }

    /// Make viable
    pub fn make_viable(&mut self) {
        self.viable = true;
    }

    /// Make unviable
    pub fn make_unviable(&mut self) {
        self.viable = false;
    }
}

/// Nursery propagator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NurseryPropagator {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Seedling ID
    pub seedling_id: String,
}

impl NurseryPropagator {
    /// Create new propagator
    pub fn new(key: impl Into<String>, name: impl Into<String>, seedling_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            seedling_id: seedling_id.into(),
        }
    }
}

/// Nursery stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NurseryStats {
    /// Total seedlings
    pub total_seedlings: usize,
    /// Viable seedlings
    pub viable: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl NurseryStats {
    /// Update from seedlings
    pub fn update(&mut self, seedlings: &[NurserySeedling], nursery_type: NurseryType) {
        self.total_seedlings = seedlings.len();
        self.viable = seedlings.iter().filter(|s| s.viable).count();
        *self.by_type.entry(nursery_type.to_string()).or_insert(0) += 1;
    }

    /// Viability rate
    pub fn viability_rate(&self) -> f64 {
        if self.total_seedlings == 0 { 0.0 } else { self.viable as f64 / self.total_seedlings as f64 * 100.0 }
    }
}

/// Settings nursery
#[derive(Debug, Clone, Default)]
pub struct SettingsNursery {
    /// Config
    config: NurseryConfig,
    /// Seedlings
    seedlings: Vec<NurserySeedling>,
    /// Propagators
    propagators: Vec<NurseryPropagator>,
    /// Stats
    stats: NurseryStats,
}

impl SettingsNursery {
    /// Create new nursery system
    pub fn new(config: NurseryConfig) -> Self {
        Self {
            config,
            seedlings: Vec::new(),
            propagators: Vec::new(),
            stats: NurseryStats::default(),
        }
    }

    /// Add seedling
    pub fn add_seedling(&mut self, seedling: NurserySeedling) -> bool {
        if self.seedlings.len() >= self.config.max_seedlings {
            return false;
        }
        self.seedlings.push(seedling);
        self.update_stats();
        true
    }

    /// Get seedling
    pub fn get_seedling(&self, id: &str) -> Option<&NurserySeedling> {
        self.seedlings.iter().find(|s| s.id == id)
    }

    /// Get seedling mut
    pub fn get_seedling_mut(&mut self, id: &str) -> Option<&mut NurserySeedling> {
        self.seedlings.iter_mut().find(|s| s.id == id)
    }

    /// Add propagator
    pub fn add_propagator(&mut self, propagator: NurseryPropagator) {
        self.propagators.push(propagator);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.seedlings, self.config.nursery_type);
    }

    /// Get stats
    pub fn stats(&self) -> &NurseryStats {
        &self.stats
    }

    /// Seedling count
    pub fn seedling_count(&self) -> usize {
        self.seedlings.len()
    }
}

/// Nursery registry
#[derive(Debug, Clone, Default)]
pub struct NurseryRegistry {
    /// Nurseries by ID
    nurseries: HashMap<String, SettingsNursery>,
}

impl NurseryRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register nursery
    pub fn register(&mut self, id: impl Into<String>, nursery: SettingsNursery) {
        self.nurseries.insert(id.into(), nursery);
    }

    /// Unregister nursery
    pub fn unregister(&mut self, id: &str) -> bool {
        self.nurseries.remove(id).is_some()
    }

    /// Get nursery
    pub fn get(&self, id: &str) -> Option<&SettingsNursery> {
        self.nurseries.get(id)
    }

    /// Get nursery mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsNursery> {
        self.nurseries.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.nurseries.len()
    }
}

/// Format nursery registry
pub fn format_nursery_registry(registry: &NurseryRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Nursery Registry:\n");
    output.push_str(&format!("  Nurseries: {}\n", registry.count()));
    output
}

/// Check if query is about nursery
pub fn is_nursery_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings nursery") || lower.contains("nursery settings") || lower.contains("plant nursery")
}

/// Fun fact about nursery
pub fn nursery_fun_fact() -> &'static str {
    "Anna's settings nursery propagates configuration boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nursery_type_display() {
        assert_eq!(format!("{}", NurseryType::Retail), "retail");
        assert_eq!(format!("{}", NurseryType::Wholesale), "wholesale");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", NurseryStatus::Seeding), "seeding");
        assert_eq!(format!("{}", NurseryStatus::Ready), "ready");
    }

    #[test]
    fn test_config_new() {
        let c = NurseryConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = NurseryConfig::new("test")
            .nursery_type(NurseryType::Specialty)
            .status(NurseryStatus::Growing);
        assert_eq!(c.nursery_type, NurseryType::Specialty);
        assert_eq!(c.status, NurseryStatus::Growing);
    }

    #[test]
    fn test_seedling_new() {
        let s = NurserySeedling::new("s1", "Title", "Content");
        assert_eq!(s.id, "s1");
    }

    #[test]
    fn test_seedling_builder() {
        let s = NurserySeedling::new("s1", "Title", "Content")
            .tray(1);
        assert_eq!(s.tray, 1);
    }

    #[test]
    fn test_seedling_viable() {
        let mut s = NurserySeedling::new("s1", "Title", "Content");
        s.make_unviable();
        assert!(!s.viable);
        s.make_viable();
        assert!(s.viable);
    }

    #[test]
    fn test_propagator_new() {
        let p = NurseryPropagator::new("key", "name", "s1");
        assert_eq!(p.seedling_id, "s1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = NurseryStats::default();
        let seedling = NurserySeedling::new("s1", "Title", "Content");
        s.update(&[seedling], NurseryType::Retail);
        assert_eq!(s.total_seedlings, 1);
        assert_eq!(s.viable, 1);
    }

    #[test]
    fn test_nursery_new() {
        let n = SettingsNursery::new(NurseryConfig::default());
        assert_eq!(n.seedling_count(), 0);
    }

    #[test]
    fn test_nursery_add_seedling() {
        let mut n = SettingsNursery::new(NurseryConfig::default());
        n.add_seedling(NurserySeedling::new("s1", "Title", "Content"));
        assert_eq!(n.seedling_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = NurseryRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = NurseryRegistry::new();
        r.register("n1", SettingsNursery::new(NurseryConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_nursery_query() {
        assert!(is_nursery_query("settings nursery"));
        assert!(!is_nursery_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = nursery_fun_fact();
        assert!(fact.contains("nursery"));
    }
}
