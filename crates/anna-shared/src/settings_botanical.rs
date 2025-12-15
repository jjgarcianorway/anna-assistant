// v0.0.773: Settings Botanical (Phase 349)
// Botanical garden for settings plant science

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Botanical type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BotanicalType {
    /// Display botanical
    #[default]
    Display,
    /// Research botanical
    Research,
    /// Conservation botanical
    Conservation,
    /// Educational botanical
    Educational,
}

impl std::fmt::Display for BotanicalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Display => write!(f, "display"),
            Self::Research => write!(f, "research"),
            Self::Conservation => write!(f, "conservation"),
            Self::Educational => write!(f, "educational"),
        }
    }
}

/// Botanical status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BotanicalStatus {
    /// Active status
    #[default]
    Active,
    /// Expanding status
    Expanding,
    /// Conserving status
    Conserving,
    /// Restoration status
    Restoration,
}

impl std::fmt::Display for BotanicalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Expanding => write!(f, "expanding"),
            Self::Conserving => write!(f, "conserving"),
            Self::Restoration => write!(f, "restoration"),
        }
    }
}

/// Botanical config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotanicalConfig {
    /// Name
    pub name: String,
    /// Botanical type
    pub botanical_type: BotanicalType,
    /// Status
    pub status: BotanicalStatus,
    /// Max collections
    pub max_collections: usize,
}

impl BotanicalConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            botanical_type: BotanicalType::Display,
            status: BotanicalStatus::Active,
            max_collections: 100,
        }
    }

    /// Set type
    pub fn botanical_type(mut self, bt: BotanicalType) -> Self {
        self.botanical_type = bt;
        self
    }

    /// Set status
    pub fn status(mut self, s: BotanicalStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max collections
    pub fn max_collections(mut self, max: usize) -> Self {
        self.max_collections = max;
        self
    }
}

impl Default for BotanicalConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Botanical collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotanicalCollection {
    /// Collection ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Wing number
    pub wing: u32,
    /// Documented
    pub documented: bool,
}

impl BotanicalCollection {
    /// Create new collection
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            wing: 0,
            documented: true,
        }
    }

    /// Set wing
    pub fn wing(mut self, w: u32) -> Self {
        self.wing = w;
        self
    }

    /// Make documented
    pub fn make_documented(&mut self) {
        self.documented = true;
    }

    /// Make undocumented
    pub fn make_undocumented(&mut self) {
        self.documented = false;
    }
}

/// Botanical botanist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotanicalBotanist {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Collection ID
    pub collection_id: String,
}

impl BotanicalBotanist {
    /// Create new botanist
    pub fn new(key: impl Into<String>, name: impl Into<String>, collection_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            collection_id: collection_id.into(),
        }
    }
}

/// Botanical stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BotanicalStats {
    /// Total collections
    pub total_collections: usize,
    /// Documented collections
    pub documented: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl BotanicalStats {
    /// Update from collections
    pub fn update(&mut self, collections: &[BotanicalCollection], botanical_type: BotanicalType) {
        self.total_collections = collections.len();
        self.documented = collections.iter().filter(|c| c.documented).count();
        *self.by_type.entry(botanical_type.to_string()).or_insert(0) += 1;
    }

    /// Documentation rate
    pub fn documentation_rate(&self) -> f64 {
        if self.total_collections == 0 { 0.0 } else { self.documented as f64 / self.total_collections as f64 * 100.0 }
    }
}

/// Settings botanical
#[derive(Debug, Clone, Default)]
pub struct SettingsBotanical {
    /// Config
    config: BotanicalConfig,
    /// Collections
    collections: Vec<BotanicalCollection>,
    /// Botanists
    botanists: Vec<BotanicalBotanist>,
    /// Stats
    stats: BotanicalStats,
}

impl SettingsBotanical {
    /// Create new botanical system
    pub fn new(config: BotanicalConfig) -> Self {
        Self {
            config,
            collections: Vec::new(),
            botanists: Vec::new(),
            stats: BotanicalStats::default(),
        }
    }

    /// Add collection
    pub fn add_collection(&mut self, collection: BotanicalCollection) -> bool {
        if self.collections.len() >= self.config.max_collections {
            return false;
        }
        self.collections.push(collection);
        self.update_stats();
        true
    }

    /// Get collection
    pub fn get_collection(&self, id: &str) -> Option<&BotanicalCollection> {
        self.collections.iter().find(|c| c.id == id)
    }

    /// Get collection mut
    pub fn get_collection_mut(&mut self, id: &str) -> Option<&mut BotanicalCollection> {
        self.collections.iter_mut().find(|c| c.id == id)
    }

    /// Add botanist
    pub fn add_botanist(&mut self, botanist: BotanicalBotanist) {
        self.botanists.push(botanist);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.collections, self.config.botanical_type);
    }

    /// Get stats
    pub fn stats(&self) -> &BotanicalStats {
        &self.stats
    }

    /// Collection count
    pub fn collection_count(&self) -> usize {
        self.collections.len()
    }
}

/// Botanical registry
#[derive(Debug, Clone, Default)]
pub struct BotanicalRegistry {
    /// Botanicals by ID
    botanicals: HashMap<String, SettingsBotanical>,
}

impl BotanicalRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register botanical
    pub fn register(&mut self, id: impl Into<String>, botanical: SettingsBotanical) {
        self.botanicals.insert(id.into(), botanical);
    }

    /// Unregister botanical
    pub fn unregister(&mut self, id: &str) -> bool {
        self.botanicals.remove(id).is_some()
    }

    /// Get botanical
    pub fn get(&self, id: &str) -> Option<&SettingsBotanical> {
        self.botanicals.get(id)
    }

    /// Get botanical mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsBotanical> {
        self.botanicals.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.botanicals.len()
    }
}

/// Format botanical registry
pub fn format_botanical_registry(registry: &BotanicalRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Botanical Registry:\n");
    output.push_str(&format!("  Botanicals: {}\n", registry.count()));
    output
}

/// Check if query is about botanical
pub fn is_botanical_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings botanical") || lower.contains("botanical settings") || lower.contains("botanical garden")
}

/// Fun fact about botanical
pub fn botanical_fun_fact() -> &'static str {
    "Anna's settings botanical documents plant science boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_botanical_type_display() {
        assert_eq!(format!("{}", BotanicalType::Display), "display");
        assert_eq!(format!("{}", BotanicalType::Research), "research");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", BotanicalStatus::Active), "active");
        assert_eq!(format!("{}", BotanicalStatus::Restoration), "restoration");
    }

    #[test]
    fn test_config_new() {
        let c = BotanicalConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = BotanicalConfig::new("test")
            .botanical_type(BotanicalType::Conservation)
            .status(BotanicalStatus::Expanding);
        assert_eq!(c.botanical_type, BotanicalType::Conservation);
        assert_eq!(c.status, BotanicalStatus::Expanding);
    }

    #[test]
    fn test_collection_new() {
        let c = BotanicalCollection::new("c1", "Title", "Content");
        assert_eq!(c.id, "c1");
    }

    #[test]
    fn test_collection_builder() {
        let c = BotanicalCollection::new("c1", "Title", "Content")
            .wing(1);
        assert_eq!(c.wing, 1);
    }

    #[test]
    fn test_collection_documented() {
        let mut c = BotanicalCollection::new("c1", "Title", "Content");
        c.make_undocumented();
        assert!(!c.documented);
        c.make_documented();
        assert!(c.documented);
    }

    #[test]
    fn test_botanist_new() {
        let b = BotanicalBotanist::new("key", "name", "c1");
        assert_eq!(b.collection_id, "c1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = BotanicalStats::default();
        let collection = BotanicalCollection::new("c1", "Title", "Content");
        s.update(&[collection], BotanicalType::Display);
        assert_eq!(s.total_collections, 1);
        assert_eq!(s.documented, 1);
    }

    #[test]
    fn test_botanical_new() {
        let b = SettingsBotanical::new(BotanicalConfig::default());
        assert_eq!(b.collection_count(), 0);
    }

    #[test]
    fn test_botanical_add_collection() {
        let mut b = SettingsBotanical::new(BotanicalConfig::default());
        b.add_collection(BotanicalCollection::new("c1", "Title", "Content"));
        assert_eq!(b.collection_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = BotanicalRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = BotanicalRegistry::new();
        r.register("b1", SettingsBotanical::new(BotanicalConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_botanical_query() {
        assert!(is_botanical_query("settings botanical"));
        assert!(!is_botanical_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = botanical_fun_fact();
        assert!(fact.contains("botanical"));
    }
}
