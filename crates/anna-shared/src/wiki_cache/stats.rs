//! Cache statistics and reporting.

use serde::{Deserialize, Serialize};

use super::index::WikiCacheIndex;
use super::utils::now_timestamp;

/// Cache statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Number of cached pages
    pub page_count: usize,
    /// Total size in bytes
    pub total_bytes: usize,
    /// Number of stale pages
    pub stale_count: usize,
    /// Oldest page age in days
    pub oldest_days: u64,
    /// Average page age in days
    pub avg_age_days: u64,
}

impl CacheStats {
    /// Format as display string
    pub fn display(&self) -> String {
        let size_str = if self.total_bytes > 1024 * 1024 {
            format!("{:.1}MB", self.total_bytes as f64 / (1024.0 * 1024.0))
        } else if self.total_bytes > 1024 {
            format!("{:.1}KB", self.total_bytes as f64 / 1024.0)
        } else {
            format!("{}B", self.total_bytes)
        };

        format!(
            "pages: {}, size: {}, stale: {}, oldest: {}d, avg: {}d",
            self.page_count, size_str, self.stale_count, self.oldest_days, self.avg_age_days
        )
    }
}

/// Get cache statistics
pub fn get_cache_stats(index: &WikiCacheIndex) -> CacheStats {
    let now = now_timestamp();
    let mut oldest: u64 = 0;
    let mut total_age: u64 = 0;

    for entry in index.entries.values() {
        let age = now.saturating_sub(entry.cached_at);
        total_age += age;
        if age > oldest {
            oldest = age;
        }
    }

    let avg_age = if index.entries.is_empty() {
        0
    } else {
        total_age / index.entries.len() as u64
    };

    CacheStats {
        page_count: index.count(),
        total_bytes: index.total_size(),
        stale_count: index.stale_entries().len(),
        oldest_days: oldest / 86400,
        avg_age_days: avg_age / 86400,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_stats_display() {
        let stats = CacheStats {
            page_count: 50,
            total_bytes: 1024 * 1024 + 512 * 1024,
            stale_count: 5,
            oldest_days: 45,
            avg_age_days: 15,
        };
        let output = stats.display();
        assert!(output.contains("50"));
        assert!(output.contains("MB"));
        assert!(output.contains("stale"));
    }
}
