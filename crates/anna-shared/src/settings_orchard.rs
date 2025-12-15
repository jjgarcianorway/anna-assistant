// v0.0.766: Settings Orchard (Phase 342)
// Fruit orchard for settings horticulture

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Orchard type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum OrchardType {
    /// Apple orchard
    #[default]
    Apple,
    /// Cherry orchard
    Cherry,
    /// Peach orchard
    Peach,
    /// Pear orchard
    Pear,
}

impl std::fmt::Display for OrchardType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Apple => write!(f, "apple"),
            Self::Cherry => write!(f, "cherry"),
            Self::Peach => write!(f, "peach"),
            Self::Pear => write!(f, "pear"),
        }
    }
}

/// Orchard status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum OrchardStatus {
    /// Dormant status
    #[default]
    Dormant,
    /// Blooming status
    Blooming,
    /// Fruiting status
    Fruiting,
    /// Harvesting status
    Harvesting,
}

impl std::fmt::Display for OrchardStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dormant => write!(f, "dormant"),
            Self::Blooming => write!(f, "blooming"),
            Self::Fruiting => write!(f, "fruiting"),
            Self::Harvesting => write!(f, "harvesting"),
        }
    }
}

/// Orchard config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchardConfig {
    /// Name
    pub name: String,
    /// Orchard type
    pub orchard_type: OrchardType,
    /// Status
    pub status: OrchardStatus,
    /// Max fruits
    pub max_fruits: usize,
}

impl OrchardConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            orchard_type: OrchardType::Apple,
            status: OrchardStatus::Dormant,
            max_fruits: 100,
        }
    }

    /// Set type
    pub fn orchard_type(mut self, ot: OrchardType) -> Self {
        self.orchard_type = ot;
        self
    }

    /// Set status
    pub fn status(mut self, s: OrchardStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max fruits
    pub fn max_fruits(mut self, max: usize) -> Self {
        self.max_fruits = max;
        self
    }
}

impl Default for OrchardConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Orchard fruit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchardFruit {
    /// Fruit ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Branch number
    pub branch: u32,
    /// Ripe
    pub ripe: bool,
}

impl OrchardFruit {
    /// Create new fruit
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            branch: 0,
            ripe: true,
        }
    }

    /// Set branch
    pub fn branch(mut self, b: u32) -> Self {
        self.branch = b;
        self
    }

    /// Make ripe
    pub fn make_ripe(&mut self) {
        self.ripe = true;
    }

    /// Make unripe
    pub fn make_unripe(&mut self) {
        self.ripe = false;
    }
}

/// Orchard picker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchardPicker {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Fruit ID
    pub fruit_id: String,
}

impl OrchardPicker {
    /// Create new picker
    pub fn new(key: impl Into<String>, name: impl Into<String>, fruit_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            fruit_id: fruit_id.into(),
        }
    }
}

/// Orchard stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrchardStats {
    /// Total fruits
    pub total_fruits: usize,
    /// Ripe fruits
    pub ripe: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl OrchardStats {
    /// Update from fruits
    pub fn update(&mut self, fruits: &[OrchardFruit], orchard_type: OrchardType) {
        self.total_fruits = fruits.len();
        self.ripe = fruits.iter().filter(|f| f.ripe).count();
        *self.by_type.entry(orchard_type.to_string()).or_insert(0) += 1;
    }

    /// Ripe rate
    pub fn ripe_rate(&self) -> f64 {
        if self.total_fruits == 0 { 0.0 } else { self.ripe as f64 / self.total_fruits as f64 * 100.0 }
    }
}

/// Settings orchard
#[derive(Debug, Clone, Default)]
pub struct SettingsOrchard {
    /// Config
    config: OrchardConfig,
    /// Fruits
    fruits: Vec<OrchardFruit>,
    /// Pickers
    pickers: Vec<OrchardPicker>,
    /// Stats
    stats: OrchardStats,
}

impl SettingsOrchard {
    /// Create new orchard system
    pub fn new(config: OrchardConfig) -> Self {
        Self {
            config,
            fruits: Vec::new(),
            pickers: Vec::new(),
            stats: OrchardStats::default(),
        }
    }

    /// Add fruit
    pub fn add_fruit(&mut self, fruit: OrchardFruit) -> bool {
        if self.fruits.len() >= self.config.max_fruits {
            return false;
        }
        self.fruits.push(fruit);
        self.update_stats();
        true
    }

    /// Get fruit
    pub fn get_fruit(&self, id: &str) -> Option<&OrchardFruit> {
        self.fruits.iter().find(|f| f.id == id)
    }

    /// Get fruit mut
    pub fn get_fruit_mut(&mut self, id: &str) -> Option<&mut OrchardFruit> {
        self.fruits.iter_mut().find(|f| f.id == id)
    }

    /// Add picker
    pub fn add_picker(&mut self, picker: OrchardPicker) {
        self.pickers.push(picker);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.fruits, self.config.orchard_type);
    }

    /// Get stats
    pub fn stats(&self) -> &OrchardStats {
        &self.stats
    }

    /// Fruit count
    pub fn fruit_count(&self) -> usize {
        self.fruits.len()
    }
}

/// Orchard registry
#[derive(Debug, Clone, Default)]
pub struct OrchardRegistry {
    /// Orchards by ID
    orchards: HashMap<String, SettingsOrchard>,
}

impl OrchardRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register orchard
    pub fn register(&mut self, id: impl Into<String>, orchard: SettingsOrchard) {
        self.orchards.insert(id.into(), orchard);
    }

    /// Unregister orchard
    pub fn unregister(&mut self, id: &str) -> bool {
        self.orchards.remove(id).is_some()
    }

    /// Get orchard
    pub fn get(&self, id: &str) -> Option<&SettingsOrchard> {
        self.orchards.get(id)
    }

    /// Get orchard mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsOrchard> {
        self.orchards.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.orchards.len()
    }
}

/// Format orchard registry
pub fn format_orchard_registry(registry: &OrchardRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Orchard Registry:\n");
    output.push_str(&format!("  Orchards: {}\n", registry.count()));
    output
}

/// Check if query is about orchard
pub fn is_orchard_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings orchard") || lower.contains("orchard settings") || lower.contains("fruit orchard")
}

/// Fun fact about orchard
pub fn orchard_fun_fact() -> &'static str {
    "Anna's settings orchard establishes horticulture boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchard_type_display() {
        assert_eq!(format!("{}", OrchardType::Apple), "apple");
        assert_eq!(format!("{}", OrchardType::Cherry), "cherry");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", OrchardStatus::Dormant), "dormant");
        assert_eq!(format!("{}", OrchardStatus::Blooming), "blooming");
    }

    #[test]
    fn test_config_new() {
        let c = OrchardConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = OrchardConfig::new("test")
            .orchard_type(OrchardType::Peach)
            .status(OrchardStatus::Fruiting);
        assert_eq!(c.orchard_type, OrchardType::Peach);
        assert_eq!(c.status, OrchardStatus::Fruiting);
    }

    #[test]
    fn test_fruit_new() {
        let f = OrchardFruit::new("f1", "Title", "Content");
        assert_eq!(f.id, "f1");
    }

    #[test]
    fn test_fruit_builder() {
        let f = OrchardFruit::new("f1", "Title", "Content")
            .branch(1);
        assert_eq!(f.branch, 1);
    }

    #[test]
    fn test_fruit_ripe() {
        let mut f = OrchardFruit::new("f1", "Title", "Content");
        f.make_unripe();
        assert!(!f.ripe);
        f.make_ripe();
        assert!(f.ripe);
    }

    #[test]
    fn test_picker_new() {
        let p = OrchardPicker::new("key", "name", "f1");
        assert_eq!(p.fruit_id, "f1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = OrchardStats::default();
        let fruit = OrchardFruit::new("f1", "Title", "Content");
        s.update(&[fruit], OrchardType::Apple);
        assert_eq!(s.total_fruits, 1);
        assert_eq!(s.ripe, 1);
    }

    #[test]
    fn test_orchard_new() {
        let o = SettingsOrchard::new(OrchardConfig::default());
        assert_eq!(o.fruit_count(), 0);
    }

    #[test]
    fn test_orchard_add_fruit() {
        let mut o = SettingsOrchard::new(OrchardConfig::default());
        o.add_fruit(OrchardFruit::new("f1", "Title", "Content"));
        assert_eq!(o.fruit_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = OrchardRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = OrchardRegistry::new();
        r.register("o1", SettingsOrchard::new(OrchardConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_orchard_query() {
        assert!(is_orchard_query("settings orchard"));
        assert!(!is_orchard_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = orchard_fun_fact();
        assert!(fact.contains("orchard"));
    }
}
