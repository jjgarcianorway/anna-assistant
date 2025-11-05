//! Advice display cache - ensures numbers in `apply` match `advise`
//!
//! When `advise` displays recommendations, it saves the exact order to a cache file.
//! When `apply` uses numbers, it reads from this cache to ensure perfect alignment.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Cache of displayed advice with their display numbers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdviceDisplayCache {
    /// List of advice IDs in display order (1-indexed)
    pub advice_ids: Vec<String>,
    /// Timestamp of when this cache was created
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl AdviceDisplayCache {
    /// Get cache file path
    fn cache_path() -> PathBuf {
        let cache_dir = std::env::var("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".cache")
            })
            .join("anna");

        cache_dir.join("advice_display_cache.json")
    }

    /// Save the current display order
    pub fn save(advice_ids: Vec<String>) -> Result<()> {
        let cache = Self {
            advice_ids,
            created_at: chrono::Utc::now(),
        };

        let cache_path = Self::cache_path();

        // Create parent directory if it doesn't exist
        if let Some(parent) = cache_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create cache directory")?;
        }

        let json = serde_json::to_string_pretty(&cache)?;
        std::fs::write(&cache_path, json).context("Failed to write cache file")?;

        Ok(())
    }

    /// Load the cached display order
    pub fn load() -> Result<Self> {
        let cache_path = Self::cache_path();
        let json = std::fs::read_to_string(&cache_path)
            .context("Failed to read cache file - run 'annactl advise' first")?;

        let cache: Self = serde_json::from_str(&json)
            .context("Failed to parse cache file")?;

        // Check if cache is too old (older than 1 hour)
        let age = chrono::Utc::now() - cache.created_at;
        if age.num_hours() > 1 {
            anyhow::bail!("Cache is too old ({}h) - run 'annactl advise' first", age.num_hours());
        }

        Ok(cache)
    }

    /// Get advice ID by display number (1-indexed)
    pub fn get_id_by_number(&self, number: usize) -> Option<&str> {
        if number < 1 || number > self.advice_ids.len() {
            None
        } else {
            Some(&self.advice_ids[number - 1])
        }
    }

    /// Get total number of cached items
    pub fn len(&self) -> usize {
        self.advice_ids.len()
    }

    /// Invalidate (delete) the cache
    /// Call this after applying advice to force regeneration on next advise
    pub fn invalidate() -> Result<()> {
        let cache_path = Self::cache_path();
        if cache_path.exists() {
            std::fs::remove_file(&cache_path)
                .context("Failed to delete cache file")?;
        }
        Ok(())
    }
}
