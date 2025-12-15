// v0.0.711: Settings Summary (Phase 287)
// Comprehensive summary of settings state

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Summary type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SummaryType {
    /// Quick summary
    #[default]
    Quick,
    /// Detailed summary
    Detailed,
    /// Comprehensive summary
    Comprehensive,
    /// Overview summary
    Overview,
}

impl std::fmt::Display for SummaryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quick => write!(f, "quick"),
            Self::Detailed => write!(f, "detailed"),
            Self::Comprehensive => write!(f, "comprehensive"),
            Self::Overview => write!(f, "overview"),
        }
    }
}

/// Summary depth
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SummaryDepth {
    /// Shallow depth
    #[default]
    Shallow,
    /// Medium depth
    Medium,
    /// Deep depth
    Deep,
    /// Full depth
    Full,
}

impl std::fmt::Display for SummaryDepth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Shallow => write!(f, "shallow"),
            Self::Medium => write!(f, "medium"),
            Self::Deep => write!(f, "deep"),
            Self::Full => write!(f, "full"),
        }
    }
}

/// Summary config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryConfig {
    /// Name
    pub name: String,
    /// Summary type
    pub summary_type: SummaryType,
    /// Depth
    pub depth: SummaryDepth,
    /// Max entries
    pub max_entries: usize,
}

impl SummaryConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            summary_type: SummaryType::Quick,
            depth: SummaryDepth::Shallow,
            max_entries: 100,
        }
    }

    /// Set type
    pub fn summary_type(mut self, st: SummaryType) -> Self {
        self.summary_type = st;
        self
    }

    /// Set depth
    pub fn depth(mut self, d: SummaryDepth) -> Self {
        self.depth = d;
        self
    }

    /// Set max entries
    pub fn max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }
}

impl Default for SummaryConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Summary entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryEntry {
    /// Entry ID
    pub id: String,
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Category
    pub category: String,
    /// Important
    pub important: bool,
}

impl SummaryEntry {
    /// Create new entry
    pub fn new(id: impl Into<String>, key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            key: key.into(),
            value: value.into(),
            category: String::new(),
            important: false,
        }
    }

    /// Set category
    pub fn category(mut self, c: impl Into<String>) -> Self {
        self.category = c.into();
        self
    }

    /// Set important
    pub fn important(mut self, i: bool) -> Self {
        self.important = i;
        self
    }
}

/// Summary metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryMetadata {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Entry ID
    pub entry_id: String,
}

impl SummaryMetadata {
    /// Create new metadata
    pub fn new(key: impl Into<String>, value: impl Into<String>, entry_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            entry_id: entry_id.into(),
        }
    }
}

/// Summary stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SummaryStats {
    /// Total entries
    pub total_entries: usize,
    /// Important entries
    pub important_entries: usize,
    /// Categories
    pub categories: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl SummaryStats {
    /// Update from entries
    pub fn update(&mut self, entries: &[SummaryEntry], summary_type: SummaryType) {
        self.total_entries = entries.len();
        self.important_entries = entries.iter().filter(|e| e.important).count();
        let cats: std::collections::HashSet<_> = entries.iter()
            .filter(|e| !e.category.is_empty())
            .map(|e| &e.category)
            .collect();
        self.categories = cats.len();
        *self.by_type.entry(summary_type.to_string()).or_insert(0) += 1;
    }

    /// Important rate
    pub fn important_rate(&self) -> f64 {
        if self.total_entries == 0 { 0.0 } else { self.important_entries as f64 / self.total_entries as f64 * 100.0 }
    }
}

/// Settings summary
#[derive(Debug, Clone, Default)]
pub struct SettingsSummary {
    /// Config
    config: SummaryConfig,
    /// Entries
    entries: Vec<SummaryEntry>,
    /// Metadata
    metadata: Vec<SummaryMetadata>,
    /// Stats
    stats: SummaryStats,
}

impl SettingsSummary {
    /// Create new summary
    pub fn new(config: SummaryConfig) -> Self {
        Self {
            config,
            entries: Vec::new(),
            metadata: Vec::new(),
            stats: SummaryStats::default(),
        }
    }

    /// Add entry
    pub fn add_entry(&mut self, entry: SummaryEntry) -> bool {
        if self.entries.len() >= self.config.max_entries {
            return false;
        }
        self.entries.push(entry);
        self.update_stats();
        true
    }

    /// Get entry
    pub fn get_entry(&self, id: &str) -> Option<&SummaryEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Get entry mut
    pub fn get_entry_mut(&mut self, id: &str) -> Option<&mut SummaryEntry> {
        self.entries.iter_mut().find(|e| e.id == id)
    }

    /// Add metadata
    pub fn add_metadata(&mut self, meta: SummaryMetadata) {
        self.metadata.push(meta);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.entries, self.config.summary_type);
    }

    /// Get stats
    pub fn stats(&self) -> &SummaryStats {
        &self.stats
    }

    /// Entry count
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

/// Summary registry
#[derive(Debug, Clone, Default)]
pub struct SummaryRegistry {
    /// Summaries by ID
    summaries: HashMap<String, SettingsSummary>,
}

impl SummaryRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register summary
    pub fn register(&mut self, id: impl Into<String>, summary: SettingsSummary) {
        self.summaries.insert(id.into(), summary);
    }

    /// Unregister summary
    pub fn unregister(&mut self, id: &str) -> bool {
        self.summaries.remove(id).is_some()
    }

    /// Get summary
    pub fn get(&self, id: &str) -> Option<&SettingsSummary> {
        self.summaries.get(id)
    }

    /// Get summary mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsSummary> {
        self.summaries.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.summaries.len()
    }
}

/// Format summary registry
pub fn format_summary_registry(registry: &SummaryRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Summary Registry:\n");
    output.push_str(&format!("  Summaries: {}\n", registry.count()));
    output
}

/// Check if query is about summary
pub fn is_summary_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings summary") || lower.contains("summary settings") || lower.contains("quick summary")
}

/// Fun fact about summary
pub fn summary_fun_fact() -> &'static str {
    "Anna's settings summary provides comprehensive overviews of configuration states!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summary_type_display() {
        assert_eq!(format!("{}", SummaryType::Quick), "quick");
        assert_eq!(format!("{}", SummaryType::Detailed), "detailed");
    }

    #[test]
    fn test_depth_display() {
        assert_eq!(format!("{}", SummaryDepth::Shallow), "shallow");
        assert_eq!(format!("{}", SummaryDepth::Deep), "deep");
    }

    #[test]
    fn test_config_new() {
        let c = SummaryConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = SummaryConfig::new("test")
            .summary_type(SummaryType::Comprehensive)
            .depth(SummaryDepth::Full);
        assert_eq!(c.summary_type, SummaryType::Comprehensive);
        assert_eq!(c.depth, SummaryDepth::Full);
    }

    #[test]
    fn test_entry_new() {
        let e = SummaryEntry::new("e1", "key", "value");
        assert_eq!(e.id, "e1");
    }

    #[test]
    fn test_entry_builder() {
        let e = SummaryEntry::new("e1", "key", "value")
            .category("cat1")
            .important(true);
        assert_eq!(e.category, "cat1");
        assert!(e.important);
    }

    #[test]
    fn test_metadata_new() {
        let m = SummaryMetadata::new("key", "value", "e1");
        assert_eq!(m.entry_id, "e1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = SummaryStats::default();
        let entry = SummaryEntry::new("e1", "key", "value").important(true).category("cat1");
        s.update(&[entry], SummaryType::Quick);
        assert_eq!(s.total_entries, 1);
        assert_eq!(s.important_entries, 1);
        assert_eq!(s.categories, 1);
    }

    #[test]
    fn test_summary_new() {
        let s = SettingsSummary::new(SummaryConfig::default());
        assert_eq!(s.entry_count(), 0);
    }

    #[test]
    fn test_summary_add_entry() {
        let mut s = SettingsSummary::new(SummaryConfig::default());
        s.add_entry(SummaryEntry::new("e1", "key", "value"));
        assert_eq!(s.entry_count(), 1);
    }

    #[test]
    fn test_summary_add_metadata() {
        let mut s = SettingsSummary::new(SummaryConfig::default());
        s.add_metadata(SummaryMetadata::new("key", "value", "e1"));
        assert_eq!(s.metadata.len(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = SummaryRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SummaryRegistry::new();
        r.register("s1", SettingsSummary::new(SummaryConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_summary_query() {
        assert!(is_summary_query("settings summary"));
        assert!(!is_summary_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = summary_fun_fact();
        assert!(fact.contains("summary"));
    }
}
