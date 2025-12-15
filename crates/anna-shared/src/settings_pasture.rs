// v0.0.764: Settings Pasture (Phase 340)
// Grazing pasture for settings livestock

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pasture type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PastureType {
    /// Permanent pasture
    #[default]
    Permanent,
    /// Rotational pasture
    Rotational,
    /// Intensive pasture
    Intensive,
    /// Rough pasture
    Rough,
}

impl std::fmt::Display for PastureType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Permanent => write!(f, "permanent"),
            Self::Rotational => write!(f, "rotational"),
            Self::Intensive => write!(f, "intensive"),
            Self::Rough => write!(f, "rough"),
        }
    }
}

/// Pasture status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PastureStatus {
    /// Open status
    #[default]
    Open,
    /// Grazed status
    Grazed,
    /// Rested status
    Rested,
    /// Improved status
    Improved,
}

impl std::fmt::Display for PastureStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::Grazed => write!(f, "grazed"),
            Self::Rested => write!(f, "rested"),
            Self::Improved => write!(f, "improved"),
        }
    }
}

/// Pasture config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastureConfig {
    /// Name
    pub name: String,
    /// Pasture type
    pub pasture_type: PastureType,
    /// Status
    pub status: PastureStatus,
    /// Max herds
    pub max_herds: usize,
}

impl PastureConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            pasture_type: PastureType::Permanent,
            status: PastureStatus::Open,
            max_herds: 100,
        }
    }

    /// Set type
    pub fn pasture_type(mut self, pt: PastureType) -> Self {
        self.pasture_type = pt;
        self
    }

    /// Set status
    pub fn status(mut self, s: PastureStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max herds
    pub fn max_herds(mut self, max: usize) -> Self {
        self.max_herds = max;
        self
    }
}

impl Default for PastureConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Pasture herd
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastureHerd {
    /// Herd ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Paddock number
    pub paddock: u32,
    /// Thriving
    pub thriving: bool,
}

impl PastureHerd {
    /// Create new herd
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            paddock: 0,
            thriving: true,
        }
    }

    /// Set paddock
    pub fn paddock(mut self, p: u32) -> Self {
        self.paddock = p;
        self
    }

    /// Make thriving
    pub fn make_thriving(&mut self) {
        self.thriving = true;
    }

    /// Make struggling
    pub fn make_struggling(&mut self) {
        self.thriving = false;
    }
}

/// Pasture herder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastureHerder {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Herd ID
    pub herd_id: String,
}

impl PastureHerder {
    /// Create new herder
    pub fn new(key: impl Into<String>, name: impl Into<String>, herd_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            herd_id: herd_id.into(),
        }
    }
}

/// Pasture stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PastureStats {
    /// Total herds
    pub total_herds: usize,
    /// Thriving herds
    pub thriving: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl PastureStats {
    /// Update from herds
    pub fn update(&mut self, herds: &[PastureHerd], pasture_type: PastureType) {
        self.total_herds = herds.len();
        self.thriving = herds.iter().filter(|h| h.thriving).count();
        *self.by_type.entry(pasture_type.to_string()).or_insert(0) += 1;
    }

    /// Thriving rate
    pub fn thriving_rate(&self) -> f64 {
        if self.total_herds == 0 { 0.0 } else { self.thriving as f64 / self.total_herds as f64 * 100.0 }
    }
}

/// Settings pasture
#[derive(Debug, Clone, Default)]
pub struct SettingsPasture {
    /// Config
    config: PastureConfig,
    /// Herds
    herds: Vec<PastureHerd>,
    /// Herders
    herders: Vec<PastureHerder>,
    /// Stats
    stats: PastureStats,
}

impl SettingsPasture {
    /// Create new pasture system
    pub fn new(config: PastureConfig) -> Self {
        Self {
            config,
            herds: Vec::new(),
            herders: Vec::new(),
            stats: PastureStats::default(),
        }
    }

    /// Add herd
    pub fn add_herd(&mut self, herd: PastureHerd) -> bool {
        if self.herds.len() >= self.config.max_herds {
            return false;
        }
        self.herds.push(herd);
        self.update_stats();
        true
    }

    /// Get herd
    pub fn get_herd(&self, id: &str) -> Option<&PastureHerd> {
        self.herds.iter().find(|h| h.id == id)
    }

    /// Get herd mut
    pub fn get_herd_mut(&mut self, id: &str) -> Option<&mut PastureHerd> {
        self.herds.iter_mut().find(|h| h.id == id)
    }

    /// Add herder
    pub fn add_herder(&mut self, herder: PastureHerder) {
        self.herders.push(herder);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.herds, self.config.pasture_type);
    }

    /// Get stats
    pub fn stats(&self) -> &PastureStats {
        &self.stats
    }

    /// Herd count
    pub fn herd_count(&self) -> usize {
        self.herds.len()
    }
}

/// Pasture registry
#[derive(Debug, Clone, Default)]
pub struct PastureRegistry {
    /// Pastures by ID
    pastures: HashMap<String, SettingsPasture>,
}

impl PastureRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register pasture
    pub fn register(&mut self, id: impl Into<String>, pasture: SettingsPasture) {
        self.pastures.insert(id.into(), pasture);
    }

    /// Unregister pasture
    pub fn unregister(&mut self, id: &str) -> bool {
        self.pastures.remove(id).is_some()
    }

    /// Get pasture
    pub fn get(&self, id: &str) -> Option<&SettingsPasture> {
        self.pastures.get(id)
    }

    /// Get pasture mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsPasture> {
        self.pastures.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.pastures.len()
    }
}

/// Format pasture registry
pub fn format_pasture_registry(registry: &PastureRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Pasture Registry:\n");
    output.push_str(&format!("  Pastures: {}\n", registry.count()));
    output
}

/// Check if query is about pasture
pub fn is_pasture_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings pasture") || lower.contains("pasture settings") || lower.contains("grazing pasture")
}

/// Fun fact about pasture
pub fn pasture_fun_fact() -> &'static str {
    "Anna's settings pasture establishes livestock boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pasture_type_display() {
        assert_eq!(format!("{}", PastureType::Permanent), "permanent");
        assert_eq!(format!("{}", PastureType::Rotational), "rotational");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", PastureStatus::Open), "open");
        assert_eq!(format!("{}", PastureStatus::Grazed), "grazed");
    }

    #[test]
    fn test_config_new() {
        let c = PastureConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = PastureConfig::new("test")
            .pasture_type(PastureType::Intensive)
            .status(PastureStatus::Rested);
        assert_eq!(c.pasture_type, PastureType::Intensive);
        assert_eq!(c.status, PastureStatus::Rested);
    }

    #[test]
    fn test_herd_new() {
        let h = PastureHerd::new("h1", "Title", "Content");
        assert_eq!(h.id, "h1");
    }

    #[test]
    fn test_herd_builder() {
        let h = PastureHerd::new("h1", "Title", "Content")
            .paddock(1);
        assert_eq!(h.paddock, 1);
    }

    #[test]
    fn test_herd_thriving() {
        let mut h = PastureHerd::new("h1", "Title", "Content");
        h.make_struggling();
        assert!(!h.thriving);
        h.make_thriving();
        assert!(h.thriving);
    }

    #[test]
    fn test_herder_new() {
        let h = PastureHerder::new("key", "name", "h1");
        assert_eq!(h.herd_id, "h1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = PastureStats::default();
        let herd = PastureHerd::new("h1", "Title", "Content");
        s.update(&[herd], PastureType::Permanent);
        assert_eq!(s.total_herds, 1);
        assert_eq!(s.thriving, 1);
    }

    #[test]
    fn test_pasture_new() {
        let p = SettingsPasture::new(PastureConfig::default());
        assert_eq!(p.herd_count(), 0);
    }

    #[test]
    fn test_pasture_add_herd() {
        let mut p = SettingsPasture::new(PastureConfig::default());
        p.add_herd(PastureHerd::new("h1", "Title", "Content"));
        assert_eq!(p.herd_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = PastureRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = PastureRegistry::new();
        r.register("p1", SettingsPasture::new(PastureConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_pasture_query() {
        assert!(is_pasture_query("settings pasture"));
        assert!(!is_pasture_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = pasture_fun_fact();
        assert!(fact.contains("pasture"));
    }
}
