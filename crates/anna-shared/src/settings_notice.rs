// v0.0.713: Settings Notice (Phase 289)
// Official notices about settings changes

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Notice type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NoticeType {
    /// Information notice
    #[default]
    Information,
    /// Warning notice
    Warning,
    /// Alert notice
    Alert,
    /// Announcement notice
    Announcement,
}

impl std::fmt::Display for NoticeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Information => write!(f, "information"),
            Self::Warning => write!(f, "warning"),
            Self::Alert => write!(f, "alert"),
            Self::Announcement => write!(f, "announcement"),
        }
    }
}

/// Notice priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NoticePriority {
    /// Low priority
    #[default]
    Low,
    /// Normal priority
    Normal,
    /// High priority
    High,
    /// Urgent priority
    Urgent,
}

impl std::fmt::Display for NoticePriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Normal => write!(f, "normal"),
            Self::High => write!(f, "high"),
            Self::Urgent => write!(f, "urgent"),
        }
    }
}

/// Notice config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoticeConfig {
    /// Name
    pub name: String,
    /// Notice type
    pub notice_type: NoticeType,
    /// Default priority
    pub default_priority: NoticePriority,
    /// Max notices
    pub max_notices: usize,
}

impl NoticeConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            notice_type: NoticeType::Information,
            default_priority: NoticePriority::Normal,
            max_notices: 200,
        }
    }

    /// Set type
    pub fn notice_type(mut self, nt: NoticeType) -> Self {
        self.notice_type = nt;
        self
    }

    /// Set default priority
    pub fn default_priority(mut self, dp: NoticePriority) -> Self {
        self.default_priority = dp;
        self
    }

    /// Set max notices
    pub fn max_notices(mut self, max: usize) -> Self {
        self.max_notices = max;
        self
    }
}

impl Default for NoticeConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Notice entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoticeEntry {
    /// Entry ID
    pub id: String,
    /// Title
    pub title: String,
    /// Message
    pub message: String,
    /// Priority
    pub priority: NoticePriority,
    /// Acknowledged
    pub acknowledged: bool,
}

impl NoticeEntry {
    /// Create new entry
    pub fn new(id: impl Into<String>, title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            message: message.into(),
            priority: NoticePriority::Normal,
            acknowledged: false,
        }
    }

    /// Set priority
    pub fn priority(mut self, p: NoticePriority) -> Self {
        self.priority = p;
        self
    }

    /// Acknowledge notice
    pub fn acknowledge(&mut self) {
        self.acknowledged = true;
    }
}

/// Notice metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoticeMetadata {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Entry ID
    pub entry_id: String,
}

impl NoticeMetadata {
    /// Create new metadata
    pub fn new(key: impl Into<String>, value: impl Into<String>, entry_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            entry_id: entry_id.into(),
        }
    }
}

/// Notice stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NoticeStats {
    /// Total notices
    pub total_notices: usize,
    /// Acknowledged notices
    pub acknowledged: usize,
    /// Urgent notices
    pub urgent: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl NoticeStats {
    /// Update from entries
    pub fn update(&mut self, entries: &[NoticeEntry], notice_type: NoticeType) {
        self.total_notices = entries.len();
        self.acknowledged = entries.iter().filter(|e| e.acknowledged).count();
        self.urgent = entries.iter().filter(|e| e.priority == NoticePriority::Urgent).count();
        *self.by_type.entry(notice_type.to_string()).or_insert(0) += 1;
    }

    /// Acknowledgment rate
    pub fn ack_rate(&self) -> f64 {
        if self.total_notices == 0 { 0.0 } else { self.acknowledged as f64 / self.total_notices as f64 * 100.0 }
    }
}

/// Settings notice
#[derive(Debug, Clone, Default)]
pub struct SettingsNotice {
    /// Config
    config: NoticeConfig,
    /// Entries
    entries: Vec<NoticeEntry>,
    /// Metadata
    metadata: Vec<NoticeMetadata>,
    /// Stats
    stats: NoticeStats,
}

impl SettingsNotice {
    /// Create new notice system
    pub fn new(config: NoticeConfig) -> Self {
        Self {
            config,
            entries: Vec::new(),
            metadata: Vec::new(),
            stats: NoticeStats::default(),
        }
    }

    /// Add entry
    pub fn add_entry(&mut self, entry: NoticeEntry) -> bool {
        if self.entries.len() >= self.config.max_notices {
            return false;
        }
        self.entries.push(entry);
        self.update_stats();
        true
    }

    /// Get entry
    pub fn get_entry(&self, id: &str) -> Option<&NoticeEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Get entry mut
    pub fn get_entry_mut(&mut self, id: &str) -> Option<&mut NoticeEntry> {
        self.entries.iter_mut().find(|e| e.id == id)
    }

    /// Add metadata
    pub fn add_metadata(&mut self, meta: NoticeMetadata) {
        self.metadata.push(meta);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.entries, self.config.notice_type);
    }

    /// Get stats
    pub fn stats(&self) -> &NoticeStats {
        &self.stats
    }

    /// Entry count
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

/// Notice registry
#[derive(Debug, Clone, Default)]
pub struct NoticeRegistry {
    /// Notices by ID
    notices: HashMap<String, SettingsNotice>,
}

impl NoticeRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register notice
    pub fn register(&mut self, id: impl Into<String>, notice: SettingsNotice) {
        self.notices.insert(id.into(), notice);
    }

    /// Unregister notice
    pub fn unregister(&mut self, id: &str) -> bool {
        self.notices.remove(id).is_some()
    }

    /// Get notice
    pub fn get(&self, id: &str) -> Option<&SettingsNotice> {
        self.notices.get(id)
    }

    /// Get notice mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsNotice> {
        self.notices.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.notices.len()
    }
}

/// Format notice registry
pub fn format_notice_registry(registry: &NoticeRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Notice Registry:\n");
    output.push_str(&format!("  Notices: {}\n", registry.count()));
    output
}

/// Check if query is about notice
pub fn is_notice_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings notice") || lower.contains("notice settings") || lower.contains("official notice")
}

/// Fun fact about notice
pub fn notice_fun_fact() -> &'static str {
    "Anna's settings notice delivers official announcements about configuration changes!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notice_type_display() {
        assert_eq!(format!("{}", NoticeType::Information), "information");
        assert_eq!(format!("{}", NoticeType::Alert), "alert");
    }

    #[test]
    fn test_priority_display() {
        assert_eq!(format!("{}", NoticePriority::Low), "low");
        assert_eq!(format!("{}", NoticePriority::Urgent), "urgent");
    }

    #[test]
    fn test_config_new() {
        let c = NoticeConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = NoticeConfig::new("test")
            .notice_type(NoticeType::Warning)
            .default_priority(NoticePriority::High);
        assert_eq!(c.notice_type, NoticeType::Warning);
        assert_eq!(c.default_priority, NoticePriority::High);
    }

    #[test]
    fn test_entry_new() {
        let e = NoticeEntry::new("e1", "Title", "Message");
        assert_eq!(e.id, "e1");
    }

    #[test]
    fn test_entry_builder() {
        let e = NoticeEntry::new("e1", "Title", "Message")
            .priority(NoticePriority::Urgent);
        assert_eq!(e.priority, NoticePriority::Urgent);
    }

    #[test]
    fn test_entry_acknowledge() {
        let mut e = NoticeEntry::new("e1", "Title", "Message");
        e.acknowledge();
        assert!(e.acknowledged);
    }

    #[test]
    fn test_metadata_new() {
        let m = NoticeMetadata::new("key", "value", "e1");
        assert_eq!(m.entry_id, "e1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = NoticeStats::default();
        let mut entry = NoticeEntry::new("e1", "Title", "Message").priority(NoticePriority::Urgent);
        entry.acknowledge();
        s.update(&[entry], NoticeType::Alert);
        assert_eq!(s.total_notices, 1);
        assert_eq!(s.acknowledged, 1);
        assert_eq!(s.urgent, 1);
    }

    #[test]
    fn test_notice_new() {
        let n = SettingsNotice::new(NoticeConfig::default());
        assert_eq!(n.entry_count(), 0);
    }

    #[test]
    fn test_notice_add_entry() {
        let mut n = SettingsNotice::new(NoticeConfig::default());
        n.add_entry(NoticeEntry::new("e1", "Title", "Message"));
        assert_eq!(n.entry_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = NoticeRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = NoticeRegistry::new();
        r.register("n1", SettingsNotice::new(NoticeConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_notice_query() {
        assert!(is_notice_query("settings notice"));
        assert!(!is_notice_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = notice_fun_fact();
        assert!(fact.contains("notice"));
    }
}
