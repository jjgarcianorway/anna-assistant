//! Arch Wiki and knowledge caching (v0.0.422).
//!
//! File-based cache for wiki pages and doc snippets:
//! - Cache path: /var/lib/anna/wiki/ or ~/.anna/wiki/
//! - TTL: 7 days
//! - Format: JSON with metadata

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::CACHE_TTL_SECS;

/// Cache entry for a wiki page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiCacheEntry {
    /// Topic/page name
    pub topic: String,
    /// Page content (text)
    pub content: String,
    /// When this was fetched (unix timestamp)
    pub fetched_at: u64,
    /// Source URL or path
    pub source: String,
    /// Content hash for change detection
    pub content_hash: u64,
}

impl WikiCacheEntry {
    /// Create a new cache entry
    pub fn new(topic: &str, content: &str, source: &str) -> Self {
        Self {
            topic: topic.to_string(),
            content: content.to_string(),
            fetched_at: current_timestamp(),
            source: source.to_string(),
            content_hash: simple_hash(content),
        }
    }

    /// Check if entry is expired
    pub fn is_expired(&self) -> bool {
        let now = current_timestamp();
        now.saturating_sub(self.fetched_at) > CACHE_TTL_SECS
    }

    /// Check if entry has valid content
    pub fn is_valid(&self) -> bool {
        !self.content.is_empty() && !self.is_expired()
    }

    /// Get age in hours
    pub fn age_hours(&self) -> u64 {
        let now = current_timestamp();
        now.saturating_sub(self.fetched_at) / 3600
    }
}

/// Wiki cache manager
pub struct WikiCache {
    /// Cache directory path
    cache_dir: PathBuf,
}

impl WikiCache {
    /// Create a new cache with default paths
    pub fn new() -> Self {
        Self {
            cache_dir: default_cache_dir(),
        }
    }

    /// Create cache with custom path
    pub fn with_path(path: PathBuf) -> Self {
        Self { cache_dir: path }
    }

    /// Get cached entry for a topic
    pub fn get(&self, topic: &str) -> Option<WikiCacheEntry> {
        let path = self.entry_path(topic);
        if !path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&path).ok()?;
        let entry: WikiCacheEntry = serde_json::from_str(&content).ok()?;

        if entry.is_expired() {
            // Clean up expired entry
            let _ = std::fs::remove_file(&path);
            return None;
        }

        Some(entry)
    }

    /// Store entry in cache
    pub fn put(&self, entry: &WikiCacheEntry) -> Result<(), String> {
        // Ensure cache directory exists
        std::fs::create_dir_all(&self.cache_dir)
            .map_err(|e| format!("Failed to create cache dir: {}", e))?;

        let path = self.entry_path(&entry.topic);
        let json = serde_json::to_string_pretty(entry)
            .map_err(|e| format!("Failed to serialize cache entry: {}", e))?;

        std::fs::write(&path, json).map_err(|e| format!("Failed to write cache entry: {}", e))?;

        Ok(())
    }

    /// Remove entry from cache
    pub fn remove(&self, topic: &str) -> bool {
        let path = self.entry_path(topic);
        std::fs::remove_file(path).is_ok()
    }

    /// Check if topic is cached and valid
    pub fn has_valid(&self, topic: &str) -> bool {
        self.get(topic).map(|e| e.is_valid()).unwrap_or(false)
    }

    /// List all cached topics
    pub fn list_topics(&self) -> Vec<String> {
        let mut topics = vec![];

        if let Ok(entries) = std::fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".json") {
                        topics.push(name.trim_end_matches(".json").to_string());
                    }
                }
            }
        }

        topics
    }

    /// Clean expired entries
    pub fn cleanup_expired(&self) -> usize {
        let mut cleaned = 0;

        if let Ok(entries) = std::fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Ok(cache_entry) = serde_json::from_str::<WikiCacheEntry>(&content) {
                        if cache_entry.is_expired() {
                            if std::fs::remove_file(entry.path()).is_ok() {
                                cleaned += 1;
                            }
                        }
                    }
                }
            }
        }

        cleaned
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let mut total = 0;
        let mut expired = 0;
        let mut total_size = 0;

        if let Ok(entries) = std::fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    total += 1;
                    total_size += metadata.len();

                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        if let Ok(cache_entry) = serde_json::from_str::<WikiCacheEntry>(&content) {
                            if cache_entry.is_expired() {
                                expired += 1;
                            }
                        }
                    }
                }
            }
        }

        CacheStats {
            total_entries: total,
            expired_entries: expired,
            total_size_bytes: total_size,
        }
    }

    /// Get path for a topic entry
    fn entry_path(&self, topic: &str) -> PathBuf {
        let safe_name = sanitize_filename(topic);
        self.cache_dir.join(format!("{}.json", safe_name))
    }
}

impl Default for WikiCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total number of entries
    pub total_entries: usize,
    /// Number of expired entries
    pub expired_entries: usize,
    /// Total size in bytes
    pub total_size_bytes: u64,
}

impl CacheStats {
    /// Get size in human-readable format
    pub fn size_human(&self) -> String {
        let kb = self.total_size_bytes as f64 / 1024.0;
        if kb < 1024.0 {
            format!("{:.1} KB", kb)
        } else {
            format!("{:.1} MB", kb / 1024.0)
        }
    }
}

/// Get default cache directory
fn default_cache_dir() -> PathBuf {
    // Try system path first
    let system_path = PathBuf::from("/var/lib/anna/wiki");
    if system_path.exists() || std::fs::create_dir_all(&system_path).is_ok() {
        return system_path;
    }

    // Fall back to user path
    if let Some(home) = dirs::home_dir() {
        return home.join(".anna/wiki");
    }

    // Last resort: temp dir
    std::env::temp_dir().join("anna-wiki-cache")
}

/// Sanitize topic name for use as filename
fn sanitize_filename(topic: &str) -> String {
    topic
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .to_lowercase()
}

/// Get current unix timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Simple string hash for change detection
fn simple_hash(s: &str) -> u64 {
    let mut hash: u64 = 5381;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry() {
        let entry = WikiCacheEntry::new("Systemd", "Content about systemd...", "archwiki:Systemd");
        assert!(!entry.is_expired());
        assert!(entry.is_valid());
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Systemd/Services"), "systemd_services");
        assert_eq!(sanitize_filename("vim: tips"), "vim__tips");
        assert_eq!(sanitize_filename("test-page_1"), "test-page_1");
    }

    #[test]
    fn test_simple_hash() {
        let h1 = simple_hash("hello");
        let h2 = simple_hash("hello");
        let h3 = simple_hash("world");
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_cache_stats() {
        let stats = CacheStats {
            total_entries: 10,
            expired_entries: 2,
            total_size_bytes: 1024 * 500,
        };
        assert!(stats.size_human().contains("KB"));
    }
}
