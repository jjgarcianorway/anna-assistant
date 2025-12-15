// v0.0.750: Settings Municipality (Phase 326)
// Municipal corporation for settings self-governance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Municipality type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MunicipalityType {
    /// City municipality
    #[default]
    City,
    /// Town municipality
    Town,
    /// Village municipality
    Village,
    /// Township municipality
    Township,
}

impl std::fmt::Display for MunicipalityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::City => write!(f, "city"),
            Self::Town => write!(f, "town"),
            Self::Village => write!(f, "village"),
            Self::Township => write!(f, "township"),
        }
    }
}

/// Municipality status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MunicipalityStatus {
    /// Incorporated status
    #[default]
    Incorporated,
    /// Chartered status
    Chartered,
    /// Consolidated status
    Consolidated,
    /// Dissolved status
    Dissolved,
}

impl std::fmt::Display for MunicipalityStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Incorporated => write!(f, "incorporated"),
            Self::Chartered => write!(f, "chartered"),
            Self::Consolidated => write!(f, "consolidated"),
            Self::Dissolved => write!(f, "dissolved"),
        }
    }
}

/// Municipality config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MunicipalityConfig {
    /// Name
    pub name: String,
    /// Municipality type
    pub municipality_type: MunicipalityType,
    /// Status
    pub status: MunicipalityStatus,
    /// Max codes
    pub max_codes: usize,
}

impl MunicipalityConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            municipality_type: MunicipalityType::City,
            status: MunicipalityStatus::Incorporated,
            max_codes: 100,
        }
    }

    /// Set type
    pub fn municipality_type(mut self, mt: MunicipalityType) -> Self {
        self.municipality_type = mt;
        self
    }

    /// Set status
    pub fn status(mut self, s: MunicipalityStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max codes
    pub fn max_codes(mut self, max: usize) -> Self {
        self.max_codes = max;
        self
    }
}

impl Default for MunicipalityConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Municipality code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MunicipalityCode {
    /// Code ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Chapter number
    pub chapter: u32,
    /// In force
    pub in_force: bool,
}

impl MunicipalityCode {
    /// Create new code
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            chapter: 0,
            in_force: true,
        }
    }

    /// Set chapter
    pub fn chapter(mut self, c: u32) -> Self {
        self.chapter = c;
        self
    }

    /// Make in force
    pub fn make_in_force(&mut self) {
        self.in_force = true;
    }

    /// Make suspended
    pub fn make_suspended(&mut self) {
        self.in_force = false;
    }
}

/// Municipality councilor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MunicipalityCouncilor {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Code ID
    pub code_id: String,
}

impl MunicipalityCouncilor {
    /// Create new councilor
    pub fn new(key: impl Into<String>, name: impl Into<String>, code_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            code_id: code_id.into(),
        }
    }
}

/// Municipality stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MunicipalityStats {
    /// Total codes
    pub total_codes: usize,
    /// In force codes
    pub in_force: usize,
    /// Chartered count
    pub chartered_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl MunicipalityStats {
    /// Update from codes
    pub fn update(&mut self, codes: &[MunicipalityCode], municipality_type: MunicipalityType) {
        self.total_codes = codes.len();
        self.in_force = codes.iter().filter(|c| c.in_force).count();
        *self.by_type.entry(municipality_type.to_string()).or_insert(0) += 1;
    }

    /// In force rate
    pub fn in_force_rate(&self) -> f64 {
        if self.total_codes == 0 { 0.0 } else { self.in_force as f64 / self.total_codes as f64 * 100.0 }
    }
}

/// Settings municipality
#[derive(Debug, Clone, Default)]
pub struct SettingsMunicipality {
    /// Config
    config: MunicipalityConfig,
    /// Codes
    codes: Vec<MunicipalityCode>,
    /// Councilors
    councilors: Vec<MunicipalityCouncilor>,
    /// Stats
    stats: MunicipalityStats,
}

impl SettingsMunicipality {
    /// Create new municipality system
    pub fn new(config: MunicipalityConfig) -> Self {
        Self {
            config,
            codes: Vec::new(),
            councilors: Vec::new(),
            stats: MunicipalityStats::default(),
        }
    }

    /// Add code
    pub fn add_code(&mut self, code: MunicipalityCode) -> bool {
        if self.codes.len() >= self.config.max_codes {
            return false;
        }
        self.codes.push(code);
        self.update_stats();
        true
    }

    /// Get code
    pub fn get_code(&self, id: &str) -> Option<&MunicipalityCode> {
        self.codes.iter().find(|c| c.id == id)
    }

    /// Get code mut
    pub fn get_code_mut(&mut self, id: &str) -> Option<&mut MunicipalityCode> {
        self.codes.iter_mut().find(|c| c.id == id)
    }

    /// Add councilor
    pub fn add_councilor(&mut self, councilor: MunicipalityCouncilor) {
        self.councilors.push(councilor);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.codes, self.config.municipality_type);
    }

    /// Get stats
    pub fn stats(&self) -> &MunicipalityStats {
        &self.stats
    }

    /// Code count
    pub fn code_count(&self) -> usize {
        self.codes.len()
    }
}

/// Municipality registry
#[derive(Debug, Clone, Default)]
pub struct MunicipalityRegistry {
    /// Municipalities by ID
    municipalities: HashMap<String, SettingsMunicipality>,
}

impl MunicipalityRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register municipality
    pub fn register(&mut self, id: impl Into<String>, municipality: SettingsMunicipality) {
        self.municipalities.insert(id.into(), municipality);
    }

    /// Unregister municipality
    pub fn unregister(&mut self, id: &str) -> bool {
        self.municipalities.remove(id).is_some()
    }

    /// Get municipality
    pub fn get(&self, id: &str) -> Option<&SettingsMunicipality> {
        self.municipalities.get(id)
    }

    /// Get municipality mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsMunicipality> {
        self.municipalities.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.municipalities.len()
    }
}

/// Format municipality registry
pub fn format_municipality_registry(registry: &MunicipalityRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Municipality Registry:\n");
    output.push_str(&format!("  Municipalities: {}\n", registry.count()));
    output
}

/// Check if query is about municipality
pub fn is_municipality_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings municipality") || lower.contains("municipality settings") || lower.contains("municipal corporation")
}

/// Fun fact about municipality
pub fn municipality_fun_fact() -> &'static str {
    "Anna's settings municipality establishes municipal self-governance!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_municipality_type_display() {
        assert_eq!(format!("{}", MunicipalityType::City), "city");
        assert_eq!(format!("{}", MunicipalityType::Town), "town");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", MunicipalityStatus::Incorporated), "incorporated");
        assert_eq!(format!("{}", MunicipalityStatus::Chartered), "chartered");
    }

    #[test]
    fn test_config_new() {
        let c = MunicipalityConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = MunicipalityConfig::new("test")
            .municipality_type(MunicipalityType::Village)
            .status(MunicipalityStatus::Chartered);
        assert_eq!(c.municipality_type, MunicipalityType::Village);
        assert_eq!(c.status, MunicipalityStatus::Chartered);
    }

    #[test]
    fn test_code_new() {
        let c = MunicipalityCode::new("c1", "Title", "Content");
        assert_eq!(c.id, "c1");
    }

    #[test]
    fn test_code_builder() {
        let c = MunicipalityCode::new("c1", "Title", "Content")
            .chapter(1);
        assert_eq!(c.chapter, 1);
    }

    #[test]
    fn test_code_in_force() {
        let mut c = MunicipalityCode::new("c1", "Title", "Content");
        c.make_suspended();
        assert!(!c.in_force);
        c.make_in_force();
        assert!(c.in_force);
    }

    #[test]
    fn test_councilor_new() {
        let c = MunicipalityCouncilor::new("key", "name", "c1");
        assert_eq!(c.code_id, "c1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = MunicipalityStats::default();
        let code = MunicipalityCode::new("c1", "Title", "Content");
        s.update(&[code], MunicipalityType::City);
        assert_eq!(s.total_codes, 1);
        assert_eq!(s.in_force, 1);
    }

    #[test]
    fn test_municipality_new() {
        let m = SettingsMunicipality::new(MunicipalityConfig::default());
        assert_eq!(m.code_count(), 0);
    }

    #[test]
    fn test_municipality_add_code() {
        let mut m = SettingsMunicipality::new(MunicipalityConfig::default());
        m.add_code(MunicipalityCode::new("c1", "Title", "Content"));
        assert_eq!(m.code_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = MunicipalityRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = MunicipalityRegistry::new();
        r.register("m1", SettingsMunicipality::new(MunicipalityConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_municipality_query() {
        assert!(is_municipality_query("settings municipality"));
        assert!(!is_municipality_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = municipality_fun_fact();
        assert!(fact.contains("municipality"));
    }
}
