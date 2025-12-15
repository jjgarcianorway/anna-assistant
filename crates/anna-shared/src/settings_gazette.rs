// v0.0.704: Settings Gazette (Phase 280)
// Official gazette of settings announcements

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Gazette type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GazetteType {
    /// Official gazette
    #[default]
    Official,
    /// Weekly gazette
    Weekly,
    /// Special gazette
    Special,
    /// Extraordinary gazette
    Extraordinary,
}

impl std::fmt::Display for GazetteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Official => write!(f, "official"),
            Self::Weekly => write!(f, "weekly"),
            Self::Special => write!(f, "special"),
            Self::Extraordinary => write!(f, "extraordinary"),
        }
    }
}

/// Gazette status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GazetteStatus {
    /// Draft
    #[default]
    Draft,
    /// Review
    Review,
    /// Published
    Published,
    /// Superseded
    Superseded,
}

impl std::fmt::Display for GazetteStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Review => write!(f, "review"),
            Self::Published => write!(f, "published"),
            Self::Superseded => write!(f, "superseded"),
        }
    }
}

/// Gazette config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GazetteConfig {
    /// Name
    pub name: String,
    /// Gazette type
    pub gazette_type: GazetteType,
    /// Issue number
    pub issue_number: usize,
    /// Max notices
    pub max_notices: usize,
}

impl GazetteConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            gazette_type: GazetteType::Official,
            issue_number: 1,
            max_notices: 100,
        }
    }

    /// Set type
    pub fn gazette_type(mut self, gt: GazetteType) -> Self {
        self.gazette_type = gt;
        self
    }

    /// Set issue number
    pub fn issue_number(mut self, num: usize) -> Self {
        self.issue_number = num;
        self
    }

    /// Set max notices
    pub fn max_notices(mut self, max: usize) -> Self {
        self.max_notices = max;
        self
    }
}

impl Default for GazetteConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Gazette notice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GazetteNotice {
    /// Notice ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Effective date
    pub effective_date: String,
    /// Urgent
    pub urgent: bool,
}

impl GazetteNotice {
    /// Create new notice
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            effective_date: String::new(),
            urgent: false,
        }
    }

    /// Set effective date
    pub fn effective_date(mut self, date: impl Into<String>) -> Self {
        self.effective_date = date.into();
        self
    }

    /// Set urgent
    pub fn urgent(mut self, u: bool) -> Self {
        self.urgent = u;
        self
    }
}

/// Gazette entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GazetteEntry {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Notice ID
    pub notice_id: String,
    /// Reference
    pub reference: Option<String>,
}

impl GazetteEntry {
    /// Create new entry
    pub fn new(key: impl Into<String>, value: impl Into<String>, notice_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            notice_id: notice_id.into(),
            reference: None,
        }
    }

    /// Set reference
    pub fn reference(mut self, ref_: impl Into<String>) -> Self {
        self.reference = Some(ref_.into());
        self
    }
}

/// Gazette stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GazetteStats {
    /// Total notices
    pub total_notices: usize,
    /// Urgent notices
    pub urgent_notices: usize,
    /// Total entries
    pub total_entries: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl GazetteStats {
    /// Update from gazette
    pub fn update(&mut self, notices: &[GazetteNotice], gazette_type: GazetteType) {
        self.total_notices = notices.len();
        self.urgent_notices = notices.iter().filter(|n| n.urgent).count();
        *self.by_type.entry(gazette_type.to_string()).or_insert(0) += 1;
    }

    /// Record entry
    pub fn record_entry(&mut self) {
        self.total_entries += 1;
    }

    /// Urgent rate
    pub fn urgent_rate(&self) -> f64 {
        if self.total_notices == 0 { 0.0 } else { self.urgent_notices as f64 / self.total_notices as f64 * 100.0 }
    }
}

/// Settings gazette
#[derive(Debug, Clone, Default)]
pub struct SettingsGazette {
    /// Config
    config: GazetteConfig,
    /// Notices
    notices: Vec<GazetteNotice>,
    /// Entries
    entries: Vec<GazetteEntry>,
    /// Status
    status: GazetteStatus,
    /// Stats
    stats: GazetteStats,
}

impl SettingsGazette {
    /// Create new gazette
    pub fn new(config: GazetteConfig) -> Self {
        Self {
            config,
            notices: Vec::new(),
            entries: Vec::new(),
            status: GazetteStatus::Draft,
            stats: GazetteStats::default(),
        }
    }

    /// Add notice
    pub fn add_notice(&mut self, notice: GazetteNotice) -> bool {
        if self.notices.len() >= self.config.max_notices {
            return false;
        }
        self.notices.push(notice);
        self.update_stats();
        true
    }

    /// Get notice
    pub fn get_notice(&self, id: &str) -> Option<&GazetteNotice> {
        self.notices.iter().find(|n| n.id == id)
    }

    /// Add entry
    pub fn add_entry(&mut self, entry: GazetteEntry) {
        self.entries.push(entry);
        self.stats.record_entry();
    }

    /// Get entries for notice
    pub fn get_entries(&self, notice_id: &str) -> Vec<&GazetteEntry> {
        self.entries.iter().filter(|e| e.notice_id == notice_id).collect()
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.notices, self.config.gazette_type);
    }

    /// Submit for review
    pub fn review(&mut self) {
        self.status = GazetteStatus::Review;
    }

    /// Publish
    pub fn publish(&mut self) {
        self.status = GazetteStatus::Published;
    }

    /// Supersede
    pub fn supersede(&mut self) {
        self.status = GazetteStatus::Superseded;
    }

    /// Get status
    pub fn status(&self) -> GazetteStatus {
        self.status
    }

    /// Get stats
    pub fn stats(&self) -> &GazetteStats {
        &self.stats
    }

    /// Notice count
    pub fn notice_count(&self) -> usize {
        self.notices.len()
    }
}

/// Gazette registry
#[derive(Debug, Clone, Default)]
pub struct GazetteRegistry {
    /// Gazettes by ID
    gazettes: HashMap<String, SettingsGazette>,
}

impl GazetteRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register gazette
    pub fn register(&mut self, id: impl Into<String>, gazette: SettingsGazette) {
        self.gazettes.insert(id.into(), gazette);
    }

    /// Unregister gazette
    pub fn unregister(&mut self, id: &str) -> bool {
        self.gazettes.remove(id).is_some()
    }

    /// Get gazette
    pub fn get(&self, id: &str) -> Option<&SettingsGazette> {
        self.gazettes.get(id)
    }

    /// Get gazette mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsGazette> {
        self.gazettes.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.gazettes.len()
    }
}

/// Format gazette registry
pub fn format_gazette_registry(registry: &GazetteRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Gazette Registry:\n");
    output.push_str(&format!("  Gazettes: {}\n", registry.count()));
    output
}

/// Check if query is about gazette
pub fn is_gazette_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings gazette") || lower.contains("gazette settings") || lower.contains("official notice")
}

/// Fun fact about gazette
pub fn gazette_fun_fact() -> &'static str {
    "Anna's settings gazette publishes official announcements about your configurations!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gazette_type_display() {
        assert_eq!(format!("{}", GazetteType::Official), "official");
        assert_eq!(format!("{}", GazetteType::Special), "special");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", GazetteStatus::Draft), "draft");
        assert_eq!(format!("{}", GazetteStatus::Published), "published");
    }

    #[test]
    fn test_config_new() {
        let c = GazetteConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = GazetteConfig::new("test")
            .gazette_type(GazetteType::Special)
            .issue_number(5);
        assert_eq!(c.gazette_type, GazetteType::Special);
        assert_eq!(c.issue_number, 5);
    }

    #[test]
    fn test_notice_new() {
        let n = GazetteNotice::new("n1", "Notice 1", "Content");
        assert_eq!(n.id, "n1");
    }

    #[test]
    fn test_notice_builder() {
        let n = GazetteNotice::new("n1", "Notice 1", "Content")
            .effective_date("2025-12-15")
            .urgent(true);
        assert_eq!(n.effective_date, "2025-12-15");
        assert!(n.urgent);
    }

    #[test]
    fn test_entry_new() {
        let e = GazetteEntry::new("key", "value", "n1");
        assert_eq!(e.notice_id, "n1");
    }

    #[test]
    fn test_entry_reference() {
        let e = GazetteEntry::new("key", "value", "n1").reference("REF-001");
        assert!(e.reference.is_some());
    }

    #[test]
    fn test_stats_update() {
        let mut s = GazetteStats::default();
        let notices = vec![GazetteNotice::new("n1", "Notice", "Content").urgent(true)];
        s.update(&notices, GazetteType::Official);
        assert_eq!(s.total_notices, 1);
        assert_eq!(s.urgent_notices, 1);
    }

    #[test]
    fn test_gazette_new() {
        let g = SettingsGazette::new(GazetteConfig::default());
        assert_eq!(g.notice_count(), 0);
    }

    #[test]
    fn test_gazette_add_notice() {
        let mut g = SettingsGazette::new(GazetteConfig::default());
        g.add_notice(GazetteNotice::new("n1", "Notice 1", "Content"));
        assert_eq!(g.notice_count(), 1);
    }

    #[test]
    fn test_gazette_review() {
        let mut g = SettingsGazette::new(GazetteConfig::default());
        g.review();
        assert_eq!(g.status(), GazetteStatus::Review);
    }

    #[test]
    fn test_gazette_publish() {
        let mut g = SettingsGazette::new(GazetteConfig::default());
        g.publish();
        assert_eq!(g.status(), GazetteStatus::Published);
    }

    #[test]
    fn test_registry_new() {
        let r = GazetteRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = GazetteRegistry::new();
        r.register("g1", SettingsGazette::new(GazetteConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_gazette_query() {
        assert!(is_gazette_query("settings gazette"));
        assert!(!is_gazette_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = gazette_fun_fact();
        assert!(fact.contains("gazette"));
    }
}
