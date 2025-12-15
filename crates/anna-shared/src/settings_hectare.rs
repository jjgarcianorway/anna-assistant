// v0.0.761: Settings Hectare (Phase 337)
// Land hectare for settings metric area

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Hectare type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HectareType {
    /// Standard hectare
    #[default]
    Standard,
    /// Cadastral hectare
    Cadastral,
    /// Agricultural hectare
    Agricultural,
    /// Forest hectare
    Forest,
}

impl std::fmt::Display for HectareType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Standard => write!(f, "standard"),
            Self::Cadastral => write!(f, "cadastral"),
            Self::Agricultural => write!(f, "agricultural"),
            Self::Forest => write!(f, "forest"),
        }
    }
}

/// Hectare status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HectareStatus {
    /// Surveyed status
    #[default]
    Surveyed,
    /// Registered status
    Registered,
    /// Contested status
    Contested,
    /// Confirmed status
    Confirmed,
}

impl std::fmt::Display for HectareStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Surveyed => write!(f, "surveyed"),
            Self::Registered => write!(f, "registered"),
            Self::Contested => write!(f, "contested"),
            Self::Confirmed => write!(f, "confirmed"),
        }
    }
}

/// Hectare config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HectareConfig {
    /// Name
    pub name: String,
    /// Hectare type
    pub hectare_type: HectareType,
    /// Status
    pub status: HectareStatus,
    /// Max records
    pub max_records: usize,
}

impl HectareConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            hectare_type: HectareType::Standard,
            status: HectareStatus::Surveyed,
            max_records: 100,
        }
    }

    /// Set type
    pub fn hectare_type(mut self, ht: HectareType) -> Self {
        self.hectare_type = ht;
        self
    }

    /// Set status
    pub fn status(mut self, s: HectareStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max records
    pub fn max_records(mut self, max: usize) -> Self {
        self.max_records = max;
        self
    }
}

impl Default for HectareConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Hectare record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HectareRecord {
    /// Record ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Grid reference
    pub grid: u32,
    /// Confirmed
    pub confirmed: bool,
}

impl HectareRecord {
    /// Create new record
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            grid: 0,
            confirmed: true,
        }
    }

    /// Set grid
    pub fn grid(mut self, g: u32) -> Self {
        self.grid = g;
        self
    }

    /// Make confirmed
    pub fn make_confirmed(&mut self) {
        self.confirmed = true;
    }

    /// Make unconfirmed
    pub fn make_unconfirmed(&mut self) {
        self.confirmed = false;
    }
}

/// Hectare inspector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HectareInspector {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Record ID
    pub record_id: String,
}

impl HectareInspector {
    /// Create new inspector
    pub fn new(key: impl Into<String>, name: impl Into<String>, record_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            record_id: record_id.into(),
        }
    }
}

/// Hectare stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HectareStats {
    /// Total records
    pub total_records: usize,
    /// Confirmed records
    pub confirmed: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl HectareStats {
    /// Update from records
    pub fn update(&mut self, records: &[HectareRecord], hectare_type: HectareType) {
        self.total_records = records.len();
        self.confirmed = records.iter().filter(|r| r.confirmed).count();
        *self.by_type.entry(hectare_type.to_string()).or_insert(0) += 1;
    }

    /// Confirmed rate
    pub fn confirmed_rate(&self) -> f64 {
        if self.total_records == 0 { 0.0 } else { self.confirmed as f64 / self.total_records as f64 * 100.0 }
    }
}

/// Settings hectare
#[derive(Debug, Clone, Default)]
pub struct SettingsHectare {
    /// Config
    config: HectareConfig,
    /// Records
    records: Vec<HectareRecord>,
    /// Inspectors
    inspectors: Vec<HectareInspector>,
    /// Stats
    stats: HectareStats,
}

impl SettingsHectare {
    /// Create new hectare system
    pub fn new(config: HectareConfig) -> Self {
        Self {
            config,
            records: Vec::new(),
            inspectors: Vec::new(),
            stats: HectareStats::default(),
        }
    }

    /// Add record
    pub fn add_record(&mut self, record: HectareRecord) -> bool {
        if self.records.len() >= self.config.max_records {
            return false;
        }
        self.records.push(record);
        self.update_stats();
        true
    }

    /// Get record
    pub fn get_record(&self, id: &str) -> Option<&HectareRecord> {
        self.records.iter().find(|r| r.id == id)
    }

    /// Get record mut
    pub fn get_record_mut(&mut self, id: &str) -> Option<&mut HectareRecord> {
        self.records.iter_mut().find(|r| r.id == id)
    }

    /// Add inspector
    pub fn add_inspector(&mut self, inspector: HectareInspector) {
        self.inspectors.push(inspector);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.records, self.config.hectare_type);
    }

    /// Get stats
    pub fn stats(&self) -> &HectareStats {
        &self.stats
    }

    /// Record count
    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

/// Hectare registry
#[derive(Debug, Clone, Default)]
pub struct HectareRegistry {
    /// Hectares by ID
    hectares: HashMap<String, SettingsHectare>,
}

impl HectareRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register hectare
    pub fn register(&mut self, id: impl Into<String>, hectare: SettingsHectare) {
        self.hectares.insert(id.into(), hectare);
    }

    /// Unregister hectare
    pub fn unregister(&mut self, id: &str) -> bool {
        self.hectares.remove(id).is_some()
    }

    /// Get hectare
    pub fn get(&self, id: &str) -> Option<&SettingsHectare> {
        self.hectares.get(id)
    }

    /// Get hectare mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsHectare> {
        self.hectares.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.hectares.len()
    }
}

/// Format hectare registry
pub fn format_hectare_registry(registry: &HectareRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Hectare Registry:\n");
    output.push_str(&format!("  Hectares: {}\n", registry.count()));
    output
}

/// Check if query is about hectare
pub fn is_hectare_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings hectare") || lower.contains("hectare settings") || lower.contains("metric area")
}

/// Fun fact about hectare
pub fn hectare_fun_fact() -> &'static str {
    "Anna's settings hectare establishes metric area standards!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hectare_type_display() {
        assert_eq!(format!("{}", HectareType::Standard), "standard");
        assert_eq!(format!("{}", HectareType::Cadastral), "cadastral");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", HectareStatus::Surveyed), "surveyed");
        assert_eq!(format!("{}", HectareStatus::Confirmed), "confirmed");
    }

    #[test]
    fn test_config_new() {
        let c = HectareConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = HectareConfig::new("test")
            .hectare_type(HectareType::Forest)
            .status(HectareStatus::Contested);
        assert_eq!(c.hectare_type, HectareType::Forest);
        assert_eq!(c.status, HectareStatus::Contested);
    }

    #[test]
    fn test_record_new() {
        let r = HectareRecord::new("r1", "Title", "Content");
        assert_eq!(r.id, "r1");
    }

    #[test]
    fn test_record_builder() {
        let r = HectareRecord::new("r1", "Title", "Content")
            .grid(1);
        assert_eq!(r.grid, 1);
    }

    #[test]
    fn test_record_confirmed() {
        let mut r = HectareRecord::new("r1", "Title", "Content");
        r.make_unconfirmed();
        assert!(!r.confirmed);
        r.make_confirmed();
        assert!(r.confirmed);
    }

    #[test]
    fn test_inspector_new() {
        let i = HectareInspector::new("key", "name", "r1");
        assert_eq!(i.record_id, "r1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = HectareStats::default();
        let record = HectareRecord::new("r1", "Title", "Content");
        s.update(&[record], HectareType::Standard);
        assert_eq!(s.total_records, 1);
        assert_eq!(s.confirmed, 1);
    }

    #[test]
    fn test_hectare_new() {
        let h = SettingsHectare::new(HectareConfig::default());
        assert_eq!(h.record_count(), 0);
    }

    #[test]
    fn test_hectare_add_record() {
        let mut h = SettingsHectare::new(HectareConfig::default());
        h.add_record(HectareRecord::new("r1", "Title", "Content"));
        assert_eq!(h.record_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = HectareRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = HectareRegistry::new();
        r.register("h1", SettingsHectare::new(HectareConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_hectare_query() {
        assert!(is_hectare_query("settings hectare"));
        assert!(!is_hectare_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = hectare_fun_fact();
        assert!(fact.contains("hectare"));
    }
}
