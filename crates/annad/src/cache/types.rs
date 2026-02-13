//! Cache data structures and types.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Tags that trigger cache invalidation when system events occur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InvalidationTag {
    /// Hardware changes (PCI devices, USB)
    Hardware,
    /// Block devices (disks, NVMe, USB storage)
    BlockDevice,
    /// Memory (RAM added/removed)
    Memory,
    /// Partition table changes
    Partitions,
    /// Bootloader config changes
    Bootloader,
    /// Package installations/removals
    Packages,
    /// Systemd service files
    Services,
    /// Network interfaces
    Network,
    /// DNS configuration
    DnsConfig,
    /// Process list (short-lived)
    Process,
    /// Filesystem mounts
    Fstab,
}

/// A cached command result with metadata.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// The command output
    pub value: String,
    /// When this was captured
    pub captured_at: Instant,
    /// TTL fallback (invalidate after this even without events)
    pub ttl: Duration,
    /// Which events invalidate this entry
    pub tags: Vec<InvalidationTag>,
}

impl CacheEntry {
    pub fn new(value: String, ttl: Duration, tags: Vec<InvalidationTag>) -> Self {
        Self {
            value,
            captured_at: Instant::now(),
            ttl,
            tags,
        }
    }

    /// Check if this entry is still valid (TTL not expired).
    pub fn is_valid(&self) -> bool {
        self.captured_at.elapsed() < self.ttl
    }
}

/// Thread-safe system command cache.
#[derive(Clone)]
pub struct SystemCache {
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

impl SystemCache {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get cached value if valid, otherwise None.
    pub fn get(&self, key: &str) -> Option<String> {
        let entries = self.entries.read().ok()?;
        let entry = entries.get(key)?;
        if entry.is_valid() {
            Some(entry.value.clone())
        } else {
            None
        }
    }

    /// Store a value in the cache.
    pub fn set(&self, key: String, value: String, ttl: Duration, tags: Vec<InvalidationTag>) {
        if let Ok(mut entries) = self.entries.write() {
            entries.insert(key, CacheEntry::new(value, ttl, tags));
        }
    }

    /// Invalidate all entries with the given tag.
    pub fn invalidate_tag(&self, tag: InvalidationTag) {
        if let Ok(mut entries) = self.entries.write() {
            entries.retain(|_, entry| !entry.tags.contains(&tag));
        }
    }

    /// Invalidate all expired entries.
    pub fn cleanup_expired(&self) {
        if let Ok(mut entries) = self.entries.write() {
            entries.retain(|_, entry| entry.is_valid());
        }
    }

    /// Get cache statistics for monitoring.
    pub fn stats(&self) -> CacheStats {
        let entries = self.entries.read().ok();
        let count = entries.as_ref().map(|e| e.len()).unwrap_or(0);
        let valid = entries
            .as_ref()
            .map(|e| e.values().filter(|entry| entry.is_valid()).count())
            .unwrap_or(0);

        CacheStats {
            total_entries: count,
            valid_entries: valid,
            expired_entries: count.saturating_sub(valid),
        }
    }
}

impl Default for SystemCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics.
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub valid_entries: usize,
    pub expired_entries: usize,
}
