// v0.0.779: Settings Apiary (Phase 355)
// Bee apiary for settings apiculture

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Apiary type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ApiaryType {
    /// Honey apiary
    #[default]
    Honey,
    /// Pollination apiary
    Pollination,
    /// Queen apiary
    Queen,
    /// Research apiary
    Research,
}

impl std::fmt::Display for ApiaryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Honey => write!(f, "honey"),
            Self::Pollination => write!(f, "pollination"),
            Self::Queen => write!(f, "queen"),
            Self::Research => write!(f, "research"),
        }
    }
}

/// Apiary status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ApiaryStatus {
    /// Active status
    #[default]
    Active,
    /// Swarming status
    Swarming,
    /// Harvesting status
    Harvesting,
    /// Wintering status
    Wintering,
}

impl std::fmt::Display for ApiaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Swarming => write!(f, "swarming"),
            Self::Harvesting => write!(f, "harvesting"),
            Self::Wintering => write!(f, "wintering"),
        }
    }
}

/// Apiary config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiaryConfig {
    /// Name
    pub name: String,
    /// Apiary type
    pub apiary_type: ApiaryType,
    /// Status
    pub status: ApiaryStatus,
    /// Max hives
    pub max_hives: usize,
}

impl ApiaryConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            apiary_type: ApiaryType::Honey,
            status: ApiaryStatus::Active,
            max_hives: 100,
        }
    }

    /// Set type
    pub fn apiary_type(mut self, at: ApiaryType) -> Self {
        self.apiary_type = at;
        self
    }

    /// Set status
    pub fn status(mut self, s: ApiaryStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max hives
    pub fn max_hives(mut self, max: usize) -> Self {
        self.max_hives = max;
        self
    }
}

impl Default for ApiaryConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Apiary hive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiaryHive {
    /// Hive ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Stand number
    pub stand: u32,
    /// Productive
    pub productive: bool,
}

impl ApiaryHive {
    /// Create new hive
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            stand: 0,
            productive: true,
        }
    }

    /// Set stand
    pub fn stand(mut self, s: u32) -> Self {
        self.stand = s;
        self
    }

    /// Make productive
    pub fn make_productive(&mut self) {
        self.productive = true;
    }

    /// Make dormant
    pub fn make_dormant(&mut self) {
        self.productive = false;
    }
}

/// Apiary beekeeper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiaryBeekeeper {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Hive ID
    pub hive_id: String,
}

impl ApiaryBeekeeper {
    /// Create new beekeeper
    pub fn new(key: impl Into<String>, name: impl Into<String>, hive_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            hive_id: hive_id.into(),
        }
    }
}

/// Apiary stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApiaryStats {
    /// Total hives
    pub total_hives: usize,
    /// Productive hives
    pub productive: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ApiaryStats {
    /// Update from hives
    pub fn update(&mut self, hives: &[ApiaryHive], apiary_type: ApiaryType) {
        self.total_hives = hives.len();
        self.productive = hives.iter().filter(|h| h.productive).count();
        *self.by_type.entry(apiary_type.to_string()).or_insert(0) += 1;
    }

    /// Productivity rate
    pub fn productivity_rate(&self) -> f64 {
        if self.total_hives == 0 { 0.0 } else { self.productive as f64 / self.total_hives as f64 * 100.0 }
    }
}

/// Settings apiary
#[derive(Debug, Clone, Default)]
pub struct SettingsApiary {
    /// Config
    config: ApiaryConfig,
    /// Hives
    hives: Vec<ApiaryHive>,
    /// Beekeepers
    beekeepers: Vec<ApiaryBeekeeper>,
    /// Stats
    stats: ApiaryStats,
}

impl SettingsApiary {
    /// Create new apiary system
    pub fn new(config: ApiaryConfig) -> Self {
        Self {
            config,
            hives: Vec::new(),
            beekeepers: Vec::new(),
            stats: ApiaryStats::default(),
        }
    }

    /// Add hive
    pub fn add_hive(&mut self, hive: ApiaryHive) -> bool {
        if self.hives.len() >= self.config.max_hives {
            return false;
        }
        self.hives.push(hive);
        self.update_stats();
        true
    }

    /// Get hive
    pub fn get_hive(&self, id: &str) -> Option<&ApiaryHive> {
        self.hives.iter().find(|h| h.id == id)
    }

    /// Get hive mut
    pub fn get_hive_mut(&mut self, id: &str) -> Option<&mut ApiaryHive> {
        self.hives.iter_mut().find(|h| h.id == id)
    }

    /// Add beekeeper
    pub fn add_beekeeper(&mut self, beekeeper: ApiaryBeekeeper) {
        self.beekeepers.push(beekeeper);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.hives, self.config.apiary_type);
    }

    /// Get stats
    pub fn stats(&self) -> &ApiaryStats {
        &self.stats
    }

    /// Hive count
    pub fn hive_count(&self) -> usize {
        self.hives.len()
    }
}

/// Apiary registry
#[derive(Debug, Clone, Default)]
pub struct ApiaryRegistry {
    /// Apiaries by ID
    apiaries: HashMap<String, SettingsApiary>,
}

impl ApiaryRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register apiary
    pub fn register(&mut self, id: impl Into<String>, apiary: SettingsApiary) {
        self.apiaries.insert(id.into(), apiary);
    }

    /// Unregister apiary
    pub fn unregister(&mut self, id: &str) -> bool {
        self.apiaries.remove(id).is_some()
    }

    /// Get apiary
    pub fn get(&self, id: &str) -> Option<&SettingsApiary> {
        self.apiaries.get(id)
    }

    /// Get apiary mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsApiary> {
        self.apiaries.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.apiaries.len()
    }
}

/// Format apiary registry
pub fn format_apiary_registry(registry: &ApiaryRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Apiary Registry:\n");
    output.push_str(&format!("  Apiaries: {}\n", registry.count()));
    output
}

/// Check if query is about apiary
pub fn is_apiary_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings apiary") || lower.contains("apiary settings") || lower.contains("bee apiary")
}

/// Fun fact about apiary
pub fn apiary_fun_fact() -> &'static str {
    "Anna's settings apiary buzzes with apiculture boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apiary_type_display() {
        assert_eq!(format!("{}", ApiaryType::Honey), "honey");
        assert_eq!(format!("{}", ApiaryType::Pollination), "pollination");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ApiaryStatus::Active), "active");
        assert_eq!(format!("{}", ApiaryStatus::Harvesting), "harvesting");
    }

    #[test]
    fn test_config_new() {
        let c = ApiaryConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ApiaryConfig::new("test")
            .apiary_type(ApiaryType::Queen)
            .status(ApiaryStatus::Swarming);
        assert_eq!(c.apiary_type, ApiaryType::Queen);
        assert_eq!(c.status, ApiaryStatus::Swarming);
    }

    #[test]
    fn test_hive_new() {
        let h = ApiaryHive::new("h1", "Title", "Content");
        assert_eq!(h.id, "h1");
    }

    #[test]
    fn test_hive_builder() {
        let h = ApiaryHive::new("h1", "Title", "Content")
            .stand(1);
        assert_eq!(h.stand, 1);
    }

    #[test]
    fn test_hive_productive() {
        let mut h = ApiaryHive::new("h1", "Title", "Content");
        h.make_dormant();
        assert!(!h.productive);
        h.make_productive();
        assert!(h.productive);
    }

    #[test]
    fn test_beekeeper_new() {
        let b = ApiaryBeekeeper::new("key", "name", "h1");
        assert_eq!(b.hive_id, "h1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = ApiaryStats::default();
        let hive = ApiaryHive::new("h1", "Title", "Content");
        s.update(&[hive], ApiaryType::Honey);
        assert_eq!(s.total_hives, 1);
        assert_eq!(s.productive, 1);
    }

    #[test]
    fn test_apiary_new() {
        let a = SettingsApiary::new(ApiaryConfig::default());
        assert_eq!(a.hive_count(), 0);
    }

    #[test]
    fn test_apiary_add_hive() {
        let mut a = SettingsApiary::new(ApiaryConfig::default());
        a.add_hive(ApiaryHive::new("h1", "Title", "Content"));
        assert_eq!(a.hive_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = ApiaryRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ApiaryRegistry::new();
        r.register("a1", SettingsApiary::new(ApiaryConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_apiary_query() {
        assert!(is_apiary_query("settings apiary"));
        assert!(!is_apiary_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = apiary_fun_fact();
        assert!(fact.contains("apiary"));
    }
}
