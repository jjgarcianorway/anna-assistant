// v0.0.751: Settings Borough (Phase 327)
// Borough subdivision for settings local governance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Borough type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BoroughType {
    /// Urban borough
    #[default]
    Urban,
    /// Metropolitan borough
    Metropolitan,
    /// London borough
    London,
    /// Municipal borough
    Municipal,
}

impl std::fmt::Display for BoroughType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Urban => write!(f, "urban"),
            Self::Metropolitan => write!(f, "metropolitan"),
            Self::London => write!(f, "london"),
            Self::Municipal => write!(f, "municipal"),
        }
    }
}

/// Borough status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BoroughStatus {
    /// Established status
    #[default]
    Established,
    /// Active status
    Active,
    /// Reformed status
    Reformed,
    /// Abolished status
    Abolished,
}

impl std::fmt::Display for BoroughStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Established => write!(f, "established"),
            Self::Active => write!(f, "active"),
            Self::Reformed => write!(f, "reformed"),
            Self::Abolished => write!(f, "abolished"),
        }
    }
}

/// Borough config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoroughConfig {
    /// Name
    pub name: String,
    /// Borough type
    pub borough_type: BoroughType,
    /// Status
    pub status: BoroughStatus,
    /// Max resolutions
    pub max_resolutions: usize,
}

impl BoroughConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            borough_type: BoroughType::Urban,
            status: BoroughStatus::Established,
            max_resolutions: 100,
        }
    }

    /// Set type
    pub fn borough_type(mut self, bt: BoroughType) -> Self {
        self.borough_type = bt;
        self
    }

    /// Set status
    pub fn status(mut self, s: BoroughStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max resolutions
    pub fn max_resolutions(mut self, max: usize) -> Self {
        self.max_resolutions = max;
        self
    }
}

impl Default for BoroughConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Borough resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoroughResolution {
    /// Resolution ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Section number
    pub section: u32,
    /// Adopted
    pub adopted: bool,
}

impl BoroughResolution {
    /// Create new resolution
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            section: 0,
            adopted: true,
        }
    }

    /// Set section
    pub fn section(mut self, s: u32) -> Self {
        self.section = s;
        self
    }

    /// Make adopted
    pub fn make_adopted(&mut self) {
        self.adopted = true;
    }

    /// Make rescinded
    pub fn make_rescinded(&mut self) {
        self.adopted = false;
    }
}

/// Borough representative
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoroughRepresentative {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Resolution ID
    pub resolution_id: String,
}

impl BoroughRepresentative {
    /// Create new representative
    pub fn new(key: impl Into<String>, name: impl Into<String>, resolution_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            resolution_id: resolution_id.into(),
        }
    }
}

/// Borough stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BoroughStats {
    /// Total resolutions
    pub total_resolutions: usize,
    /// Adopted resolutions
    pub adopted: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl BoroughStats {
    /// Update from resolutions
    pub fn update(&mut self, resolutions: &[BoroughResolution], borough_type: BoroughType) {
        self.total_resolutions = resolutions.len();
        self.adopted = resolutions.iter().filter(|r| r.adopted).count();
        *self.by_type.entry(borough_type.to_string()).or_insert(0) += 1;
    }

    /// Adopted rate
    pub fn adopted_rate(&self) -> f64 {
        if self.total_resolutions == 0 { 0.0 } else { self.adopted as f64 / self.total_resolutions as f64 * 100.0 }
    }
}

/// Settings borough
#[derive(Debug, Clone, Default)]
pub struct SettingsBorough {
    /// Config
    config: BoroughConfig,
    /// Resolutions
    resolutions: Vec<BoroughResolution>,
    /// Representatives
    representatives: Vec<BoroughRepresentative>,
    /// Stats
    stats: BoroughStats,
}

impl SettingsBorough {
    /// Create new borough system
    pub fn new(config: BoroughConfig) -> Self {
        Self {
            config,
            resolutions: Vec::new(),
            representatives: Vec::new(),
            stats: BoroughStats::default(),
        }
    }

    /// Add resolution
    pub fn add_resolution(&mut self, resolution: BoroughResolution) -> bool {
        if self.resolutions.len() >= self.config.max_resolutions {
            return false;
        }
        self.resolutions.push(resolution);
        self.update_stats();
        true
    }

    /// Get resolution
    pub fn get_resolution(&self, id: &str) -> Option<&BoroughResolution> {
        self.resolutions.iter().find(|r| r.id == id)
    }

    /// Get resolution mut
    pub fn get_resolution_mut(&mut self, id: &str) -> Option<&mut BoroughResolution> {
        self.resolutions.iter_mut().find(|r| r.id == id)
    }

    /// Add representative
    pub fn add_representative(&mut self, representative: BoroughRepresentative) {
        self.representatives.push(representative);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.resolutions, self.config.borough_type);
    }

    /// Get stats
    pub fn stats(&self) -> &BoroughStats {
        &self.stats
    }

    /// Resolution count
    pub fn resolution_count(&self) -> usize {
        self.resolutions.len()
    }
}

/// Borough registry
#[derive(Debug, Clone, Default)]
pub struct BoroughRegistry {
    /// Boroughs by ID
    boroughs: HashMap<String, SettingsBorough>,
}

impl BoroughRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register borough
    pub fn register(&mut self, id: impl Into<String>, borough: SettingsBorough) {
        self.boroughs.insert(id.into(), borough);
    }

    /// Unregister borough
    pub fn unregister(&mut self, id: &str) -> bool {
        self.boroughs.remove(id).is_some()
    }

    /// Get borough
    pub fn get(&self, id: &str) -> Option<&SettingsBorough> {
        self.boroughs.get(id)
    }

    /// Get borough mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsBorough> {
        self.boroughs.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.boroughs.len()
    }
}

/// Format borough registry
pub fn format_borough_registry(registry: &BoroughRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Borough Registry:\n");
    output.push_str(&format!("  Boroughs: {}\n", registry.count()));
    output
}

/// Check if query is about borough
pub fn is_borough_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings borough") || lower.contains("borough settings") || lower.contains("borough subdivision")
}

/// Fun fact about borough
pub fn borough_fun_fact() -> &'static str {
    "Anna's settings borough establishes local subdivision governance!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_borough_type_display() {
        assert_eq!(format!("{}", BoroughType::Urban), "urban");
        assert_eq!(format!("{}", BoroughType::Metropolitan), "metropolitan");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", BoroughStatus::Established), "established");
        assert_eq!(format!("{}", BoroughStatus::Active), "active");
    }

    #[test]
    fn test_config_new() {
        let c = BoroughConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = BoroughConfig::new("test")
            .borough_type(BoroughType::London)
            .status(BoroughStatus::Reformed);
        assert_eq!(c.borough_type, BoroughType::London);
        assert_eq!(c.status, BoroughStatus::Reformed);
    }

    #[test]
    fn test_resolution_new() {
        let r = BoroughResolution::new("r1", "Title", "Content");
        assert_eq!(r.id, "r1");
    }

    #[test]
    fn test_resolution_builder() {
        let r = BoroughResolution::new("r1", "Title", "Content")
            .section(1);
        assert_eq!(r.section, 1);
    }

    #[test]
    fn test_resolution_adopted() {
        let mut r = BoroughResolution::new("r1", "Title", "Content");
        r.make_rescinded();
        assert!(!r.adopted);
        r.make_adopted();
        assert!(r.adopted);
    }

    #[test]
    fn test_representative_new() {
        let r = BoroughRepresentative::new("key", "name", "r1");
        assert_eq!(r.resolution_id, "r1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = BoroughStats::default();
        let resolution = BoroughResolution::new("r1", "Title", "Content");
        s.update(&[resolution], BoroughType::Urban);
        assert_eq!(s.total_resolutions, 1);
        assert_eq!(s.adopted, 1);
    }

    #[test]
    fn test_borough_new() {
        let b = SettingsBorough::new(BoroughConfig::default());
        assert_eq!(b.resolution_count(), 0);
    }

    #[test]
    fn test_borough_add_resolution() {
        let mut b = SettingsBorough::new(BoroughConfig::default());
        b.add_resolution(BoroughResolution::new("r1", "Title", "Content"));
        assert_eq!(b.resolution_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = BoroughRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = BoroughRegistry::new();
        r.register("b1", SettingsBorough::new(BoroughConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_borough_query() {
        assert!(is_borough_query("settings borough"));
        assert!(!is_borough_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = borough_fun_fact();
        assert!(fact.contains("borough"));
    }
}
