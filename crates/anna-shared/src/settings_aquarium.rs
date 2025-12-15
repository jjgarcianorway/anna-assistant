// v0.0.775: Settings Aquarium (Phase 351)
// Aquatic aquarium for settings marine life

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Aquarium type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AquariumType {
    /// Freshwater aquarium
    #[default]
    Freshwater,
    /// Saltwater aquarium
    Saltwater,
    /// Reef aquarium
    Reef,
    /// Brackish aquarium
    Brackish,
}

impl std::fmt::Display for AquariumType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Freshwater => write!(f, "freshwater"),
            Self::Saltwater => write!(f, "saltwater"),
            Self::Reef => write!(f, "reef"),
            Self::Brackish => write!(f, "brackish"),
        }
    }
}

/// Aquarium status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AquariumStatus {
    /// Cycling status
    #[default]
    Cycling,
    /// Stable status
    Stable,
    /// Stocking status
    Stocking,
    /// Maintenance status
    Maintenance,
}

impl std::fmt::Display for AquariumStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cycling => write!(f, "cycling"),
            Self::Stable => write!(f, "stable"),
            Self::Stocking => write!(f, "stocking"),
            Self::Maintenance => write!(f, "maintenance"),
        }
    }
}

/// Aquarium config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AquariumConfig {
    /// Name
    pub name: String,
    /// Aquarium type
    pub aquarium_type: AquariumType,
    /// Status
    pub status: AquariumStatus,
    /// Max inhabitants
    pub max_inhabitants: usize,
}

impl AquariumConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            aquarium_type: AquariumType::Freshwater,
            status: AquariumStatus::Cycling,
            max_inhabitants: 100,
        }
    }

    /// Set type
    pub fn aquarium_type(mut self, at: AquariumType) -> Self {
        self.aquarium_type = at;
        self
    }

    /// Set status
    pub fn status(mut self, s: AquariumStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max inhabitants
    pub fn max_inhabitants(mut self, max: usize) -> Self {
        self.max_inhabitants = max;
        self
    }
}

impl Default for AquariumConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Aquarium inhabitant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AquariumInhabitant {
    /// Inhabitant ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Tank number
    pub tank: u32,
    /// Healthy
    pub healthy: bool,
}

impl AquariumInhabitant {
    /// Create new inhabitant
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            tank: 0,
            healthy: true,
        }
    }

    /// Set tank
    pub fn tank(mut self, t: u32) -> Self {
        self.tank = t;
        self
    }

    /// Make healthy
    pub fn make_healthy(&mut self) {
        self.healthy = true;
    }

    /// Make sick
    pub fn make_sick(&mut self) {
        self.healthy = false;
    }
}

/// Aquarium aquarist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AquariumAquarist {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Inhabitant ID
    pub inhabitant_id: String,
}

impl AquariumAquarist {
    /// Create new aquarist
    pub fn new(key: impl Into<String>, name: impl Into<String>, inhabitant_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            inhabitant_id: inhabitant_id.into(),
        }
    }
}

/// Aquarium stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AquariumStats {
    /// Total inhabitants
    pub total_inhabitants: usize,
    /// Healthy inhabitants
    pub healthy: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl AquariumStats {
    /// Update from inhabitants
    pub fn update(&mut self, inhabitants: &[AquariumInhabitant], aquarium_type: AquariumType) {
        self.total_inhabitants = inhabitants.len();
        self.healthy = inhabitants.iter().filter(|i| i.healthy).count();
        *self.by_type.entry(aquarium_type.to_string()).or_insert(0) += 1;
    }

    /// Health rate
    pub fn health_rate(&self) -> f64 {
        if self.total_inhabitants == 0 { 0.0 } else { self.healthy as f64 / self.total_inhabitants as f64 * 100.0 }
    }
}

/// Settings aquarium
#[derive(Debug, Clone, Default)]
pub struct SettingsAquarium {
    /// Config
    config: AquariumConfig,
    /// Inhabitants
    inhabitants: Vec<AquariumInhabitant>,
    /// Aquarists
    aquarists: Vec<AquariumAquarist>,
    /// Stats
    stats: AquariumStats,
}

impl SettingsAquarium {
    /// Create new aquarium system
    pub fn new(config: AquariumConfig) -> Self {
        Self {
            config,
            inhabitants: Vec::new(),
            aquarists: Vec::new(),
            stats: AquariumStats::default(),
        }
    }

    /// Add inhabitant
    pub fn add_inhabitant(&mut self, inhabitant: AquariumInhabitant) -> bool {
        if self.inhabitants.len() >= self.config.max_inhabitants {
            return false;
        }
        self.inhabitants.push(inhabitant);
        self.update_stats();
        true
    }

    /// Get inhabitant
    pub fn get_inhabitant(&self, id: &str) -> Option<&AquariumInhabitant> {
        self.inhabitants.iter().find(|i| i.id == id)
    }

    /// Get inhabitant mut
    pub fn get_inhabitant_mut(&mut self, id: &str) -> Option<&mut AquariumInhabitant> {
        self.inhabitants.iter_mut().find(|i| i.id == id)
    }

    /// Add aquarist
    pub fn add_aquarist(&mut self, aquarist: AquariumAquarist) {
        self.aquarists.push(aquarist);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.inhabitants, self.config.aquarium_type);
    }

    /// Get stats
    pub fn stats(&self) -> &AquariumStats {
        &self.stats
    }

    /// Inhabitant count
    pub fn inhabitant_count(&self) -> usize {
        self.inhabitants.len()
    }
}

/// Aquarium registry
#[derive(Debug, Clone, Default)]
pub struct AquariumRegistry {
    /// Aquariums by ID
    aquariums: HashMap<String, SettingsAquarium>,
}

impl AquariumRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register aquarium
    pub fn register(&mut self, id: impl Into<String>, aquarium: SettingsAquarium) {
        self.aquariums.insert(id.into(), aquarium);
    }

    /// Unregister aquarium
    pub fn unregister(&mut self, id: &str) -> bool {
        self.aquariums.remove(id).is_some()
    }

    /// Get aquarium
    pub fn get(&self, id: &str) -> Option<&SettingsAquarium> {
        self.aquariums.get(id)
    }

    /// Get aquarium mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsAquarium> {
        self.aquariums.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.aquariums.len()
    }
}

/// Format aquarium registry
pub fn format_aquarium_registry(registry: &AquariumRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Aquarium Registry:\n");
    output.push_str(&format!("  Aquariums: {}\n", registry.count()));
    output
}

/// Check if query is about aquarium
pub fn is_aquarium_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings aquarium") || lower.contains("aquarium settings") || lower.contains("aquatic aquarium")
}

/// Fun fact about aquarium
pub fn aquarium_fun_fact() -> &'static str {
    "Anna's settings aquarium maintains marine life boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aquarium_type_display() {
        assert_eq!(format!("{}", AquariumType::Freshwater), "freshwater");
        assert_eq!(format!("{}", AquariumType::Saltwater), "saltwater");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", AquariumStatus::Cycling), "cycling");
        assert_eq!(format!("{}", AquariumStatus::Stable), "stable");
    }

    #[test]
    fn test_config_new() {
        let c = AquariumConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = AquariumConfig::new("test")
            .aquarium_type(AquariumType::Reef)
            .status(AquariumStatus::Stocking);
        assert_eq!(c.aquarium_type, AquariumType::Reef);
        assert_eq!(c.status, AquariumStatus::Stocking);
    }

    #[test]
    fn test_inhabitant_new() {
        let i = AquariumInhabitant::new("i1", "Title", "Content");
        assert_eq!(i.id, "i1");
    }

    #[test]
    fn test_inhabitant_builder() {
        let i = AquariumInhabitant::new("i1", "Title", "Content")
            .tank(1);
        assert_eq!(i.tank, 1);
    }

    #[test]
    fn test_inhabitant_healthy() {
        let mut i = AquariumInhabitant::new("i1", "Title", "Content");
        i.make_sick();
        assert!(!i.healthy);
        i.make_healthy();
        assert!(i.healthy);
    }

    #[test]
    fn test_aquarist_new() {
        let a = AquariumAquarist::new("key", "name", "i1");
        assert_eq!(a.inhabitant_id, "i1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = AquariumStats::default();
        let inhabitant = AquariumInhabitant::new("i1", "Title", "Content");
        s.update(&[inhabitant], AquariumType::Freshwater);
        assert_eq!(s.total_inhabitants, 1);
        assert_eq!(s.healthy, 1);
    }

    #[test]
    fn test_aquarium_new() {
        let a = SettingsAquarium::new(AquariumConfig::default());
        assert_eq!(a.inhabitant_count(), 0);
    }

    #[test]
    fn test_aquarium_add_inhabitant() {
        let mut a = SettingsAquarium::new(AquariumConfig::default());
        a.add_inhabitant(AquariumInhabitant::new("i1", "Title", "Content"));
        assert_eq!(a.inhabitant_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = AquariumRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = AquariumRegistry::new();
        r.register("a1", SettingsAquarium::new(AquariumConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_aquarium_query() {
        assert!(is_aquarium_query("settings aquarium"));
        assert!(!is_aquarium_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = aquarium_fun_fact();
        assert!(fact.contains("aquarium"));
    }
}
