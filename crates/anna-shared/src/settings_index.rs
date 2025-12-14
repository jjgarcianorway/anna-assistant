// v0.0.623: Settings Index (Phase 199)
// Fast lookup index for settings with multiple access patterns

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Index type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum IndexType {
    /// Primary key index
    #[default]
    Primary,
    /// Secondary index
    Secondary,
    /// Category index
    Category,
    /// Tag index
    Tag,
    /// Full-text index
    FullText,
}

impl std::fmt::Display for IndexType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Primary => write!(f, "primary"),
            Self::Secondary => write!(f, "secondary"),
            Self::Category => write!(f, "category"),
            Self::Tag => write!(f, "tag"),
            Self::FullText => write!(f, "full_text"),
        }
    }
}

/// Index status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum IndexStatus {
    /// Building
    Building,
    /// Ready
    #[default]
    Ready,
    /// Stale
    Stale,
    /// Rebuilding
    Rebuilding,
    /// Error
    Error,
}

impl std::fmt::Display for IndexStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Building => write!(f, "building"),
            Self::Ready => write!(f, "ready"),
            Self::Stale => write!(f, "stale"),
            Self::Rebuilding => write!(f, "rebuilding"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// Index entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    /// Key
    pub key: String,
    /// Value reference
    pub value_ref: String,
    /// Category
    pub category: SettingsCategory,
    /// Tags
    pub tags: Vec<String>,
    /// Created timestamp
    pub created_at: u64,
}

impl IndexEntry {
    /// Create new entry
    pub fn new(key: impl Into<String>, value_ref: impl Into<String>, category: SettingsCategory) -> Self {
        Self {
            key: key.into(),
            value_ref: value_ref.into(),
            category,
            tags: Vec::new(),
            created_at: 0,
        }
    }

    /// Add tag
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set created timestamp
    pub fn created_at(mut self, ts: u64) -> Self {
        self.created_at = ts;
        self
    }

    /// Has tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

/// Index statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IndexStats {
    /// Entry count
    pub entry_count: usize,
    /// Query count
    pub query_count: usize,
    /// Hit count
    pub hit_count: usize,
    /// Last rebuild timestamp
    pub last_rebuild: u64,
}

impl IndexStats {
    /// Record query
    pub fn record_query(&mut self, hit: bool) {
        self.query_count += 1;
        if hit {
            self.hit_count += 1;
        }
    }

    /// Hit rate
    pub fn hit_rate(&self) -> f64 {
        if self.query_count == 0 {
            1.0
        } else {
            self.hit_count as f64 / self.query_count as f64
        }
    }
}

/// Settings index
#[derive(Debug, Clone, Default)]
pub struct SettingsIndex {
    /// Status
    status: IndexStatus,
    /// Primary index
    primary: HashMap<String, IndexEntry>,
    /// Category index
    by_category: HashMap<SettingsCategory, Vec<String>>,
    /// Tag index
    by_tag: HashMap<String, Vec<String>>,
    /// Statistics
    stats: IndexStats,
}

impl SettingsIndex {
    /// Create new index
    pub fn new() -> Self {
        Self::default()
    }

    /// Get status
    pub fn status(&self) -> IndexStatus {
        self.status
    }

    /// Set status
    pub fn set_status(&mut self, status: IndexStatus) {
        self.status = status;
    }

    /// Add entry
    pub fn add(&mut self, entry: IndexEntry) {
        let key = entry.key.clone();
        let category = entry.category;
        let tags = entry.tags.clone();

        // Add to category index
        self.by_category.entry(category).or_default().push(key.clone());

        // Add to tag index
        for tag in &tags {
            self.by_tag.entry(tag.clone()).or_default().push(key.clone());
        }

        // Add to primary index
        self.primary.insert(key, entry);
        self.stats.entry_count = self.primary.len();
    }

    /// Remove entry
    pub fn remove(&mut self, key: &str) -> bool {
        if let Some(entry) = self.primary.remove(key) {
            // Remove from category index
            if let Some(keys) = self.by_category.get_mut(&entry.category) {
                keys.retain(|k| k != key);
            }
            // Remove from tag index
            for tag in &entry.tags {
                if let Some(keys) = self.by_tag.get_mut(tag) {
                    keys.retain(|k| k != key);
                }
            }
            self.stats.entry_count = self.primary.len();
            true
        } else {
            false
        }
    }

    /// Get by key
    pub fn get(&mut self, key: &str) -> Option<&IndexEntry> {
        let hit = self.primary.contains_key(key);
        self.stats.record_query(hit);
        self.primary.get(key)
    }

    /// Get by category
    pub fn get_by_category(&self, category: SettingsCategory) -> Vec<&IndexEntry> {
        self.by_category
            .get(&category)
            .map(|keys| keys.iter().filter_map(|k| self.primary.get(k)).collect())
            .unwrap_or_default()
    }

    /// Get by tag
    pub fn get_by_tag(&self, tag: &str) -> Vec<&IndexEntry> {
        self.by_tag
            .get(tag)
            .map(|keys| keys.iter().filter_map(|k| self.primary.get(k)).collect())
            .unwrap_or_default()
    }

    /// Entry count
    pub fn count(&self) -> usize {
        self.primary.len()
    }

    /// Get stats
    pub fn stats(&self) -> &IndexStats {
        &self.stats
    }

    /// Is ready
    pub fn is_ready(&self) -> bool {
        self.status == IndexStatus::Ready
    }

    /// Mark rebuild
    pub fn mark_rebuild(&mut self, timestamp: u64) {
        self.stats.last_rebuild = timestamp;
        self.status = IndexStatus::Ready;
    }
}

/// Format index
pub fn format_index(index: &SettingsIndex) -> String {
    let mut output = String::new();
    output.push_str("Settings Index:\n");
    output.push_str(&format!("  Status: {}\n", index.status()));
    output.push_str(&format!("  Entries: {}\n", index.count()));
    output.push_str(&format!("  Hit Rate: {:.1}%\n", index.stats().hit_rate() * 100.0));
    output
}

/// Check if query is about index
pub fn is_index_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("index")
        || lower.contains("settings index")
        || lower.contains("lookup index")
}

/// Fun fact about index
pub fn index_fun_fact() -> &'static str {
    "Anna's settings index provides fast multi-pattern lookups for settings!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_type_display() {
        assert_eq!(format!("{}", IndexType::Primary), "primary");
        assert_eq!(format!("{}", IndexType::Tag), "tag");
    }

    #[test]
    fn test_index_status_display() {
        assert_eq!(format!("{}", IndexStatus::Ready), "ready");
        assert_eq!(format!("{}", IndexStatus::Stale), "stale");
    }

    #[test]
    fn test_entry_new() {
        let e = IndexEntry::new("key", "ref", SettingsCategory::Privacy);
        assert!(e.tags.is_empty());
    }

    #[test]
    fn test_entry_tag() {
        let e = IndexEntry::new("key", "ref", SettingsCategory::Privacy)
            .tag("important");
        assert!(e.has_tag("important"));
    }

    #[test]
    fn test_stats_record() {
        let mut s = IndexStats::default();
        s.record_query(true);
        s.record_query(false);
        assert_eq!(s.query_count, 2);
    }

    #[test]
    fn test_index_new() {
        let i = SettingsIndex::new();
        assert!(i.is_ready());
    }

    #[test]
    fn test_index_add() {
        let mut i = SettingsIndex::new();
        i.add(IndexEntry::new("k1", "r1", SettingsCategory::Privacy));
        assert_eq!(i.count(), 1);
    }

    #[test]
    fn test_index_get() {
        let mut i = SettingsIndex::new();
        i.add(IndexEntry::new("k1", "r1", SettingsCategory::Privacy));
        assert!(i.get("k1").is_some());
    }

    #[test]
    fn test_index_remove() {
        let mut i = SettingsIndex::new();
        i.add(IndexEntry::new("k1", "r1", SettingsCategory::Privacy));
        assert!(i.remove("k1"));
    }

    #[test]
    fn test_index_by_category() {
        let mut i = SettingsIndex::new();
        i.add(IndexEntry::new("k1", "r1", SettingsCategory::Privacy));
        assert_eq!(i.get_by_category(SettingsCategory::Privacy).len(), 1);
    }

    #[test]
    fn test_is_index_query() {
        assert!(is_index_query("settings index"));
        assert!(!is_index_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = index_fun_fact();
        assert!(fact.contains("index"));
    }
}
