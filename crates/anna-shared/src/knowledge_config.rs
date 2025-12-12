//! Knowledge Configuration (v0.0.414).
//!
//! Configuration for Anna's knowledge sources.
//! Loaded from ~/.anna/config.toml or /etc/anna/config.toml.
//!
//! Example config:
//! ```toml
//! [knowledge]
//! arch_wiki_enabled = true
//! arch_wiki_path = "/var/lib/anna/arch_wiki"
//! doc_cache_path = "/var/lib/anna/docs"
//! preferred_sources = ["probe_output", "man_page", "cli_help", "arch_wiki_page"]
//! ```

use crate::knowledge_query::KnowledgeSourceKind;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Default Arch Wiki cache path
pub const DEFAULT_WIKI_PATH: &str = "/var/lib/anna/arch_wiki";

/// User wiki path
pub const USER_WIKI_PATH: &str = "~/.anna/wiki-cache";

/// Default doc cache path
pub const DEFAULT_DOC_CACHE: &str = "/var/lib/anna/docs";

/// Man page cache path
pub const MAN_CACHE_PATH: &str = "/var/lib/anna/man-cache";

/// Knowledge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeConfig {
    /// Whether Arch Wiki lookup is enabled
    #[serde(default = "default_wiki_enabled")]
    pub arch_wiki_enabled: bool,

    /// Path to local Arch Wiki cache
    #[serde(default = "default_wiki_path")]
    pub arch_wiki_path: String,

    /// Path to doc cache directory
    #[serde(default = "default_doc_cache")]
    pub doc_cache_path: String,

    /// Preferred source types in priority order
    #[serde(default = "default_preferred_sources")]
    pub preferred_sources: Vec<String>,

    /// Whether to cache man pages on disk
    #[serde(default = "default_true")]
    pub cache_man_pages: bool,

    /// Whether to cache --help output on disk
    #[serde(default = "default_true")]
    pub cache_help_output: bool,

    /// Max age for cached docs (hours)
    #[serde(default = "default_cache_hours")]
    pub cache_max_hours: u32,

    /// Enable learning from successful tickets
    #[serde(default = "default_true")]
    pub learning_enabled: bool,

    /// Minimum confidence for auto-learning recipes
    #[serde(default = "default_learn_confidence")]
    pub learning_min_confidence: u8,
}

fn default_wiki_enabled() -> bool {
    true
}
fn default_wiki_path() -> String {
    DEFAULT_WIKI_PATH.to_string()
}
fn default_doc_cache() -> String {
    DEFAULT_DOC_CACHE.to_string()
}
fn default_true() -> bool {
    true
}
fn default_cache_hours() -> u32 {
    168
} // 1 week
fn default_learn_confidence() -> u8 {
    80
}

fn default_preferred_sources() -> Vec<String> {
    vec![
        "probe_output".to_string(),
        "man_page".to_string(),
        "cli_help".to_string(),
        "arch_wiki_page".to_string(),
    ]
}

impl Default for KnowledgeConfig {
    fn default() -> Self {
        Self {
            arch_wiki_enabled: default_wiki_enabled(),
            arch_wiki_path: default_wiki_path(),
            doc_cache_path: default_doc_cache(),
            preferred_sources: default_preferred_sources(),
            cache_man_pages: default_true(),
            cache_help_output: default_true(),
            cache_max_hours: default_cache_hours(),
            learning_enabled: default_true(),
            learning_min_confidence: default_learn_confidence(),
        }
    }
}

impl KnowledgeConfig {
    /// Load from config file or use defaults
    pub fn load() -> Self {
        let paths = [
            dirs::config_dir().map(|p| p.join("anna/config.toml")),
            Some(PathBuf::from("/etc/anna/config.toml")),
            dirs::home_dir().map(|p| p.join(".anna/config.toml")),
        ];

        for path_opt in paths.iter().flatten() {
            if path_opt.exists() {
                if let Ok(content) = std::fs::read_to_string(path_opt) {
                    if let Ok(full_config) = toml::from_str::<toml::Table>(&content) {
                        if let Some(knowledge) = full_config.get("knowledge") {
                            if let Ok(config) = knowledge.clone().try_into::<KnowledgeConfig>() {
                                return config;
                            }
                        }
                    }
                }
            }
        }

        Self::default()
    }

    /// Get the wiki path, expanding ~ to home directory
    pub fn wiki_path(&self) -> PathBuf {
        expand_path(&self.arch_wiki_path)
    }

    /// Get the doc cache path
    pub fn doc_cache_path(&self) -> PathBuf {
        expand_path(&self.doc_cache_path)
    }

    /// Check if wiki is available
    pub fn wiki_available(&self) -> bool {
        self.arch_wiki_enabled && self.wiki_path().exists()
    }

    /// Get preferred sources as enum
    pub fn preferred_source_kinds(&self) -> Vec<KnowledgeSourceKind> {
        self.preferred_sources
            .iter()
            .filter_map(|s| parse_source_kind(s))
            .collect()
    }

    /// Check if a source kind is enabled
    pub fn is_source_enabled(&self, kind: KnowledgeSourceKind) -> bool {
        match kind {
            KnowledgeSourceKind::ArchWikiPage | KnowledgeSourceKind::ArchWikiSection => {
                self.arch_wiki_enabled
            }
            _ => true,
        }
    }
}

/// Parse source kind from string
fn parse_source_kind(s: &str) -> Option<KnowledgeSourceKind> {
    match s.to_lowercase().as_str() {
        "man_page" | "man" => Some(KnowledgeSourceKind::ManPage),
        "cli_help" | "help" => Some(KnowledgeSourceKind::CliHelp),
        "arch_wiki_page" | "wiki" => Some(KnowledgeSourceKind::ArchWikiPage),
        "arch_wiki_section" => Some(KnowledgeSourceKind::ArchWikiSection),
        "local_doc_file" | "doc" => Some(KnowledgeSourceKind::LocalDocFile),
        "probe_output" | "probe" => Some(KnowledgeSourceKind::ProbeOutput),
        "config_file" | "config" => Some(KnowledgeSourceKind::ConfigFile),
        "log_excerpt" | "log" => Some(KnowledgeSourceKind::LogExcerpt),
        "built_in" | "builtin" => Some(KnowledgeSourceKind::BuiltIn),
        "learned_recipe" | "recipe" => Some(KnowledgeSourceKind::LearnedRecipe),
        _ => None,
    }
}

/// Expand ~ to home directory
fn expand_path(path: &str) -> PathBuf {
    if path.starts_with("~/") || path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

/// Wiki cache statistics
#[derive(Debug, Clone, Default)]
pub struct WikiStats {
    /// Cache path
    pub path: String,
    /// Number of cached pages
    pub page_count: usize,
    /// Total size in bytes
    pub total_bytes: u64,
    /// Whether cache is available
    pub available: bool,
}

impl WikiStats {
    /// Gather wiki cache statistics
    pub fn gather(config: &KnowledgeConfig) -> Self {
        let path = config.wiki_path();
        let mut stats = WikiStats {
            path: path.display().to_string(),
            available: config.wiki_available(),
            ..Default::default()
        };

        if stats.available {
            if let Ok(entries) = std::fs::read_dir(&path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    if entry.path().is_file() {
                        stats.page_count += 1;
                        if let Ok(meta) = entry.metadata() {
                            stats.total_bytes += meta.len();
                        }
                    }
                }
            }
        }

        stats
    }
}

/// Documentation cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocCacheEntry {
    /// Document ID
    pub doc_id: String,
    /// Cached content
    pub content: String,
    /// Cache timestamp (Unix secs)
    pub cached_at: u64,
    /// Source command or path
    pub source: String,
}

impl DocCacheEntry {
    /// Create new cache entry
    pub fn new(doc_id: &str, content: &str, source: &str) -> Self {
        Self {
            doc_id: doc_id.to_string(),
            content: content.to_string(),
            cached_at: current_secs(),
            source: source.to_string(),
        }
    }

    /// Check if cache entry is stale
    pub fn is_stale(&self, max_hours: u32) -> bool {
        let max_secs = max_hours as u64 * 3600;
        current_secs().saturating_sub(self.cached_at) > max_secs
    }
}

fn current_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = KnowledgeConfig::default();
        assert!(config.arch_wiki_enabled);
        assert!(config.cache_man_pages);
        assert_eq!(config.learning_min_confidence, 80);
    }

    #[test]
    fn test_parse_source_kind() {
        assert_eq!(
            parse_source_kind("man_page"),
            Some(KnowledgeSourceKind::ManPage)
        );
        assert_eq!(parse_source_kind("man"), Some(KnowledgeSourceKind::ManPage));
        assert_eq!(
            parse_source_kind("wiki"),
            Some(KnowledgeSourceKind::ArchWikiPage)
        );
        assert_eq!(parse_source_kind("unknown"), None);
    }

    #[test]
    fn test_expand_path() {
        let expanded = expand_path("/var/lib/anna");
        assert_eq!(expanded, PathBuf::from("/var/lib/anna"));

        // Home expansion would depend on system, but path should be valid
        let home_path = expand_path("~/.anna");
        assert!(!home_path.to_string_lossy().contains("~"));
    }

    #[test]
    fn test_doc_cache_entry_stale() {
        let entry = DocCacheEntry::new("test", "content", "source");
        assert!(!entry.is_stale(1)); // Not stale within 1 hour

        let old_entry = DocCacheEntry {
            doc_id: "old".to_string(),
            content: "old content".to_string(),
            cached_at: 0, // Unix epoch
            source: "test".to_string(),
        };
        assert!(old_entry.is_stale(1)); // Definitely stale
    }
}
