// v0.0.757: Settings Parcel (Phase 333)
// Land parcel for settings ownership

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Parcel type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ParcelType {
    /// Fee simple parcel
    #[default]
    FeeSimple,
    /// Leasehold parcel
    Leasehold,
    /// Easement parcel
    Easement,
    /// Right-of-way parcel
    RightOfWay,
}

impl std::fmt::Display for ParcelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FeeSimple => write!(f, "fee-simple"),
            Self::Leasehold => write!(f, "leasehold"),
            Self::Easement => write!(f, "easement"),
            Self::RightOfWay => write!(f, "right-of-way"),
        }
    }
}

/// Parcel status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ParcelStatus {
    /// Platted status
    #[default]
    Platted,
    /// Conveyed status
    Conveyed,
    /// Encumbered status
    Encumbered,
    /// Cleared status
    Cleared,
}

impl std::fmt::Display for ParcelStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Platted => write!(f, "platted"),
            Self::Conveyed => write!(f, "conveyed"),
            Self::Encumbered => write!(f, "encumbered"),
            Self::Cleared => write!(f, "cleared"),
        }
    }
}

/// Parcel config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParcelConfig {
    /// Name
    pub name: String,
    /// Parcel type
    pub parcel_type: ParcelType,
    /// Status
    pub status: ParcelStatus,
    /// Max titles
    pub max_titles: usize,
}

impl ParcelConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            parcel_type: ParcelType::FeeSimple,
            status: ParcelStatus::Platted,
            max_titles: 100,
        }
    }

    /// Set type
    pub fn parcel_type(mut self, pt: ParcelType) -> Self {
        self.parcel_type = pt;
        self
    }

    /// Set status
    pub fn status(mut self, s: ParcelStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max titles
    pub fn max_titles(mut self, max: usize) -> Self {
        self.max_titles = max;
        self
    }
}

impl Default for ParcelConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Parcel title
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParcelTitle {
    /// Title ID
    pub id: String,
    /// Title name
    pub title: String,
    /// Content
    pub content: String,
    /// Section number
    pub section: u32,
    /// Cleared
    pub cleared: bool,
}

impl ParcelTitle {
    /// Create new title
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            section: 0,
            cleared: true,
        }
    }

    /// Set section
    pub fn section(mut self, s: u32) -> Self {
        self.section = s;
        self
    }

    /// Make cleared
    pub fn make_cleared(&mut self) {
        self.cleared = true;
    }

    /// Make clouded
    pub fn make_clouded(&mut self) {
        self.cleared = false;
    }
}

/// Parcel examiner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParcelExaminer {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Title ID
    pub title_id: String,
}

impl ParcelExaminer {
    /// Create new examiner
    pub fn new(key: impl Into<String>, name: impl Into<String>, title_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            title_id: title_id.into(),
        }
    }
}

/// Parcel stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParcelStats {
    /// Total titles
    pub total_titles: usize,
    /// Cleared titles
    pub cleared: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ParcelStats {
    /// Update from titles
    pub fn update(&mut self, titles: &[ParcelTitle], parcel_type: ParcelType) {
        self.total_titles = titles.len();
        self.cleared = titles.iter().filter(|t| t.cleared).count();
        *self.by_type.entry(parcel_type.to_string()).or_insert(0) += 1;
    }

    /// Cleared rate
    pub fn cleared_rate(&self) -> f64 {
        if self.total_titles == 0 { 0.0 } else { self.cleared as f64 / self.total_titles as f64 * 100.0 }
    }
}

/// Settings parcel
#[derive(Debug, Clone, Default)]
pub struct SettingsParcel {
    /// Config
    config: ParcelConfig,
    /// Titles
    titles: Vec<ParcelTitle>,
    /// Examiners
    examiners: Vec<ParcelExaminer>,
    /// Stats
    stats: ParcelStats,
}

impl SettingsParcel {
    /// Create new parcel system
    pub fn new(config: ParcelConfig) -> Self {
        Self {
            config,
            titles: Vec::new(),
            examiners: Vec::new(),
            stats: ParcelStats::default(),
        }
    }

    /// Add title
    pub fn add_title(&mut self, title: ParcelTitle) -> bool {
        if self.titles.len() >= self.config.max_titles {
            return false;
        }
        self.titles.push(title);
        self.update_stats();
        true
    }

    /// Get title
    pub fn get_title(&self, id: &str) -> Option<&ParcelTitle> {
        self.titles.iter().find(|t| t.id == id)
    }

    /// Get title mut
    pub fn get_title_mut(&mut self, id: &str) -> Option<&mut ParcelTitle> {
        self.titles.iter_mut().find(|t| t.id == id)
    }

    /// Add examiner
    pub fn add_examiner(&mut self, examiner: ParcelExaminer) {
        self.examiners.push(examiner);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.titles, self.config.parcel_type);
    }

    /// Get stats
    pub fn stats(&self) -> &ParcelStats {
        &self.stats
    }

    /// Title count
    pub fn title_count(&self) -> usize {
        self.titles.len()
    }
}

/// Parcel registry
#[derive(Debug, Clone, Default)]
pub struct ParcelRegistry {
    /// Parcels by ID
    parcels: HashMap<String, SettingsParcel>,
}

impl ParcelRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register parcel
    pub fn register(&mut self, id: impl Into<String>, parcel: SettingsParcel) {
        self.parcels.insert(id.into(), parcel);
    }

    /// Unregister parcel
    pub fn unregister(&mut self, id: &str) -> bool {
        self.parcels.remove(id).is_some()
    }

    /// Get parcel
    pub fn get(&self, id: &str) -> Option<&SettingsParcel> {
        self.parcels.get(id)
    }

    /// Get parcel mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsParcel> {
        self.parcels.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.parcels.len()
    }
}

/// Format parcel registry
pub fn format_parcel_registry(registry: &ParcelRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Parcel Registry:\n");
    output.push_str(&format!("  Parcels: {}\n", registry.count()));
    output
}

/// Check if query is about parcel
pub fn is_parcel_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings parcel") || lower.contains("parcel settings") || lower.contains("land parcel")
}

/// Fun fact about parcel
pub fn parcel_fun_fact() -> &'static str {
    "Anna's settings parcel establishes ownership boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parcel_type_display() {
        assert_eq!(format!("{}", ParcelType::FeeSimple), "fee-simple");
        assert_eq!(format!("{}", ParcelType::Leasehold), "leasehold");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ParcelStatus::Platted), "platted");
        assert_eq!(format!("{}", ParcelStatus::Conveyed), "conveyed");
    }

    #[test]
    fn test_config_new() {
        let c = ParcelConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ParcelConfig::new("test")
            .parcel_type(ParcelType::Easement)
            .status(ParcelStatus::Encumbered);
        assert_eq!(c.parcel_type, ParcelType::Easement);
        assert_eq!(c.status, ParcelStatus::Encumbered);
    }

    #[test]
    fn test_title_new() {
        let t = ParcelTitle::new("t1", "Title", "Content");
        assert_eq!(t.id, "t1");
    }

    #[test]
    fn test_title_builder() {
        let t = ParcelTitle::new("t1", "Title", "Content")
            .section(1);
        assert_eq!(t.section, 1);
    }

    #[test]
    fn test_title_cleared() {
        let mut t = ParcelTitle::new("t1", "Title", "Content");
        t.make_clouded();
        assert!(!t.cleared);
        t.make_cleared();
        assert!(t.cleared);
    }

    #[test]
    fn test_examiner_new() {
        let e = ParcelExaminer::new("key", "name", "t1");
        assert_eq!(e.title_id, "t1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = ParcelStats::default();
        let title = ParcelTitle::new("t1", "Title", "Content");
        s.update(&[title], ParcelType::FeeSimple);
        assert_eq!(s.total_titles, 1);
        assert_eq!(s.cleared, 1);
    }

    #[test]
    fn test_parcel_new() {
        let p = SettingsParcel::new(ParcelConfig::default());
        assert_eq!(p.title_count(), 0);
    }

    #[test]
    fn test_parcel_add_title() {
        let mut p = SettingsParcel::new(ParcelConfig::default());
        p.add_title(ParcelTitle::new("t1", "Title", "Content"));
        assert_eq!(p.title_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = ParcelRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ParcelRegistry::new();
        r.register("p1", SettingsParcel::new(ParcelConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_parcel_query() {
        assert!(is_parcel_query("settings parcel"));
        assert!(!is_parcel_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = parcel_fun_fact();
        assert!(fact.contains("parcel"));
    }
}
