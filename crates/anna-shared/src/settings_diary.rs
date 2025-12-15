// v0.0.694: Settings Diary (Phase 270)
// Daily diary of settings activities

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Diary entry type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DiaryEntryType {
    /// Note
    #[default]
    Note,
    /// Change
    Change,
    /// Alert
    Alert,
    /// Milestone
    Milestone,
}

impl std::fmt::Display for DiaryEntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Note => write!(f, "note"),
            Self::Change => write!(f, "change"),
            Self::Alert => write!(f, "alert"),
            Self::Milestone => write!(f, "milestone"),
        }
    }
}

/// Diary importance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DiaryImportance {
    /// Low
    #[default]
    Low,
    /// Normal
    Normal,
    /// High
    High,
    /// Critical
    Critical,
}

impl std::fmt::Display for DiaryImportance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Normal => write!(f, "normal"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// Diary config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiaryConfig {
    /// Name
    pub name: String,
    /// Max entries per day
    pub max_entries_per_day: usize,
    /// Auto summarize
    pub auto_summarize: bool,
    /// Retention days
    pub retention_days: usize,
}

impl DiaryConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            max_entries_per_day: 100,
            auto_summarize: true,
            retention_days: 30,
        }
    }

    /// Set max entries
    pub fn max_entries_per_day(mut self, max: usize) -> Self {
        self.max_entries_per_day = max;
        self
    }

    /// Set retention
    pub fn retention_days(mut self, days: usize) -> Self {
        self.retention_days = days;
        self
    }
}

impl Default for DiaryConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Diary entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiaryEntry {
    /// Entry ID
    pub id: usize,
    /// Entry type
    pub entry_type: DiaryEntryType,
    /// Content
    pub content: String,
    /// Related key
    pub related_key: Option<String>,
    /// Importance
    pub importance: DiaryImportance,
    /// Tags
    pub tags: Vec<String>,
}

impl DiaryEntry {
    /// Create new entry
    pub fn new(id: usize, entry_type: DiaryEntryType, content: impl Into<String>) -> Self {
        Self {
            id,
            entry_type,
            content: content.into(),
            related_key: None,
            importance: DiaryImportance::Normal,
            tags: Vec::new(),
        }
    }

    /// Set related key
    pub fn related_key(mut self, key: impl Into<String>) -> Self {
        self.related_key = Some(key.into());
        self
    }

    /// Set importance
    pub fn importance(mut self, imp: DiaryImportance) -> Self {
        self.importance = imp;
        self
    }

    /// Add tag
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Is important
    pub fn is_important(&self) -> bool {
        matches!(self.importance, DiaryImportance::High | DiaryImportance::Critical)
    }
}

/// Daily page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyPage {
    /// Date (YYYY-MM-DD)
    pub date: String,
    /// Entries
    pub entries: Vec<DiaryEntry>,
    /// Summary
    pub summary: Option<String>,
}

impl DailyPage {
    /// Create new page
    pub fn new(date: impl Into<String>) -> Self {
        Self {
            date: date.into(),
            entries: Vec::new(),
            summary: None,
        }
    }

    /// Add entry
    pub fn add(&mut self, entry: DiaryEntry) {
        self.entries.push(entry);
    }

    /// Set summary
    pub fn summarize(&mut self, summary: impl Into<String>) {
        self.summary = Some(summary.into());
    }

    /// Entry count
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Important entries
    pub fn important_entries(&self) -> Vec<&DiaryEntry> {
        self.entries.iter().filter(|e| e.is_important()).collect()
    }
}

/// Diary stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiaryStats {
    /// Total entries
    pub total_entries: usize,
    /// Total days
    pub total_days: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
    /// By importance
    pub by_importance: HashMap<String, usize>,
}

impl DiaryStats {
    /// Record entry
    pub fn record(&mut self, entry: &DiaryEntry) {
        self.total_entries += 1;
        *self.by_type.entry(entry.entry_type.to_string()).or_insert(0) += 1;
        *self.by_importance.entry(entry.importance.to_string()).or_insert(0) += 1;
    }

    /// Update days
    pub fn set_days(&mut self, days: usize) {
        self.total_days = days;
    }

    /// Avg entries per day
    pub fn avg_per_day(&self) -> f64 {
        if self.total_days == 0 { 0.0 } else { self.total_entries as f64 / self.total_days as f64 }
    }
}

/// Settings diary
#[derive(Debug, Clone, Default)]
pub struct SettingsDiary {
    /// Config
    config: DiaryConfig,
    /// Pages by date
    pages: HashMap<String, DailyPage>,
    /// Stats
    stats: DiaryStats,
    /// Next ID
    next_id: usize,
}

impl SettingsDiary {
    /// Create new diary
    pub fn new(config: DiaryConfig) -> Self {
        Self {
            config,
            pages: HashMap::new(),
            stats: DiaryStats::default(),
            next_id: 1,
        }
    }

    /// Get or create page for date
    fn get_or_create_page(&mut self, date: &str) -> &mut DailyPage {
        if !self.pages.contains_key(date) {
            self.pages.insert(date.to_string(), DailyPage::new(date));
            self.stats.set_days(self.pages.len());
        }
        self.pages.get_mut(date).unwrap()
    }

    /// Add note
    pub fn add_note(&mut self, date: &str, content: &str) -> usize {
        let entry = DiaryEntry::new(self.next_id, DiaryEntryType::Note, content);
        let id = self.next_id;
        self.next_id += 1;
        self.stats.record(&entry);
        self.get_or_create_page(date).add(entry);
        id
    }

    /// Add change
    pub fn add_change(&mut self, date: &str, key: &str, content: &str) -> usize {
        let entry = DiaryEntry::new(self.next_id, DiaryEntryType::Change, content)
            .related_key(key);
        let id = self.next_id;
        self.next_id += 1;
        self.stats.record(&entry);
        self.get_or_create_page(date).add(entry);
        id
    }

    /// Add alert
    pub fn add_alert(&mut self, date: &str, content: &str, importance: DiaryImportance) -> usize {
        let entry = DiaryEntry::new(self.next_id, DiaryEntryType::Alert, content)
            .importance(importance);
        let id = self.next_id;
        self.next_id += 1;
        self.stats.record(&entry);
        self.get_or_create_page(date).add(entry);
        id
    }

    /// Get page
    pub fn get_page(&self, date: &str) -> Option<&DailyPage> {
        self.pages.get(date)
    }

    /// Get stats
    pub fn stats(&self) -> &DiaryStats {
        &self.stats
    }

    /// Day count
    pub fn day_count(&self) -> usize {
        self.pages.len()
    }
}

/// Diary registry
#[derive(Debug, Clone, Default)]
pub struct DiaryRegistry {
    /// Diaries by ID
    diaries: HashMap<String, SettingsDiary>,
}

impl DiaryRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register diary
    pub fn register(&mut self, id: impl Into<String>, diary: SettingsDiary) {
        self.diaries.insert(id.into(), diary);
    }

    /// Unregister diary
    pub fn unregister(&mut self, id: &str) -> bool {
        self.diaries.remove(id).is_some()
    }

    /// Get diary
    pub fn get(&self, id: &str) -> Option<&SettingsDiary> {
        self.diaries.get(id)
    }

    /// Get diary mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsDiary> {
        self.diaries.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.diaries.len()
    }
}

/// Format diary registry
pub fn format_diary_registry(registry: &DiaryRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Diary Registry:\n");
    output.push_str(&format!("  Diaries: {}\n", registry.count()));
    output
}

/// Check if query is about diary
pub fn is_diary_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings diary") || lower.contains("diary settings") || lower.contains("daily settings")
}

/// Fun fact about diary
pub fn diary_fun_fact() -> &'static str {
    "Anna's settings diary keeps a daily record of all configuration activities!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_type_display() {
        assert_eq!(format!("{}", DiaryEntryType::Note), "note");
        assert_eq!(format!("{}", DiaryEntryType::Alert), "alert");
    }

    #[test]
    fn test_importance_display() {
        assert_eq!(format!("{}", DiaryImportance::High), "high");
        assert_eq!(format!("{}", DiaryImportance::Critical), "critical");
    }

    #[test]
    fn test_config_new() {
        let c = DiaryConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = DiaryConfig::new("test")
            .max_entries_per_day(50)
            .retention_days(7);
        assert_eq!(c.max_entries_per_day, 50);
        assert_eq!(c.retention_days, 7);
    }

    #[test]
    fn test_entry_new() {
        let e = DiaryEntry::new(1, DiaryEntryType::Note, "test note");
        assert!(!e.is_important());
    }

    #[test]
    fn test_entry_important() {
        let e = DiaryEntry::new(1, DiaryEntryType::Alert, "alert")
            .importance(DiaryImportance::High);
        assert!(e.is_important());
    }

    #[test]
    fn test_entry_tags() {
        let e = DiaryEntry::new(1, DiaryEntryType::Note, "note")
            .tag("config")
            .tag("update");
        assert_eq!(e.tags.len(), 2);
    }

    #[test]
    fn test_page_new() {
        let p = DailyPage::new("2025-12-15");
        assert_eq!(p.count(), 0);
    }

    #[test]
    fn test_page_add() {
        let mut p = DailyPage::new("2025-12-15");
        p.add(DiaryEntry::new(1, DiaryEntryType::Note, "test"));
        assert_eq!(p.count(), 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = DiaryStats::default();
        s.record(&DiaryEntry::new(1, DiaryEntryType::Note, "test"));
        assert_eq!(s.total_entries, 1);
    }

    #[test]
    fn test_diary_new() {
        let d = SettingsDiary::new(DiaryConfig::default());
        assert_eq!(d.day_count(), 0);
    }

    #[test]
    fn test_diary_add_note() {
        let mut d = SettingsDiary::new(DiaryConfig::default());
        d.add_note("2025-12-15", "test note");
        assert_eq!(d.stats().total_entries, 1);
    }

    #[test]
    fn test_diary_add_change() {
        let mut d = SettingsDiary::new(DiaryConfig::default());
        d.add_change("2025-12-15", "app.name", "Changed app name");
        assert_eq!(d.day_count(), 1);
    }

    #[test]
    fn test_diary_get_page() {
        let mut d = SettingsDiary::new(DiaryConfig::default());
        d.add_note("2025-12-15", "test");
        let page = d.get_page("2025-12-15");
        assert!(page.is_some());
    }

    #[test]
    fn test_registry_new() {
        let r = DiaryRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = DiaryRegistry::new();
        r.register("d1", SettingsDiary::new(DiaryConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_diary_query() {
        assert!(is_diary_query("settings diary"));
        assert!(!is_diary_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = diary_fun_fact();
        assert!(fact.contains("diary"));
    }
}
