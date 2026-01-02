// v0.0.669: Settings Indexer Types (Phase 245)
// Index types and configuration structures

use serde::{Deserialize, Serialize};

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
