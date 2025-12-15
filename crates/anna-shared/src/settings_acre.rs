// v0.0.760: Settings Acre (Phase 336)
// Land acre for settings measurement

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Acre type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AcreType {
    /// Survey acre
    #[default]
    Survey,
    /// Statute acre
    Statute,
    /// Irish acre
    Irish,
    /// Scottish acre
    Scottish,
}

impl std::fmt::Display for AcreType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Survey => write!(f, "survey"),
            Self::Statute => write!(f, "statute"),
            Self::Irish => write!(f, "irish"),
            Self::Scottish => write!(f, "scottish"),
        }
    }
}

/// Acre status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AcreStatus {
    /// Measured status
    #[default]
    Measured,
    /// Verified status
    Verified,
    /// Disputed status
    Disputed,
    /// Certified status
    Certified,
}

impl std::fmt::Display for AcreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Measured => write!(f, "measured"),
            Self::Verified => write!(f, "verified"),
            Self::Disputed => write!(f, "disputed"),
            Self::Certified => write!(f, "certified"),
        }
    }
}

/// Acre config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcreConfig {
    /// Name
    pub name: String,
    /// Acre type
    pub acre_type: AcreType,
    /// Status
    pub status: AcreStatus,
    /// Max measurements
    pub max_measurements: usize,
}

impl AcreConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            acre_type: AcreType::Survey,
            status: AcreStatus::Measured,
            max_measurements: 100,
        }
    }

    /// Set type
    pub fn acre_type(mut self, at: AcreType) -> Self {
        self.acre_type = at;
        self
    }

    /// Set status
    pub fn status(mut self, s: AcreStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max measurements
    pub fn max_measurements(mut self, max: usize) -> Self {
        self.max_measurements = max;
        self
    }
}

impl Default for AcreConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Acre measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcreMeasurement {
    /// Measurement ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Chain number
    pub chain: u32,
    /// Certified
    pub certified: bool,
}

impl AcreMeasurement {
    /// Create new measurement
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            chain: 0,
            certified: true,
        }
    }

    /// Set chain
    pub fn chain(mut self, c: u32) -> Self {
        self.chain = c;
        self
    }

    /// Make certified
    pub fn make_certified(&mut self) {
        self.certified = true;
    }

    /// Make uncertified
    pub fn make_uncertified(&mut self) {
        self.certified = false;
    }
}

/// Acre surveyor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcreSurveyor {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Measurement ID
    pub measurement_id: String,
}

impl AcreSurveyor {
    /// Create new surveyor
    pub fn new(key: impl Into<String>, name: impl Into<String>, measurement_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            measurement_id: measurement_id.into(),
        }
    }
}

/// Acre stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AcreStats {
    /// Total measurements
    pub total_measurements: usize,
    /// Certified measurements
    pub certified: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl AcreStats {
    /// Update from measurements
    pub fn update(&mut self, measurements: &[AcreMeasurement], acre_type: AcreType) {
        self.total_measurements = measurements.len();
        self.certified = measurements.iter().filter(|m| m.certified).count();
        *self.by_type.entry(acre_type.to_string()).or_insert(0) += 1;
    }

    /// Certified rate
    pub fn certified_rate(&self) -> f64 {
        if self.total_measurements == 0 { 0.0 } else { self.certified as f64 / self.total_measurements as f64 * 100.0 }
    }
}

/// Settings acre
#[derive(Debug, Clone, Default)]
pub struct SettingsAcre {
    /// Config
    config: AcreConfig,
    /// Measurements
    measurements: Vec<AcreMeasurement>,
    /// Surveyors
    surveyors: Vec<AcreSurveyor>,
    /// Stats
    stats: AcreStats,
}

impl SettingsAcre {
    /// Create new acre system
    pub fn new(config: AcreConfig) -> Self {
        Self {
            config,
            measurements: Vec::new(),
            surveyors: Vec::new(),
            stats: AcreStats::default(),
        }
    }

    /// Add measurement
    pub fn add_measurement(&mut self, measurement: AcreMeasurement) -> bool {
        if self.measurements.len() >= self.config.max_measurements {
            return false;
        }
        self.measurements.push(measurement);
        self.update_stats();
        true
    }

    /// Get measurement
    pub fn get_measurement(&self, id: &str) -> Option<&AcreMeasurement> {
        self.measurements.iter().find(|m| m.id == id)
    }

    /// Get measurement mut
    pub fn get_measurement_mut(&mut self, id: &str) -> Option<&mut AcreMeasurement> {
        self.measurements.iter_mut().find(|m| m.id == id)
    }

    /// Add surveyor
    pub fn add_surveyor(&mut self, surveyor: AcreSurveyor) {
        self.surveyors.push(surveyor);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.measurements, self.config.acre_type);
    }

    /// Get stats
    pub fn stats(&self) -> &AcreStats {
        &self.stats
    }

    /// Measurement count
    pub fn measurement_count(&self) -> usize {
        self.measurements.len()
    }
}

/// Acre registry
#[derive(Debug, Clone, Default)]
pub struct AcreRegistry {
    /// Acres by ID
    acres: HashMap<String, SettingsAcre>,
}

impl AcreRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register acre
    pub fn register(&mut self, id: impl Into<String>, acre: SettingsAcre) {
        self.acres.insert(id.into(), acre);
    }

    /// Unregister acre
    pub fn unregister(&mut self, id: &str) -> bool {
        self.acres.remove(id).is_some()
    }

    /// Get acre
    pub fn get(&self, id: &str) -> Option<&SettingsAcre> {
        self.acres.get(id)
    }

    /// Get acre mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsAcre> {
        self.acres.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.acres.len()
    }
}

/// Format acre registry
pub fn format_acre_registry(registry: &AcreRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Acre Registry:\n");
    output.push_str(&format!("  Acres: {}\n", registry.count()));
    output
}

/// Check if query is about acre
pub fn is_acre_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings acre") || lower.contains("acre settings") || lower.contains("land acre")
}

/// Fun fact about acre
pub fn acre_fun_fact() -> &'static str {
    "Anna's settings acre establishes measurement standards!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acre_type_display() {
        assert_eq!(format!("{}", AcreType::Survey), "survey");
        assert_eq!(format!("{}", AcreType::Statute), "statute");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", AcreStatus::Measured), "measured");
        assert_eq!(format!("{}", AcreStatus::Certified), "certified");
    }

    #[test]
    fn test_config_new() {
        let c = AcreConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = AcreConfig::new("test")
            .acre_type(AcreType::Irish)
            .status(AcreStatus::Disputed);
        assert_eq!(c.acre_type, AcreType::Irish);
        assert_eq!(c.status, AcreStatus::Disputed);
    }

    #[test]
    fn test_measurement_new() {
        let m = AcreMeasurement::new("m1", "Title", "Content");
        assert_eq!(m.id, "m1");
    }

    #[test]
    fn test_measurement_builder() {
        let m = AcreMeasurement::new("m1", "Title", "Content")
            .chain(1);
        assert_eq!(m.chain, 1);
    }

    #[test]
    fn test_measurement_certified() {
        let mut m = AcreMeasurement::new("m1", "Title", "Content");
        m.make_uncertified();
        assert!(!m.certified);
        m.make_certified();
        assert!(m.certified);
    }

    #[test]
    fn test_surveyor_new() {
        let s = AcreSurveyor::new("key", "name", "m1");
        assert_eq!(s.measurement_id, "m1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = AcreStats::default();
        let measurement = AcreMeasurement::new("m1", "Title", "Content");
        s.update(&[measurement], AcreType::Survey);
        assert_eq!(s.total_measurements, 1);
        assert_eq!(s.certified, 1);
    }

    #[test]
    fn test_acre_new() {
        let a = SettingsAcre::new(AcreConfig::default());
        assert_eq!(a.measurement_count(), 0);
    }

    #[test]
    fn test_acre_add_measurement() {
        let mut a = SettingsAcre::new(AcreConfig::default());
        a.add_measurement(AcreMeasurement::new("m1", "Title", "Content"));
        assert_eq!(a.measurement_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = AcreRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = AcreRegistry::new();
        r.register("a1", SettingsAcre::new(AcreConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_acre_query() {
        assert!(is_acre_query("settings acre"));
        assert!(!is_acre_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = acre_fun_fact();
        assert!(fact.contains("acre"));
    }
}
