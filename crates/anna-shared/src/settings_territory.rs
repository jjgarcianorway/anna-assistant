// v0.0.745: Settings Territory (Phase 321)
// Controlled territory for settings administration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Territory type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TerritoryType {
    /// Sovereign territory
    #[default]
    Sovereign,
    /// Occupied territory
    Occupied,
    /// Trust territory
    Trust,
    /// Dependent territory
    Dependent,
}

impl std::fmt::Display for TerritoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sovereign => write!(f, "sovereign"),
            Self::Occupied => write!(f, "occupied"),
            Self::Trust => write!(f, "trust"),
            Self::Dependent => write!(f, "dependent"),
        }
    }
}

/// Territory status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TerritoryStatus {
    /// Administered status
    #[default]
    Administered,
    /// Autonomous status
    Autonomous,
    /// Contested status
    Contested,
    /// Ceded status
    Ceded,
}

impl std::fmt::Display for TerritoryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Administered => write!(f, "administered"),
            Self::Autonomous => write!(f, "autonomous"),
            Self::Contested => write!(f, "contested"),
            Self::Ceded => write!(f, "ceded"),
        }
    }
}

/// Territory config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryConfig {
    /// Name
    pub name: String,
    /// Territory type
    pub territory_type: TerritoryType,
    /// Status
    pub status: TerritoryStatus,
    /// Max ordinances
    pub max_ordinances: usize,
}

impl TerritoryConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            territory_type: TerritoryType::Sovereign,
            status: TerritoryStatus::Administered,
            max_ordinances: 100,
        }
    }

    /// Set type
    pub fn territory_type(mut self, tt: TerritoryType) -> Self {
        self.territory_type = tt;
        self
    }

    /// Set status
    pub fn status(mut self, s: TerritoryStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max ordinances
    pub fn max_ordinances(mut self, max: usize) -> Self {
        self.max_ordinances = max;
        self
    }
}

impl Default for TerritoryConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Territory ordinance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryOrdinance {
    /// Ordinance ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// District number
    pub district: u32,
    /// Enforced
    pub enforced: bool,
}

impl TerritoryOrdinance {
    /// Create new ordinance
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            district: 0,
            enforced: true,
        }
    }

    /// Set district
    pub fn district(mut self, d: u32) -> Self {
        self.district = d;
        self
    }

    /// Make enforced
    pub fn make_enforced(&mut self) {
        self.enforced = true;
    }

    /// Make suspended
    pub fn make_suspended(&mut self) {
        self.enforced = false;
    }
}

/// Territory administrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryAdministrator {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Ordinance ID
    pub ordinance_id: String,
}

impl TerritoryAdministrator {
    /// Create new administrator
    pub fn new(key: impl Into<String>, name: impl Into<String>, ordinance_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            ordinance_id: ordinance_id.into(),
        }
    }
}

/// Territory stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TerritoryStats {
    /// Total ordinances
    pub total_ordinances: usize,
    /// Enforced ordinances
    pub enforced: usize,
    /// Autonomous count
    pub autonomous_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl TerritoryStats {
    /// Update from ordinances
    pub fn update(&mut self, ordinances: &[TerritoryOrdinance], territory_type: TerritoryType) {
        self.total_ordinances = ordinances.len();
        self.enforced = ordinances.iter().filter(|o| o.enforced).count();
        *self.by_type.entry(territory_type.to_string()).or_insert(0) += 1;
    }

    /// Enforcement rate
    pub fn enforcement_rate(&self) -> f64 {
        if self.total_ordinances == 0 { 0.0 } else { self.enforced as f64 / self.total_ordinances as f64 * 100.0 }
    }
}

/// Settings territory
#[derive(Debug, Clone, Default)]
pub struct SettingsTerritory {
    /// Config
    config: TerritoryConfig,
    /// Ordinances
    ordinances: Vec<TerritoryOrdinance>,
    /// Administrators
    administrators: Vec<TerritoryAdministrator>,
    /// Stats
    stats: TerritoryStats,
}

impl SettingsTerritory {
    /// Create new territory system
    pub fn new(config: TerritoryConfig) -> Self {
        Self {
            config,
            ordinances: Vec::new(),
            administrators: Vec::new(),
            stats: TerritoryStats::default(),
        }
    }

    /// Add ordinance
    pub fn add_ordinance(&mut self, ordinance: TerritoryOrdinance) -> bool {
        if self.ordinances.len() >= self.config.max_ordinances {
            return false;
        }
        self.ordinances.push(ordinance);
        self.update_stats();
        true
    }

    /// Get ordinance
    pub fn get_ordinance(&self, id: &str) -> Option<&TerritoryOrdinance> {
        self.ordinances.iter().find(|o| o.id == id)
    }

    /// Get ordinance mut
    pub fn get_ordinance_mut(&mut self, id: &str) -> Option<&mut TerritoryOrdinance> {
        self.ordinances.iter_mut().find(|o| o.id == id)
    }

    /// Add administrator
    pub fn add_administrator(&mut self, administrator: TerritoryAdministrator) {
        self.administrators.push(administrator);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.ordinances, self.config.territory_type);
    }

    /// Get stats
    pub fn stats(&self) -> &TerritoryStats {
        &self.stats
    }

    /// Ordinance count
    pub fn ordinance_count(&self) -> usize {
        self.ordinances.len()
    }
}

/// Territory registry
#[derive(Debug, Clone, Default)]
pub struct TerritoryRegistry {
    /// Territories by ID
    territories: HashMap<String, SettingsTerritory>,
}

impl TerritoryRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register territory
    pub fn register(&mut self, id: impl Into<String>, territory: SettingsTerritory) {
        self.territories.insert(id.into(), territory);
    }

    /// Unregister territory
    pub fn unregister(&mut self, id: &str) -> bool {
        self.territories.remove(id).is_some()
    }

    /// Get territory
    pub fn get(&self, id: &str) -> Option<&SettingsTerritory> {
        self.territories.get(id)
    }

    /// Get territory mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsTerritory> {
        self.territories.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.territories.len()
    }
}

/// Format territory registry
pub fn format_territory_registry(registry: &TerritoryRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Territory Registry:\n");
    output.push_str(&format!("  Territories: {}\n", registry.count()));
    output
}

/// Check if query is about territory
pub fn is_territory_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings territory") || lower.contains("territory settings") || lower.contains("controlled territory")
}

/// Fun fact about territory
pub fn territory_fun_fact() -> &'static str {
    "Anna's settings territory establishes controlled administration!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_territory_type_display() {
        assert_eq!(format!("{}", TerritoryType::Sovereign), "sovereign");
        assert_eq!(format!("{}", TerritoryType::Occupied), "occupied");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", TerritoryStatus::Administered), "administered");
        assert_eq!(format!("{}", TerritoryStatus::Autonomous), "autonomous");
    }

    #[test]
    fn test_config_new() {
        let c = TerritoryConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = TerritoryConfig::new("test")
            .territory_type(TerritoryType::Trust)
            .status(TerritoryStatus::Autonomous);
        assert_eq!(c.territory_type, TerritoryType::Trust);
        assert_eq!(c.status, TerritoryStatus::Autonomous);
    }

    #[test]
    fn test_ordinance_new() {
        let o = TerritoryOrdinance::new("o1", "Title", "Content");
        assert_eq!(o.id, "o1");
    }

    #[test]
    fn test_ordinance_builder() {
        let o = TerritoryOrdinance::new("o1", "Title", "Content")
            .district(1);
        assert_eq!(o.district, 1);
    }

    #[test]
    fn test_ordinance_enforced() {
        let mut o = TerritoryOrdinance::new("o1", "Title", "Content");
        o.make_suspended();
        assert!(!o.enforced);
        o.make_enforced();
        assert!(o.enforced);
    }

    #[test]
    fn test_administrator_new() {
        let a = TerritoryAdministrator::new("key", "name", "o1");
        assert_eq!(a.ordinance_id, "o1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = TerritoryStats::default();
        let ordinance = TerritoryOrdinance::new("o1", "Title", "Content");
        s.update(&[ordinance], TerritoryType::Sovereign);
        assert_eq!(s.total_ordinances, 1);
        assert_eq!(s.enforced, 1);
    }

    #[test]
    fn test_territory_new() {
        let t = SettingsTerritory::new(TerritoryConfig::default());
        assert_eq!(t.ordinance_count(), 0);
    }

    #[test]
    fn test_territory_add_ordinance() {
        let mut t = SettingsTerritory::new(TerritoryConfig::default());
        t.add_ordinance(TerritoryOrdinance::new("o1", "Title", "Content"));
        assert_eq!(t.ordinance_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = TerritoryRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = TerritoryRegistry::new();
        r.register("t1", SettingsTerritory::new(TerritoryConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_territory_query() {
        assert!(is_territory_query("settings territory"));
        assert!(!is_territory_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = territory_fun_fact();
        assert!(fact.contains("territory"));
    }
}
