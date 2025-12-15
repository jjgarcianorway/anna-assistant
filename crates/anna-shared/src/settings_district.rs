// v0.0.748: Settings District (Phase 324)
// Local district for settings administration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// District type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DistrictType {
    /// Urban district
    #[default]
    Urban,
    /// Rural district
    Rural,
    /// Industrial district
    Industrial,
    /// Commercial district
    Commercial,
}

impl std::fmt::Display for DistrictType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Urban => write!(f, "urban"),
            Self::Rural => write!(f, "rural"),
            Self::Industrial => write!(f, "industrial"),
            Self::Commercial => write!(f, "commercial"),
        }
    }
}

/// District status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DistrictStatus {
    /// Planned status
    #[default]
    Planned,
    /// Operational status
    Operational,
    /// Developing status
    Developing,
    /// Restructuring status
    Restructuring,
}

impl std::fmt::Display for DistrictStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Planned => write!(f, "planned"),
            Self::Operational => write!(f, "operational"),
            Self::Developing => write!(f, "developing"),
            Self::Restructuring => write!(f, "restructuring"),
        }
    }
}

/// District config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistrictConfig {
    /// Name
    pub name: String,
    /// District type
    pub district_type: DistrictType,
    /// Status
    pub status: DistrictStatus,
    /// Max bylaws
    pub max_bylaws: usize,
}

impl DistrictConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            district_type: DistrictType::Urban,
            status: DistrictStatus::Planned,
            max_bylaws: 100,
        }
    }

    /// Set type
    pub fn district_type(mut self, dt: DistrictType) -> Self {
        self.district_type = dt;
        self
    }

    /// Set status
    pub fn status(mut self, s: DistrictStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max bylaws
    pub fn max_bylaws(mut self, max: usize) -> Self {
        self.max_bylaws = max;
        self
    }
}

impl Default for DistrictConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// District bylaw
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistrictBylaw {
    /// Bylaw ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Ward number
    pub ward: u32,
    /// Active
    pub active: bool,
}

impl DistrictBylaw {
    /// Create new bylaw
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            ward: 0,
            active: true,
        }
    }

    /// Set ward
    pub fn ward(mut self, w: u32) -> Self {
        self.ward = w;
        self
    }

    /// Make active
    pub fn make_active(&mut self) {
        self.active = true;
    }

    /// Make inactive
    pub fn make_inactive(&mut self) {
        self.active = false;
    }
}

/// District official
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistrictOfficial {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Bylaw ID
    pub bylaw_id: String,
}

impl DistrictOfficial {
    /// Create new official
    pub fn new(key: impl Into<String>, name: impl Into<String>, bylaw_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            bylaw_id: bylaw_id.into(),
        }
    }
}

/// District stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DistrictStats {
    /// Total bylaws
    pub total_bylaws: usize,
    /// Active bylaws
    pub active: usize,
    /// Operational count
    pub operational_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl DistrictStats {
    /// Update from bylaws
    pub fn update(&mut self, bylaws: &[DistrictBylaw], district_type: DistrictType) {
        self.total_bylaws = bylaws.len();
        self.active = bylaws.iter().filter(|b| b.active).count();
        *self.by_type.entry(district_type.to_string()).or_insert(0) += 1;
    }

    /// Active rate
    pub fn active_rate(&self) -> f64 {
        if self.total_bylaws == 0 { 0.0 } else { self.active as f64 / self.total_bylaws as f64 * 100.0 }
    }
}

/// Settings district
#[derive(Debug, Clone, Default)]
pub struct SettingsDistrict {
    /// Config
    config: DistrictConfig,
    /// Bylaws
    bylaws: Vec<DistrictBylaw>,
    /// Officials
    officials: Vec<DistrictOfficial>,
    /// Stats
    stats: DistrictStats,
}

impl SettingsDistrict {
    /// Create new district system
    pub fn new(config: DistrictConfig) -> Self {
        Self {
            config,
            bylaws: Vec::new(),
            officials: Vec::new(),
            stats: DistrictStats::default(),
        }
    }

    /// Add bylaw
    pub fn add_bylaw(&mut self, bylaw: DistrictBylaw) -> bool {
        if self.bylaws.len() >= self.config.max_bylaws {
            return false;
        }
        self.bylaws.push(bylaw);
        self.update_stats();
        true
    }

    /// Get bylaw
    pub fn get_bylaw(&self, id: &str) -> Option<&DistrictBylaw> {
        self.bylaws.iter().find(|b| b.id == id)
    }

    /// Get bylaw mut
    pub fn get_bylaw_mut(&mut self, id: &str) -> Option<&mut DistrictBylaw> {
        self.bylaws.iter_mut().find(|b| b.id == id)
    }

    /// Add official
    pub fn add_official(&mut self, official: DistrictOfficial) {
        self.officials.push(official);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.bylaws, self.config.district_type);
    }

    /// Get stats
    pub fn stats(&self) -> &DistrictStats {
        &self.stats
    }

    /// Bylaw count
    pub fn bylaw_count(&self) -> usize {
        self.bylaws.len()
    }
}

/// District registry
#[derive(Debug, Clone, Default)]
pub struct DistrictRegistry {
    /// Districts by ID
    districts: HashMap<String, SettingsDistrict>,
}

impl DistrictRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register district
    pub fn register(&mut self, id: impl Into<String>, district: SettingsDistrict) {
        self.districts.insert(id.into(), district);
    }

    /// Unregister district
    pub fn unregister(&mut self, id: &str) -> bool {
        self.districts.remove(id).is_some()
    }

    /// Get district
    pub fn get(&self, id: &str) -> Option<&SettingsDistrict> {
        self.districts.get(id)
    }

    /// Get district mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsDistrict> {
        self.districts.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.districts.len()
    }
}

/// Format district registry
pub fn format_district_registry(registry: &DistrictRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings District Registry:\n");
    output.push_str(&format!("  Districts: {}\n", registry.count()));
    output
}

/// Check if query is about district
pub fn is_district_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings district") || lower.contains("district settings") || lower.contains("local district")
}

/// Fun fact about district
pub fn district_fun_fact() -> &'static str {
    "Anna's settings district establishes local administration!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_district_type_display() {
        assert_eq!(format!("{}", DistrictType::Urban), "urban");
        assert_eq!(format!("{}", DistrictType::Rural), "rural");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", DistrictStatus::Planned), "planned");
        assert_eq!(format!("{}", DistrictStatus::Operational), "operational");
    }

    #[test]
    fn test_config_new() {
        let c = DistrictConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = DistrictConfig::new("test")
            .district_type(DistrictType::Industrial)
            .status(DistrictStatus::Developing);
        assert_eq!(c.district_type, DistrictType::Industrial);
        assert_eq!(c.status, DistrictStatus::Developing);
    }

    #[test]
    fn test_bylaw_new() {
        let b = DistrictBylaw::new("b1", "Title", "Content");
        assert_eq!(b.id, "b1");
    }

    #[test]
    fn test_bylaw_builder() {
        let b = DistrictBylaw::new("b1", "Title", "Content")
            .ward(1);
        assert_eq!(b.ward, 1);
    }

    #[test]
    fn test_bylaw_active() {
        let mut b = DistrictBylaw::new("b1", "Title", "Content");
        b.make_inactive();
        assert!(!b.active);
        b.make_active();
        assert!(b.active);
    }

    #[test]
    fn test_official_new() {
        let o = DistrictOfficial::new("key", "name", "b1");
        assert_eq!(o.bylaw_id, "b1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = DistrictStats::default();
        let bylaw = DistrictBylaw::new("b1", "Title", "Content");
        s.update(&[bylaw], DistrictType::Urban);
        assert_eq!(s.total_bylaws, 1);
        assert_eq!(s.active, 1);
    }

    #[test]
    fn test_district_new() {
        let d = SettingsDistrict::new(DistrictConfig::default());
        assert_eq!(d.bylaw_count(), 0);
    }

    #[test]
    fn test_district_add_bylaw() {
        let mut d = SettingsDistrict::new(DistrictConfig::default());
        d.add_bylaw(DistrictBylaw::new("b1", "Title", "Content"));
        assert_eq!(d.bylaw_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = DistrictRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = DistrictRegistry::new();
        r.register("d1", SettingsDistrict::new(DistrictConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_district_query() {
        assert!(is_district_query("settings district"));
        assert!(!is_district_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = district_fun_fact();
        assert!(fact.contains("district"));
    }
}
