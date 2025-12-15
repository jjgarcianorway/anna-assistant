// v0.0.744: Settings Realm (Phase 320)
// Royal realm for settings sovereignty

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Realm type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RealmType {
    /// Kingdom realm
    #[default]
    Kingdom,
    /// Empire realm
    Empire,
    /// Principality realm
    Principality,
    /// Duchy realm
    Duchy,
}

impl std::fmt::Display for RealmType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Kingdom => write!(f, "kingdom"),
            Self::Empire => write!(f, "empire"),
            Self::Principality => write!(f, "principality"),
            Self::Duchy => write!(f, "duchy"),
        }
    }
}

/// Realm status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RealmStatus {
    /// Rising status
    #[default]
    Rising,
    /// Prosperous status
    Prosperous,
    /// Stagnant status
    Stagnant,
    /// Declining status
    Declining,
}

impl std::fmt::Display for RealmStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rising => write!(f, "rising"),
            Self::Prosperous => write!(f, "prosperous"),
            Self::Stagnant => write!(f, "stagnant"),
            Self::Declining => write!(f, "declining"),
        }
    }
}

/// Realm config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmConfig {
    /// Name
    pub name: String,
    /// Realm type
    pub realm_type: RealmType,
    /// Status
    pub status: RealmStatus,
    /// Max decrees
    pub max_decrees: usize,
}

impl RealmConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            realm_type: RealmType::Kingdom,
            status: RealmStatus::Rising,
            max_decrees: 100,
        }
    }

    /// Set type
    pub fn realm_type(mut self, rt: RealmType) -> Self {
        self.realm_type = rt;
        self
    }

    /// Set status
    pub fn status(mut self, s: RealmStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max decrees
    pub fn max_decrees(mut self, max: usize) -> Self {
        self.max_decrees = max;
        self
    }
}

impl Default for RealmConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Realm decree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmDecree {
    /// Decree ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Order number
    pub order: u32,
    /// Royal
    pub royal: bool,
}

impl RealmDecree {
    /// Create new decree
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            order: 0,
            royal: true,
        }
    }

    /// Set order
    pub fn order(mut self, o: u32) -> Self {
        self.order = o;
        self
    }

    /// Make royal
    pub fn make_royal(&mut self) {
        self.royal = true;
    }

    /// Make common
    pub fn make_common(&mut self) {
        self.royal = false;
    }
}

/// Realm vassal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmVassal {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Decree ID
    pub decree_id: String,
}

impl RealmVassal {
    /// Create new vassal
    pub fn new(key: impl Into<String>, name: impl Into<String>, decree_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            decree_id: decree_id.into(),
        }
    }
}

/// Realm stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RealmStats {
    /// Total decrees
    pub total_decrees: usize,
    /// Royal decrees
    pub royal: usize,
    /// Prosperous count
    pub prosperous_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl RealmStats {
    /// Update from decrees
    pub fn update(&mut self, decrees: &[RealmDecree], realm_type: RealmType) {
        self.total_decrees = decrees.len();
        self.royal = decrees.iter().filter(|d| d.royal).count();
        *self.by_type.entry(realm_type.to_string()).or_insert(0) += 1;
    }

    /// Royal rate
    pub fn royal_rate(&self) -> f64 {
        if self.total_decrees == 0 { 0.0 } else { self.royal as f64 / self.total_decrees as f64 * 100.0 }
    }
}

/// Settings realm
#[derive(Debug, Clone, Default)]
pub struct SettingsRealm {
    /// Config
    config: RealmConfig,
    /// Decrees
    decrees: Vec<RealmDecree>,
    /// Vassals
    vassals: Vec<RealmVassal>,
    /// Stats
    stats: RealmStats,
}

impl SettingsRealm {
    /// Create new realm system
    pub fn new(config: RealmConfig) -> Self {
        Self {
            config,
            decrees: Vec::new(),
            vassals: Vec::new(),
            stats: RealmStats::default(),
        }
    }

    /// Add decree
    pub fn add_decree(&mut self, decree: RealmDecree) -> bool {
        if self.decrees.len() >= self.config.max_decrees {
            return false;
        }
        self.decrees.push(decree);
        self.update_stats();
        true
    }

    /// Get decree
    pub fn get_decree(&self, id: &str) -> Option<&RealmDecree> {
        self.decrees.iter().find(|d| d.id == id)
    }

    /// Get decree mut
    pub fn get_decree_mut(&mut self, id: &str) -> Option<&mut RealmDecree> {
        self.decrees.iter_mut().find(|d| d.id == id)
    }

    /// Add vassal
    pub fn add_vassal(&mut self, vassal: RealmVassal) {
        self.vassals.push(vassal);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.decrees, self.config.realm_type);
    }

    /// Get stats
    pub fn stats(&self) -> &RealmStats {
        &self.stats
    }

    /// Decree count
    pub fn decree_count(&self) -> usize {
        self.decrees.len()
    }
}

/// Realm registry
#[derive(Debug, Clone, Default)]
pub struct RealmRegistry {
    /// Realms by ID
    realms: HashMap<String, SettingsRealm>,
}

impl RealmRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register realm
    pub fn register(&mut self, id: impl Into<String>, realm: SettingsRealm) {
        self.realms.insert(id.into(), realm);
    }

    /// Unregister realm
    pub fn unregister(&mut self, id: &str) -> bool {
        self.realms.remove(id).is_some()
    }

    /// Get realm
    pub fn get(&self, id: &str) -> Option<&SettingsRealm> {
        self.realms.get(id)
    }

    /// Get realm mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsRealm> {
        self.realms.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.realms.len()
    }
}

/// Format realm registry
pub fn format_realm_registry(registry: &RealmRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Realm Registry:\n");
    output.push_str(&format!("  Realms: {}\n", registry.count()));
    output
}

/// Check if query is about realm
pub fn is_realm_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings realm") || lower.contains("realm settings") || lower.contains("royal realm")
}

/// Fun fact about realm
pub fn realm_fun_fact() -> &'static str {
    "Anna's settings realm establishes royal sovereignty!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_realm_type_display() {
        assert_eq!(format!("{}", RealmType::Kingdom), "kingdom");
        assert_eq!(format!("{}", RealmType::Empire), "empire");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", RealmStatus::Rising), "rising");
        assert_eq!(format!("{}", RealmStatus::Prosperous), "prosperous");
    }

    #[test]
    fn test_config_new() {
        let c = RealmConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = RealmConfig::new("test")
            .realm_type(RealmType::Empire)
            .status(RealmStatus::Prosperous);
        assert_eq!(c.realm_type, RealmType::Empire);
        assert_eq!(c.status, RealmStatus::Prosperous);
    }

    #[test]
    fn test_decree_new() {
        let d = RealmDecree::new("d1", "Title", "Content");
        assert_eq!(d.id, "d1");
    }

    #[test]
    fn test_decree_builder() {
        let d = RealmDecree::new("d1", "Title", "Content")
            .order(1);
        assert_eq!(d.order, 1);
    }

    #[test]
    fn test_decree_royal() {
        let mut d = RealmDecree::new("d1", "Title", "Content");
        d.make_common();
        assert!(!d.royal);
        d.make_royal();
        assert!(d.royal);
    }

    #[test]
    fn test_vassal_new() {
        let v = RealmVassal::new("key", "name", "d1");
        assert_eq!(v.decree_id, "d1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = RealmStats::default();
        let decree = RealmDecree::new("d1", "Title", "Content");
        s.update(&[decree], RealmType::Kingdom);
        assert_eq!(s.total_decrees, 1);
        assert_eq!(s.royal, 1);
    }

    #[test]
    fn test_realm_new() {
        let r = SettingsRealm::new(RealmConfig::default());
        assert_eq!(r.decree_count(), 0);
    }

    #[test]
    fn test_realm_add_decree() {
        let mut r = SettingsRealm::new(RealmConfig::default());
        r.add_decree(RealmDecree::new("d1", "Title", "Content"));
        assert_eq!(r.decree_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = RealmRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = RealmRegistry::new();
        r.register("r1", SettingsRealm::new(RealmConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_realm_query() {
        assert!(is_realm_query("settings realm"));
        assert!(!is_realm_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = realm_fun_fact();
        assert!(fact.contains("realm"));
    }
}
