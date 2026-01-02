// v0.0.586: Settings Cache Types (Phase 162)
// Type definitions for caching settings values

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

/// Cache entry state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CacheState {
    /// Valid cached value
    #[default]
    Valid,
    /// Stale (needs refresh)
    Stale,
    /// Expired (will be removed)
    Expired,
    /// Loading
    Loading,
}

impl std::fmt::Display for CacheState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Valid => write!(f, "valid"),
            Self::Stale => write!(f, "stale"),
            Self::Expired => write!(f, "expired"),
            Self::Loading => write!(f, "loading"),
        }
    }
}

/// Cache eviction policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum EvictionPolicy {
    /// Least recently used
    #[default]
    LRU,
    /// Least frequently used
    LFU,
    /// First in first out
    FIFO,
    /// Time-based expiry
    TTL,
}

impl std::fmt::Display for EvictionPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LRU => write!(f, "LRU"),
            Self::LFU => write!(f, "LFU"),
            Self::FIFO => write!(f, "FIFO"),
            Self::TTL => write!(f, "TTL"),
        }
    }
}

/// Cached value entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Entry key
    pub key: String,
    /// Cached value (serialized)
    pub value: String,
    /// Category
    pub category: Option<SettingsCategory>,
    /// State
    pub state: CacheState,
    /// Created time
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last accessed
    pub last_access: chrono::DateTime<chrono::Utc>,
    /// Access count
    pub access_count: u64,
    /// Expiry time
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Size in bytes
    pub size: usize,
}

impl CacheEntry {
    /// Create new cache entry
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        let now = chrono::Utc::now();
        let value_str = value.into();
        let size = value_str.len();
        Self {
            key: key.into(),
            value: value_str,
            category: None,
            state: CacheState::Valid,
            created_at: now,
            last_access: now,
            access_count: 0,
            expires_at: None,
            size,
        }
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set TTL in seconds
    pub fn ttl(mut self, seconds: i64) -> Self {
        self.expires_at = Some(chrono::Utc::now() + chrono::Duration::seconds(seconds));
        self
    }

    /// Check if expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            chrono::Utc::now() > expires
        } else {
            false
        }
    }

    /// Mark as accessed
    pub fn touch(&mut self) {
        self.last_access = chrono::Utc::now();
        self.access_count += 1;
    }

    /// Mark as stale
    pub fn mark_stale(&mut self) {
        self.state = CacheState::Stale;
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total hits
    pub hits: u64,
    /// Total misses
    pub misses: u64,
    /// Evictions
    pub evictions: u64,
    /// Current entry count
    pub entries: usize,
    /// Total size in bytes
    pub size: usize,
}

impl CacheStats {
    /// Hit rate (0.0-1.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}
