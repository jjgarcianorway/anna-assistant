// v0.0.746: Settings Province (Phase 322)
// Administrative province for settings governance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Province type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ProvinceType {
    /// Autonomous province
    #[default]
    Autonomous,
    /// Imperial province
    Imperial,
    /// Colonial province
    Colonial,
    /// Federal province
    Federal,
}

impl std::fmt::Display for ProvinceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Autonomous => write!(f, "autonomous"),
            Self::Imperial => write!(f, "imperial"),
            Self::Colonial => write!(f, "colonial"),
            Self::Federal => write!(f, "federal"),
        }
    }
}

/// Province status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ProvinceStatus {
    /// Established status
    #[default]
    Established,
    /// Developing status
    Developing,
    /// Integrated status
    Integrated,
    /// Reorganizing status
    Reorganizing,
}

impl std::fmt::Display for ProvinceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Established => write!(f, "established"),
            Self::Developing => write!(f, "developing"),
            Self::Integrated => write!(f, "integrated"),
            Self::Reorganizing => write!(f, "reorganizing"),
        }
    }
}

/// Province config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvinceConfig {
    /// Name
    pub name: String,
    /// Province type
    pub province_type: ProvinceType,
    /// Status
    pub status: ProvinceStatus,
    /// Max edicts
    pub max_edicts: usize,
}

impl ProvinceConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            province_type: ProvinceType::Autonomous,
            status: ProvinceStatus::Established,
            max_edicts: 100,
        }
    }

    /// Set type
    pub fn province_type(mut self, pt: ProvinceType) -> Self {
        self.province_type = pt;
        self
    }

    /// Set status
    pub fn status(mut self, s: ProvinceStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max edicts
    pub fn max_edicts(mut self, max: usize) -> Self {
        self.max_edicts = max;
        self
    }
}

impl Default for ProvinceConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Province edict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvinceEdict {
    /// Edict ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Prefecture number
    pub prefecture: u32,
    /// Provincial
    pub provincial: bool,
}

impl ProvinceEdict {
    /// Create new edict
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            prefecture: 0,
            provincial: true,
        }
    }

    /// Set prefecture
    pub fn prefecture(mut self, p: u32) -> Self {
        self.prefecture = p;
        self
    }

    /// Make provincial
    pub fn make_provincial(&mut self) {
        self.provincial = true;
    }

    /// Make local
    pub fn make_local(&mut self) {
        self.provincial = false;
    }
}

/// Province governor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvinceGovernor {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Edict ID
    pub edict_id: String,
}

impl ProvinceGovernor {
    /// Create new governor
    pub fn new(key: impl Into<String>, name: impl Into<String>, edict_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            edict_id: edict_id.into(),
        }
    }
}

/// Province stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProvinceStats {
    /// Total edicts
    pub total_edicts: usize,
    /// Provincial edicts
    pub provincial: usize,
    /// Integrated count
    pub integrated_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ProvinceStats {
    /// Update from edicts
    pub fn update(&mut self, edicts: &[ProvinceEdict], province_type: ProvinceType) {
        self.total_edicts = edicts.len();
        self.provincial = edicts.iter().filter(|e| e.provincial).count();
        *self.by_type.entry(province_type.to_string()).or_insert(0) += 1;
    }

    /// Provincial rate
    pub fn provincial_rate(&self) -> f64 {
        if self.total_edicts == 0 { 0.0 } else { self.provincial as f64 / self.total_edicts as f64 * 100.0 }
    }
}

/// Settings province
#[derive(Debug, Clone, Default)]
pub struct SettingsProvince {
    /// Config
    config: ProvinceConfig,
    /// Edicts
    edicts: Vec<ProvinceEdict>,
    /// Governors
    governors: Vec<ProvinceGovernor>,
    /// Stats
    stats: ProvinceStats,
}

impl SettingsProvince {
    /// Create new province system
    pub fn new(config: ProvinceConfig) -> Self {
        Self {
            config,
            edicts: Vec::new(),
            governors: Vec::new(),
            stats: ProvinceStats::default(),
        }
    }

    /// Add edict
    pub fn add_edict(&mut self, edict: ProvinceEdict) -> bool {
        if self.edicts.len() >= self.config.max_edicts {
            return false;
        }
        self.edicts.push(edict);
        self.update_stats();
        true
    }

    /// Get edict
    pub fn get_edict(&self, id: &str) -> Option<&ProvinceEdict> {
        self.edicts.iter().find(|e| e.id == id)
    }

    /// Get edict mut
    pub fn get_edict_mut(&mut self, id: &str) -> Option<&mut ProvinceEdict> {
        self.edicts.iter_mut().find(|e| e.id == id)
    }

    /// Add governor
    pub fn add_governor(&mut self, governor: ProvinceGovernor) {
        self.governors.push(governor);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.edicts, self.config.province_type);
    }

    /// Get stats
    pub fn stats(&self) -> &ProvinceStats {
        &self.stats
    }

    /// Edict count
    pub fn edict_count(&self) -> usize {
        self.edicts.len()
    }
}

/// Province registry
#[derive(Debug, Clone, Default)]
pub struct ProvinceRegistry {
    /// Provinces by ID
    provinces: HashMap<String, SettingsProvince>,
}

impl ProvinceRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register province
    pub fn register(&mut self, id: impl Into<String>, province: SettingsProvince) {
        self.provinces.insert(id.into(), province);
    }

    /// Unregister province
    pub fn unregister(&mut self, id: &str) -> bool {
        self.provinces.remove(id).is_some()
    }

    /// Get province
    pub fn get(&self, id: &str) -> Option<&SettingsProvince> {
        self.provinces.get(id)
    }

    /// Get province mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsProvince> {
        self.provinces.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.provinces.len()
    }
}

/// Format province registry
pub fn format_province_registry(registry: &ProvinceRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Province Registry:\n");
    output.push_str(&format!("  Provinces: {}\n", registry.count()));
    output
}

/// Check if query is about province
pub fn is_province_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings province") || lower.contains("province settings") || lower.contains("administrative province")
}

/// Fun fact about province
pub fn province_fun_fact() -> &'static str {
    "Anna's settings province establishes administrative governance!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_province_type_display() {
        assert_eq!(format!("{}", ProvinceType::Autonomous), "autonomous");
        assert_eq!(format!("{}", ProvinceType::Imperial), "imperial");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ProvinceStatus::Established), "established");
        assert_eq!(format!("{}", ProvinceStatus::Integrated), "integrated");
    }

    #[test]
    fn test_config_new() {
        let c = ProvinceConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ProvinceConfig::new("test")
            .province_type(ProvinceType::Federal)
            .status(ProvinceStatus::Developing);
        assert_eq!(c.province_type, ProvinceType::Federal);
        assert_eq!(c.status, ProvinceStatus::Developing);
    }

    #[test]
    fn test_edict_new() {
        let e = ProvinceEdict::new("e1", "Title", "Content");
        assert_eq!(e.id, "e1");
    }

    #[test]
    fn test_edict_builder() {
        let e = ProvinceEdict::new("e1", "Title", "Content")
            .prefecture(1);
        assert_eq!(e.prefecture, 1);
    }

    #[test]
    fn test_edict_provincial() {
        let mut e = ProvinceEdict::new("e1", "Title", "Content");
        e.make_local();
        assert!(!e.provincial);
        e.make_provincial();
        assert!(e.provincial);
    }

    #[test]
    fn test_governor_new() {
        let g = ProvinceGovernor::new("key", "name", "e1");
        assert_eq!(g.edict_id, "e1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = ProvinceStats::default();
        let edict = ProvinceEdict::new("e1", "Title", "Content");
        s.update(&[edict], ProvinceType::Autonomous);
        assert_eq!(s.total_edicts, 1);
        assert_eq!(s.provincial, 1);
    }

    #[test]
    fn test_province_new() {
        let p = SettingsProvince::new(ProvinceConfig::default());
        assert_eq!(p.edict_count(), 0);
    }

    #[test]
    fn test_province_add_edict() {
        let mut p = SettingsProvince::new(ProvinceConfig::default());
        p.add_edict(ProvinceEdict::new("e1", "Title", "Content"));
        assert_eq!(p.edict_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = ProvinceRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ProvinceRegistry::new();
        r.register("p1", SettingsProvince::new(ProvinceConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_province_query() {
        assert!(is_province_query("settings province"));
        assert!(!is_province_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = province_fun_fact();
        assert!(fact.contains("province"));
    }
}
