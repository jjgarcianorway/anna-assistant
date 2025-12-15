// v0.0.707: Settings Journal (Phase 283)
// Personal journal for settings reflections

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Journal type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum JournalType {
    /// Personal journal
    #[default]
    Personal,
    /// Technical journal
    Technical,
    /// Research journal
    Research,
    /// Log journal
    Log,
}

impl std::fmt::Display for JournalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Personal => write!(f, "personal"),
            Self::Technical => write!(f, "technical"),
            Self::Research => write!(f, "research"),
            Self::Log => write!(f, "log"),
        }
    }
}

/// Journal mood
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum JournalMood {
    /// Productive
    #[default]
    Productive,
    /// Challenging
    Challenging,
    /// Learning
    Learning,
    /// Resolved
    Resolved,
}

impl std::fmt::Display for JournalMood {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Productive => write!(f, "productive"),
            Self::Challenging => write!(f, "challenging"),
            Self::Learning => write!(f, "learning"),
            Self::Resolved => write!(f, "resolved"),
        }
    }
}

/// Journal config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalConfig {
    /// Name
    pub name: String,
    /// Journal type
    pub journal_type: JournalType,
    /// Max entries
    pub max_entries: usize,
    /// Private
    pub private: bool,
}

impl JournalConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            journal_type: JournalType::Personal,
            max_entries: 1000,
            private: true,
        }
    }

    /// Set type
    pub fn journal_type(mut self, jt: JournalType) -> Self {
        self.journal_type = jt;
        self
    }

    /// Set max entries
    pub fn max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// Set private
    pub fn private(mut self, p: bool) -> Self {
        self.private = p;
        self
    }
}

impl Default for JournalConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Journal entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    /// Entry ID
    pub id: usize,
    /// Date
    pub date: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Mood
    pub mood: JournalMood,
    /// Tags
    pub tags: Vec<String>,
}

impl JournalEntry {
    /// Create new entry
    pub fn new(id: usize, date: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id,
            date: date.into(),
            title: title.into(),
            content: content.into(),
            mood: JournalMood::Productive,
            tags: Vec::new(),
        }
    }

    /// Set mood
    pub fn mood(mut self, m: JournalMood) -> Self {
        self.mood = m;
        self
    }

    /// Add tag
    pub fn tag(mut self, t: impl Into<String>) -> Self {
        self.tags.push(t.into());
        self
    }
}

/// Journal item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalItem {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Entry ID
    pub entry_id: usize,
    /// Reflection
    pub reflection: Option<String>,
}

impl JournalItem {
    /// Create new item
    pub fn new(key: impl Into<String>, value: impl Into<String>, entry_id: usize) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            entry_id,
            reflection: None,
        }
    }

    /// Set reflection
    pub fn reflection(mut self, r: impl Into<String>) -> Self {
        self.reflection = Some(r.into());
        self
    }
}

/// Journal stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JournalStats {
    /// Total entries
    pub total_entries: usize,
    /// Total items
    pub total_items: usize,
    /// By mood
    pub by_mood: HashMap<String, usize>,
    /// Total tags
    pub total_tags: usize,
}

impl JournalStats {
    /// Update from journal
    pub fn update(&mut self, entries: &[JournalEntry]) {
        self.total_entries = entries.len();
        self.by_mood.clear();
        self.total_tags = 0;
        for entry in entries {
            *self.by_mood.entry(entry.mood.to_string()).or_insert(0) += 1;
            self.total_tags += entry.tags.len();
        }
    }

    /// Record item
    pub fn record_item(&mut self) {
        self.total_items += 1;
    }

    /// Avg tags per entry
    pub fn avg_tags(&self) -> f64 {
        if self.total_entries == 0 { 0.0 } else { self.total_tags as f64 / self.total_entries as f64 }
    }
}

/// Settings journal
#[derive(Debug, Clone, Default)]
pub struct SettingsJournal {
    /// Config
    config: JournalConfig,
    /// Entries
    entries: Vec<JournalEntry>,
    /// Items
    items: Vec<JournalItem>,
    /// Stats
    stats: JournalStats,
    /// Next ID
    next_id: usize,
}

impl SettingsJournal {
    /// Create new journal
    pub fn new(config: JournalConfig) -> Self {
        Self {
            config,
            entries: Vec::new(),
            items: Vec::new(),
            stats: JournalStats::default(),
            next_id: 1,
        }
    }

    /// Write entry
    pub fn write(&mut self, date: &str, title: &str, content: &str) -> usize {
        if self.entries.len() >= self.config.max_entries {
            return 0;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.entries.push(JournalEntry::new(id, date, title, content));
        self.update_stats();
        id
    }

    /// Get entry
    pub fn get_entry(&self, id: usize) -> Option<&JournalEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Add item
    pub fn add_item(&mut self, item: JournalItem) {
        self.items.push(item);
        self.stats.record_item();
    }

    /// Get items for entry
    pub fn get_items(&self, entry_id: usize) -> Vec<&JournalItem> {
        self.items.iter().filter(|i| i.entry_id == entry_id).collect()
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.entries);
    }

    /// Get stats
    pub fn stats(&self) -> &JournalStats {
        &self.stats
    }

    /// Entry count
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

/// Journal registry
#[derive(Debug, Clone, Default)]
pub struct JournalRegistry {
    /// Journals by ID
    journals: HashMap<String, SettingsJournal>,
}

impl JournalRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register journal
    pub fn register(&mut self, id: impl Into<String>, journal: SettingsJournal) {
        self.journals.insert(id.into(), journal);
    }

    /// Unregister journal
    pub fn unregister(&mut self, id: &str) -> bool {
        self.journals.remove(id).is_some()
    }

    /// Get journal
    pub fn get(&self, id: &str) -> Option<&SettingsJournal> {
        self.journals.get(id)
    }

    /// Get journal mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsJournal> {
        self.journals.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.journals.len()
    }
}

/// Format journal registry
pub fn format_journal_registry(registry: &JournalRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Journal Registry:\n");
    output.push_str(&format!("  Journals: {}\n", registry.count()));
    output
}

/// Check if query is about journal
pub fn is_journal_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings journal") || lower.contains("journal settings") || lower.contains("config journal")
}

/// Fun fact about journal
pub fn journal_fun_fact() -> &'static str {
    "Anna's settings journal helps you reflect on your configuration journey!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_journal_type_display() {
        assert_eq!(format!("{}", JournalType::Personal), "personal");
        assert_eq!(format!("{}", JournalType::Technical), "technical");
    }

    #[test]
    fn test_mood_display() {
        assert_eq!(format!("{}", JournalMood::Productive), "productive");
        assert_eq!(format!("{}", JournalMood::Learning), "learning");
    }

    #[test]
    fn test_config_new() {
        let c = JournalConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = JournalConfig::new("test")
            .journal_type(JournalType::Research)
            .private(false);
        assert_eq!(c.journal_type, JournalType::Research);
        assert!(!c.private);
    }

    #[test]
    fn test_entry_new() {
        let e = JournalEntry::new(1, "2025-12-15", "Title", "Content");
        assert_eq!(e.id, 1);
    }

    #[test]
    fn test_entry_builder() {
        let e = JournalEntry::new(1, "2025-12-15", "Title", "Content")
            .mood(JournalMood::Learning)
            .tag("config");
        assert_eq!(e.mood, JournalMood::Learning);
        assert_eq!(e.tags.len(), 1);
    }

    #[test]
    fn test_item_new() {
        let i = JournalItem::new("key", "value", 1);
        assert_eq!(i.entry_id, 1);
    }

    #[test]
    fn test_item_reflection() {
        let i = JournalItem::new("key", "value", 1).reflection("Interesting change");
        assert!(i.reflection.is_some());
    }

    #[test]
    fn test_stats_update() {
        let mut s = JournalStats::default();
        let entries = vec![JournalEntry::new(1, "2025-12-15", "Title", "Content").tag("test")];
        s.update(&entries);
        assert_eq!(s.total_entries, 1);
        assert_eq!(s.total_tags, 1);
    }

    #[test]
    fn test_journal_new() {
        let j = SettingsJournal::new(JournalConfig::default());
        assert_eq!(j.entry_count(), 0);
    }

    #[test]
    fn test_journal_write() {
        let mut j = SettingsJournal::new(JournalConfig::default());
        let id = j.write("2025-12-15", "Title", "Content");
        assert_eq!(id, 1);
        assert_eq!(j.entry_count(), 1);
    }

    #[test]
    fn test_journal_add_item() {
        let mut j = SettingsJournal::new(JournalConfig::default());
        j.add_item(JournalItem::new("key", "value", 1));
        assert_eq!(j.stats().total_items, 1);
    }

    #[test]
    fn test_registry_new() {
        let r = JournalRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = JournalRegistry::new();
        r.register("j1", SettingsJournal::new(JournalConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_journal_query() {
        assert!(is_journal_query("settings journal"));
        assert!(!is_journal_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = journal_fun_fact();
        assert!(fact.contains("journal"));
    }
}
