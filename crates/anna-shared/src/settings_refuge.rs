// v0.0.783: Settings Refuge (Phase 359)
// Wildlife refuge for settings shelter

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Refuge type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RefugeType {
    /// Wildlife refuge
    #[default]
    Wildlife,
    /// Bird refuge
    Bird,
    /// Fish refuge
    Fish,
    /// Mammal refuge
    Mammal,
}

impl std::fmt::Display for RefugeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wildlife => write!(f, "wildlife"),
            Self::Bird => write!(f, "bird"),
            Self::Fish => write!(f, "fish"),
            Self::Mammal => write!(f, "mammal"),
        }
    }
}

/// Refuge status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RefugeStatus {
    /// Active status
    #[default]
    Active,
    /// Sheltering status
    Sheltering,
    /// Protecting status
    Protecting,
    /// Recovering status
    Recovering,
}

impl std::fmt::Display for RefugeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Sheltering => write!(f, "sheltering"),
            Self::Protecting => write!(f, "protecting"),
            Self::Recovering => write!(f, "recovering"),
        }
    }
}

/// Refuge config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefugeConfig {
    /// Name
    pub name: String,
    /// Refuge type
    pub refuge_type: RefugeType,
    /// Status
    pub status: RefugeStatus,
    /// Max inhabitants
    pub max_inhabitants: usize,
}

impl RefugeConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            refuge_type: RefugeType::Wildlife,
            status: RefugeStatus::Active,
            max_inhabitants: 100,
        }
    }

    /// Set type
    pub fn refuge_type(mut self, rt: RefugeType) -> Self {
        self.refuge_type = rt;
        self
    }

    /// Set status
    pub fn status(mut self, s: RefugeStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max inhabitants
    pub fn max_inhabitants(mut self, max: usize) -> Self {
        self.max_inhabitants = max;
        self
    }
}

impl Default for RefugeConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Refuge inhabitant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefugeInhabitant {
    /// Inhabitant ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Shelter number
    pub shelter: u32,
    /// Safe
    pub safe: bool,
}

impl RefugeInhabitant {
    /// Create new inhabitant
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            shelter: 0,
            safe: true,
        }
    }

    /// Set shelter
    pub fn shelter(mut self, s: u32) -> Self {
        self.shelter = s;
        self
    }

    /// Make safe
    pub fn make_safe(&mut self) {
        self.safe = true;
    }

    /// Make vulnerable
    pub fn make_vulnerable(&mut self) {
        self.safe = false;
    }
}

/// Refuge warden
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefugeWarden {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Inhabitant ID
    pub inhabitant_id: String,
}

impl RefugeWarden {
    /// Create new warden
    pub fn new(key: impl Into<String>, name: impl Into<String>, inhabitant_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            inhabitant_id: inhabitant_id.into(),
        }
    }
}

/// Refuge stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RefugeStats {
    /// Total inhabitants
    pub total_inhabitants: usize,
    /// Safe inhabitants
    pub safe: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl RefugeStats {
    /// Update from inhabitants
    pub fn update(&mut self, inhabitants: &[RefugeInhabitant], refuge_type: RefugeType) {
        self.total_inhabitants = inhabitants.len();
        self.safe = inhabitants.iter().filter(|i| i.safe).count();
        *self.by_type.entry(refuge_type.to_string()).or_insert(0) += 1;
    }

    /// Safety rate
    pub fn safety_rate(&self) -> f64 {
        if self.total_inhabitants == 0 { 0.0 } else { self.safe as f64 / self.total_inhabitants as f64 * 100.0 }
    }
}

/// Settings refuge
#[derive(Debug, Clone, Default)]
pub struct SettingsRefuge {
    /// Config
    config: RefugeConfig,
    /// Inhabitants
    inhabitants: Vec<RefugeInhabitant>,
    /// Wardens
    wardens: Vec<RefugeWarden>,
    /// Stats
    stats: RefugeStats,
}

impl SettingsRefuge {
    /// Create new refuge system
    pub fn new(config: RefugeConfig) -> Self {
        Self {
            config,
            inhabitants: Vec::new(),
            wardens: Vec::new(),
            stats: RefugeStats::default(),
        }
    }

    /// Add inhabitant
    pub fn add_inhabitant(&mut self, inhabitant: RefugeInhabitant) -> bool {
        if self.inhabitants.len() >= self.config.max_inhabitants {
            return false;
        }
        self.inhabitants.push(inhabitant);
        self.update_stats();
        true
    }

    /// Get inhabitant
    pub fn get_inhabitant(&self, id: &str) -> Option<&RefugeInhabitant> {
        self.inhabitants.iter().find(|i| i.id == id)
    }

    /// Get inhabitant mut
    pub fn get_inhabitant_mut(&mut self, id: &str) -> Option<&mut RefugeInhabitant> {
        self.inhabitants.iter_mut().find(|i| i.id == id)
    }

    /// Add warden
    pub fn add_warden(&mut self, warden: RefugeWarden) {
        self.wardens.push(warden);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.inhabitants, self.config.refuge_type);
    }

    /// Get stats
    pub fn stats(&self) -> &RefugeStats {
        &self.stats
    }

    /// Inhabitant count
    pub fn inhabitant_count(&self) -> usize {
        self.inhabitants.len()
    }
}

/// Refuge registry
#[derive(Debug, Clone, Default)]
pub struct RefugeRegistry {
    /// Refuges by ID
    refuges: HashMap<String, SettingsRefuge>,
}

impl RefugeRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register refuge
    pub fn register(&mut self, id: impl Into<String>, refuge: SettingsRefuge) {
        self.refuges.insert(id.into(), refuge);
    }

    /// Unregister refuge
    pub fn unregister(&mut self, id: &str) -> bool {
        self.refuges.remove(id).is_some()
    }

    /// Get refuge
    pub fn get(&self, id: &str) -> Option<&SettingsRefuge> {
        self.refuges.get(id)
    }

    /// Get refuge mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsRefuge> {
        self.refuges.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.refuges.len()
    }
}

/// Format refuge registry
pub fn format_refuge_registry(registry: &RefugeRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Refuge Registry:\n");
    output.push_str(&format!("  Refuges: {}\n", registry.count()));
    output
}

/// Check if query is about refuge
pub fn is_refuge_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings refuge") || lower.contains("refuge settings") || lower.contains("wildlife refuge")
}

/// Fun fact about refuge
pub fn refuge_fun_fact() -> &'static str {
    "Anna's settings refuge provides shelter for configuration safety!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refuge_type_display() {
        assert_eq!(format!("{}", RefugeType::Wildlife), "wildlife");
        assert_eq!(format!("{}", RefugeType::Bird), "bird");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", RefugeStatus::Active), "active");
        assert_eq!(format!("{}", RefugeStatus::Recovering), "recovering");
    }

    #[test]
    fn test_config_new() {
        let c = RefugeConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = RefugeConfig::new("test")
            .refuge_type(RefugeType::Bird)
            .status(RefugeStatus::Sheltering);
        assert_eq!(c.refuge_type, RefugeType::Bird);
        assert_eq!(c.status, RefugeStatus::Sheltering);
    }

    #[test]
    fn test_inhabitant_new() {
        let i = RefugeInhabitant::new("i1", "Title", "Content");
        assert_eq!(i.id, "i1");
    }

    #[test]
    fn test_inhabitant_builder() {
        let i = RefugeInhabitant::new("i1", "Title", "Content")
            .shelter(1);
        assert_eq!(i.shelter, 1);
    }

    #[test]
    fn test_inhabitant_safety() {
        let mut i = RefugeInhabitant::new("i1", "Title", "Content");
        i.make_vulnerable();
        assert!(!i.safe);
        i.make_safe();
        assert!(i.safe);
    }

    #[test]
    fn test_warden_new() {
        let w = RefugeWarden::new("key", "name", "i1");
        assert_eq!(w.inhabitant_id, "i1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = RefugeStats::default();
        let inhabitant = RefugeInhabitant::new("i1", "Title", "Content");
        s.update(&[inhabitant], RefugeType::Wildlife);
        assert_eq!(s.total_inhabitants, 1);
        assert_eq!(s.safe, 1);
    }

    #[test]
    fn test_refuge_new() {
        let r = SettingsRefuge::new(RefugeConfig::default());
        assert_eq!(r.inhabitant_count(), 0);
    }

    #[test]
    fn test_refuge_add_inhabitant() {
        let mut r = SettingsRefuge::new(RefugeConfig::default());
        r.add_inhabitant(RefugeInhabitant::new("i1", "Title", "Content"));
        assert_eq!(r.inhabitant_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = RefugeRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = RefugeRegistry::new();
        r.register("r1", SettingsRefuge::new(RefugeConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_refuge_query() {
        assert!(is_refuge_query("settings refuge"));
        assert!(!is_refuge_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = refuge_fun_fact();
        assert!(fact.contains("refuge"));
    }
}
