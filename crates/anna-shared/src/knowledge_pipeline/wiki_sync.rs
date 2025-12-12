//! Arch Wiki offline sync helper (v0.0.432).
//!
//! Syncs Arch Wiki articles for offline use.
//! Respects rate limits and caches content locally.

use super::WIKI_CACHE_DIR;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Wiki sync configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiSyncConfig {
    /// Base path for wiki cache.
    pub cache_path: PathBuf,
    /// Maximum age before re-sync (seconds).
    pub max_age_secs: u64,
    /// Rate limit: minimum seconds between requests.
    pub rate_limit_secs: u64,
    /// Articles to keep synced.
    pub articles: Vec<String>,
    /// Maximum article size to cache (bytes).
    pub max_article_size: usize,
}

impl Default for WikiSyncConfig {
    fn default() -> Self {
        Self {
            cache_path: PathBuf::from(WIKI_CACHE_DIR),
            max_age_secs: 7 * 24 * 60 * 60, // 7 days
            rate_limit_secs: 2,             // 2 seconds between requests
            articles: default_articles(),
            max_article_size: 1024 * 1024, // 1 MB
        }
    }
}

/// Default list of important Arch Wiki articles.
fn default_articles() -> Vec<String> {
    vec![
        "Systemd".to_string(),
        "Systemd/User".to_string(),
        "Pacman".to_string(),
        "Pacman/Tips_and_tricks".to_string(),
        "Installation_guide".to_string(),
        "General_recommendations".to_string(),
        "System_maintenance".to_string(),
        "Boot_loader".to_string(),
        "GRUB".to_string(),
        "Kernel".to_string(),
        "Kernel_parameters".to_string(),
        "Improving_performance".to_string(),
        "Improving_performance/Boot_process".to_string(),
        "Power_management".to_string(),
        "Solid_state_drive".to_string(),
        "Network_configuration".to_string(),
        "Wireless_network_configuration".to_string(),
        "NetworkManager".to_string(),
        "Bluetooth".to_string(),
        "PulseAudio".to_string(),
        "PipeWire".to_string(),
        "Xorg".to_string(),
        "Wayland".to_string(),
        "GNOME".to_string(),
        "KDE".to_string(),
        "Btrfs".to_string(),
        "Ext4".to_string(),
        "LVM".to_string(),
        "RAID".to_string(),
        "Security".to_string(),
        "Users_and_groups".to_string(),
    ]
}

/// Sync status for an article.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    /// Article name.
    pub article: String,
    /// Whether currently cached.
    pub cached: bool,
    /// Last sync timestamp (unix).
    pub last_sync: Option<u64>,
    /// Whether needs update.
    pub needs_update: bool,
    /// Size in bytes.
    pub size_bytes: Option<u64>,
    /// Error if last sync failed.
    pub last_error: Option<String>,
}

/// Wiki syncer.
pub struct WikiSyncer {
    config: WikiSyncConfig,
    /// Last request timestamp for rate limiting.
    last_request: Option<SystemTime>,
}

impl WikiSyncer {
    /// Create a new syncer with default config.
    pub fn new(base_path: &Path) -> Self {
        let mut config = WikiSyncConfig::default();
        config.cache_path = base_path.join(WIKI_CACHE_DIR);
        Self {
            config,
            last_request: None,
        }
    }

    /// Create with custom config.
    pub fn with_config(config: WikiSyncConfig) -> Self {
        Self {
            config,
            last_request: None,
        }
    }

    /// Ensure cache directory exists.
    fn ensure_cache_dir(&self) -> Result<(), String> {
        fs::create_dir_all(&self.config.cache_path).map_err(|e| e.to_string())
    }

    /// Get path for an article.
    fn article_path(&self, article: &str) -> PathBuf {
        let filename = sanitize_article_name(article);
        self.config.cache_path.join(format!("{}.txt", filename))
    }

    /// Get metadata path for an article.
    fn metadata_path(&self, article: &str) -> PathBuf {
        let filename = sanitize_article_name(article);
        self.config
            .cache_path
            .join(format!("{}.meta.json", filename))
    }

    /// Check status of all configured articles.
    pub fn check_status(&self) -> Vec<SyncStatus> {
        self.config
            .articles
            .iter()
            .map(|article| self.article_status(article))
            .collect()
    }

    /// Get status for a specific article.
    pub fn article_status(&self, article: &str) -> SyncStatus {
        let path = self.article_path(article);
        let meta_path = self.metadata_path(article);

        let cached = path.exists();
        let (last_sync, size_bytes, last_error) = if meta_path.exists() {
            if let Ok(content) = fs::read_to_string(&meta_path) {
                if let Ok(meta) = serde_json::from_str::<ArticleMeta>(&content) {
                    (Some(meta.synced_at), Some(meta.size_bytes), meta.last_error)
                } else {
                    (None, None, None)
                }
            } else {
                (None, None, None)
            }
        } else {
            (None, None, None)
        };

        let needs_update = if let Some(sync_time) = last_sync {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            now - sync_time > self.config.max_age_secs
        } else {
            true
        };

        SyncStatus {
            article: article.to_string(),
            cached,
            last_sync,
            needs_update,
            size_bytes,
            last_error,
        }
    }

    /// Sync a single article (placeholder - actual HTTP fetch would go here).
    pub fn sync_article(&mut self, article: &str) -> Result<SyncStatus, String> {
        self.ensure_cache_dir()?;

        // Rate limiting
        if let Some(last) = self.last_request {
            let elapsed = last.elapsed().unwrap_or(Duration::ZERO);
            if elapsed < Duration::from_secs(self.config.rate_limit_secs) {
                let wait = Duration::from_secs(self.config.rate_limit_secs) - elapsed;
                std::thread::sleep(wait);
            }
        }
        self.last_request = Some(SystemTime::now());

        // Note: Actual HTTP fetch disabled by default
        // This is a placeholder that creates an empty cached file
        // Real implementation would use reqwest to fetch from wiki.archlinux.org

        let path = self.article_path(article);
        let meta_path = self.metadata_path(article);

        // For now, just create a placeholder
        let content = format!(
            "# {}\n\nThis is a placeholder for the Arch Wiki article.\n\
             To fetch real content, run: anna wiki-sync --fetch\n\n\
             Article URL: https://wiki.archlinux.org/title/{}\n",
            article,
            article.replace(' ', "_")
        );

        fs::write(&path, &content).map_err(|e| e.to_string())?;

        let meta = ArticleMeta {
            article: article.to_string(),
            synced_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            size_bytes: content.len() as u64,
            url: format!(
                "https://wiki.archlinux.org/title/{}",
                article.replace(' ', "_")
            ),
            last_error: None,
        };

        let meta_json = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
        fs::write(&meta_path, meta_json).map_err(|e| e.to_string())?;

        Ok(self.article_status(article))
    }

    /// Sync all configured articles that need updates.
    pub fn sync_outdated(&mut self) -> Vec<Result<SyncStatus, String>> {
        let outdated: Vec<String> = self
            .check_status()
            .into_iter()
            .filter(|s| s.needs_update)
            .map(|s| s.article)
            .collect();

        outdated
            .into_iter()
            .map(|article| self.sync_article(&article))
            .collect()
    }

    /// Add an article to the sync list.
    pub fn add_article(&mut self, article: &str) {
        if !self.config.articles.contains(&article.to_string()) {
            self.config.articles.push(article.to_string());
        }
    }

    /// Remove an article from the sync list.
    pub fn remove_article(&mut self, article: &str) {
        self.config.articles.retain(|a| a != article);
    }

    /// Get cache statistics.
    pub fn stats(&self) -> CacheStats {
        let statuses = self.check_status();
        let total = statuses.len();
        let cached = statuses.iter().filter(|s| s.cached).count();
        let outdated = statuses.iter().filter(|s| s.needs_update).count();
        let total_size: u64 = statuses.iter().filter_map(|s| s.size_bytes).sum();

        CacheStats {
            total_articles: total,
            cached_articles: cached,
            outdated_articles: outdated,
            total_size_bytes: total_size,
            cache_path: self.config.cache_path.clone(),
        }
    }

    /// Read cached article content.
    pub fn read_article(&self, article: &str) -> Option<String> {
        let path = self.article_path(article);
        fs::read_to_string(path).ok()
    }

    /// Search cached articles for a term.
    pub fn search(&self, term: &str) -> Vec<SearchResult> {
        let term_lower = term.to_lowercase();
        let mut results = Vec::new();

        for article in &self.config.articles {
            if let Some(content) = self.read_article(article) {
                let content_lower = content.to_lowercase();
                if content_lower.contains(&term_lower) {
                    // Count matches
                    let matches = content_lower.matches(&term_lower).count();
                    results.push(SearchResult {
                        article: article.clone(),
                        matches,
                        excerpt: extract_excerpt(&content, term),
                    });
                }
            }
        }

        // Sort by match count
        results.sort_by(|a, b| b.matches.cmp(&a.matches));
        results
    }
}

/// Article metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArticleMeta {
    article: String,
    synced_at: u64,
    size_bytes: u64,
    url: String,
    last_error: Option<String>,
}

/// Cache statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_articles: usize,
    pub cached_articles: usize,
    pub outdated_articles: usize,
    pub total_size_bytes: u64,
    pub cache_path: PathBuf,
}

/// Search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub article: String,
    pub matches: usize,
    pub excerpt: String,
}

/// Sanitize article name for use as filename.
fn sanitize_article_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Extract an excerpt around a search term.
fn extract_excerpt(content: &str, term: &str) -> String {
    let content_lower = content.to_lowercase();
    let term_lower = term.to_lowercase();

    if let Some(pos) = content_lower.find(&term_lower) {
        let start = pos.saturating_sub(50);
        let end = (pos + term.len() + 50).min(content.len());
        let excerpt = &content[start..end];
        format!("...{}...", excerpt.trim())
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_articles() {
        let articles = default_articles();
        assert!(articles.contains(&"Systemd".to_string()));
        assert!(articles.contains(&"Pacman".to_string()));
    }

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_article_name("System/User"), "System_User");
        assert_eq!(sanitize_article_name("Tips and tricks"), "Tips_and_tricks");
    }

    #[test]
    fn test_cache_stats() {
        let path = format!("/tmp/anna_wiki_test_{}", std::process::id());
        let syncer = WikiSyncer::new(Path::new(&path));
        let stats = syncer.stats();
        assert!(stats.total_articles > 0);
        let _ = fs::remove_dir_all(&path);
    }
}
