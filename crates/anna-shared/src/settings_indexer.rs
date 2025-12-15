// v0.0.669: Settings Indexer (Phase 245)
// Index settings for fast lookup and search

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Index type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum IndexType {
    /// Hash index for exact lookups
    #[default]
    Hash,
    /// B-tree index for range queries
    BTree,
    /// Full-text index for search
    FullText,
    /// Prefix index for prefix matches
    Prefix,
    /// Inverted index for multi-value
    Inverted,
}

impl std::fmt::Display for IndexType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hash => write!(f, "hash"),
            Self::BTree => write!(f, "btree"),
            Self::FullText => write!(f, "fulltext"),
            Self::Prefix => write!(f, "prefix"),
            Self::Inverted => write!(f, "inverted"),
        }
    }
}

/// Index status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum IndexStatus {
    /// Ready for use
    #[default]
    Ready,
    /// Building index
    Building,
    /// Needs rebuild
    Stale,
    /// Index error
    Error,
}

impl std::fmt::Display for IndexStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ready => write!(f, "ready"),
            Self::Building => write!(f, "building"),
            Self::Stale => write!(f, "stale"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// Indexer config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerConfig {
    /// Default index type
    pub default_type: IndexType,
    /// Auto rebuild on changes
    pub auto_rebuild: bool,
    /// Max index entries
    pub max_entries: usize,
    /// Enable statistics
    pub enable_stats: bool,
}

impl IndexerConfig {
    /// Create new config
    pub fn new(index_type: IndexType) -> Self {
        Self {
            default_type: index_type,
            auto_rebuild: true,
            max_entries: 100000,
            enable_stats: true,
        }
    }

    /// Set auto rebuild
    pub fn auto_rebuild(mut self, auto: bool) -> Self {
        self.auto_rebuild = auto;
        self
    }

    /// Set max entries
    pub fn max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }
}

impl Default for IndexerConfig {
    fn default() -> Self {
        Self::new(IndexType::Hash)
    }
}

/// Index entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    /// Key
    pub key: String,
    /// Value hash
    pub value_hash: u64,
    /// Created timestamp
    pub created: u64,
    /// Indexed terms (for full-text)
    pub terms: Vec<String>,
}

impl IndexEntry {
    /// Create new entry
    pub fn new(key: impl Into<String>, value: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let key_str = key.into();
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        
        Self {
            key: key_str,
            value_hash: hasher.finish(),
            created: 0,
            terms: value.split_whitespace().map(|s| s.to_lowercase()).collect(),
        }
    }

    /// With timestamp
    pub fn with_timestamp(mut self, ts: u64) -> Self {
        self.created = ts;
        self
    }
}

/// Index lookup result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexLookupResult {
    /// Found entries
    pub entries: Vec<String>,
    /// Lookup time (ns)
    pub lookup_time_ns: u64,
    /// Index used
    pub index_used: String,
    /// Hit count
    pub hit_count: usize,
}

impl IndexLookupResult {
    /// Create new result
    pub fn new(entries: Vec<String>, index: impl Into<String>) -> Self {
        let hit_count = entries.len();
        Self {
            entries,
            lookup_time_ns: 0,
            index_used: index.into(),
            hit_count,
        }
    }

    /// With time
    pub fn with_time(mut self, time_ns: u64) -> Self {
        self.lookup_time_ns = time_ns;
        self
    }

    /// Has results
    pub fn has_results(&self) -> bool {
        !self.entries.is_empty()
    }
}

impl Default for IndexLookupResult {
    fn default() -> Self {
        Self::new(Vec::new(), "none")
    }
}

/// Indexer stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IndexerStats {
    /// Total lookups
    pub total_lookups: usize,
    /// Total hits
    pub total_hits: usize,
    /// Total misses
    pub total_misses: usize,
    /// Index builds
    pub index_builds: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl IndexerStats {
    /// Record lookup
    pub fn record_lookup(&mut self, result: &IndexLookupResult) {
        self.total_lookups += 1;
        if result.has_results() {
            self.total_hits += result.hit_count;
        } else {
            self.total_misses += 1;
        }
    }

    /// Record build
    pub fn record_build(&mut self, index_type: IndexType) {
        self.index_builds += 1;
        *self.by_type.entry(index_type.to_string()).or_insert(0) += 1;
    }

    /// Hit rate
    pub fn hit_rate(&self) -> f64 {
        if self.total_lookups == 0 {
            0.0
        } else {
            (self.total_lookups - self.total_misses) as f64 / self.total_lookups as f64
        }
    }
}

/// Settings indexer
#[derive(Debug, Clone, Default)]
pub struct SettingsIndexer {
    /// Config
    config: IndexerConfig,
    /// Index entries
    entries: HashMap<String, IndexEntry>,
    /// Status
    status: IndexStatus,
    /// Stats
    stats: IndexerStats,
}

impl SettingsIndexer {
    /// Create new indexer
    pub fn new(config: IndexerConfig) -> Self {
        Self {
            config,
            entries: HashMap::new(),
            status: IndexStatus::Ready,
            stats: IndexerStats::default(),
        }
    }

    /// Index settings
    pub fn index(&mut self, settings: &HashMap<String, String>) {
        self.status = IndexStatus::Building;
        self.entries.clear();

        for (key, value) in settings {
            if self.entries.len() >= self.config.max_entries {
                break;
            }
            let entry = IndexEntry::new(key, value);
            self.entries.insert(key.clone(), entry);
        }

        self.stats.record_build(self.config.default_type);
        self.status = IndexStatus::Ready;
    }

    /// Lookup by key
    pub fn lookup(&mut self, key: &str) -> IndexLookupResult {
        let result = if self.entries.contains_key(key) {
            IndexLookupResult::new(vec![key.to_string()], "hash")
        } else {
            IndexLookupResult::default()
        };
        
        self.stats.record_lookup(&result);
        result
    }

    /// Search by prefix
    pub fn search_prefix(&mut self, prefix: &str) -> IndexLookupResult {
        let matches: Vec<String> = self.entries.keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        
        let result = IndexLookupResult::new(matches, "prefix");
        self.stats.record_lookup(&result);
        result
    }

    /// Search by term
    pub fn search_term(&mut self, term: &str) -> IndexLookupResult {
        let lower_term = term.to_lowercase();
        let matches: Vec<String> = self.entries.iter()
            .filter(|(_, e)| e.terms.contains(&lower_term))
            .map(|(k, _)| k.clone())
            .collect();
        
        let result = IndexLookupResult::new(matches, "fulltext");
        self.stats.record_lookup(&result);
        result
    }

    /// Get status
    pub fn status(&self) -> IndexStatus {
        self.status
    }

    /// Get stats
    pub fn stats(&self) -> &IndexerStats {
        &self.stats
    }

    /// Entry count
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

/// Indexer registry
#[derive(Debug, Clone, Default)]
pub struct IndexerRegistry {
    /// Indexers by ID
    indexers: HashMap<String, SettingsIndexer>,
}

impl IndexerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register indexer
    pub fn register(&mut self, id: impl Into<String>, indexer: SettingsIndexer) {
        self.indexers.insert(id.into(), indexer);
    }

    /// Unregister indexer
    pub fn unregister(&mut self, id: &str) -> bool {
        self.indexers.remove(id).is_some()
    }

    /// Get indexer
    pub fn get(&self, id: &str) -> Option<&SettingsIndexer> {
        self.indexers.get(id)
    }

    /// Get indexer mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsIndexer> {
        self.indexers.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.indexers.len()
    }
}

/// Format indexer registry
pub fn format_indexer_registry(registry: &IndexerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Indexer Registry:\n");
    output.push_str(&format!("  Indexers: {}\n", registry.count()));
    output
}

/// Check if query is about indexer
pub fn is_indexer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("index") || lower.contains("settings index") || lower.contains("search settings")
}

/// Fun fact about indexer
pub fn indexer_fun_fact() -> &'static str {
    "Anna's settings indexer enables fast lookup and full-text search!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_type_display() {
        assert_eq!(format!("{}", IndexType::Hash), "hash");
        assert_eq!(format!("{}", IndexType::FullText), "fulltext");
    }

    #[test]
    fn test_index_status_display() {
        assert_eq!(format!("{}", IndexStatus::Ready), "ready");
        assert_eq!(format!("{}", IndexStatus::Building), "building");
    }

    #[test]
    fn test_config_new() {
        let c = IndexerConfig::new(IndexType::Hash);
        assert!(c.auto_rebuild);
    }

    #[test]
    fn test_config_builder() {
        let c = IndexerConfig::new(IndexType::BTree)
            .auto_rebuild(false)
            .max_entries(1000);
        assert!(!c.auto_rebuild);
        assert_eq!(c.max_entries, 1000);
    }

    #[test]
    fn test_entry_new() {
        let e = IndexEntry::new("key", "hello world");
        assert_eq!(e.key, "key");
        assert_eq!(e.terms, vec!["hello", "world"]);
    }

    #[test]
    fn test_result_new() {
        let r = IndexLookupResult::new(vec!["k1".to_string()], "hash");
        assert!(r.has_results());
        assert_eq!(r.hit_count, 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = IndexerStats::default();
        let r = IndexLookupResult::new(vec!["k".to_string()], "hash");
        s.record_lookup(&r);
        assert_eq!(s.total_lookups, 1);
        assert_eq!(s.total_hits, 1);
    }

    #[test]
    fn test_indexer_new() {
        let i = SettingsIndexer::new(IndexerConfig::default());
        assert_eq!(i.entry_count(), 0);
    }

    #[test]
    fn test_indexer_index() {
        let mut i = SettingsIndexer::new(IndexerConfig::default());
        let mut settings = HashMap::new();
        settings.insert("key1".to_string(), "value1".to_string());
        settings.insert("key2".to_string(), "value2".to_string());
        
        i.index(&settings);
        assert_eq!(i.entry_count(), 2);
    }

    #[test]
    fn test_indexer_lookup() {
        let mut i = SettingsIndexer::new(IndexerConfig::default());
        let mut settings = HashMap::new();
        settings.insert("key".to_string(), "value".to_string());
        i.index(&settings);
        
        let result = i.lookup("key");
        assert!(result.has_results());
    }

    #[test]
    fn test_indexer_search_prefix() {
        let mut i = SettingsIndexer::new(IndexerConfig::default());
        let mut settings = HashMap::new();
        settings.insert("app.name".to_string(), "test".to_string());
        settings.insert("app.version".to_string(), "1.0".to_string());
        settings.insert("db.host".to_string(), "localhost".to_string());
        i.index(&settings);
        
        let result = i.search_prefix("app.");
        assert_eq!(result.hit_count, 2);
    }

    #[test]
    fn test_registry_new() {
        let r = IndexerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = IndexerRegistry::new();
        r.register("i1", SettingsIndexer::new(IndexerConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_indexer_query() {
        assert!(is_indexer_query("search settings"));
        assert!(!is_indexer_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = indexer_fun_fact();
        assert!(fact.contains("indexer"));
    }
}
