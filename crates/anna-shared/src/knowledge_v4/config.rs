//! Knowledge Engine configuration (v0.0.424).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::{DEFAULT_MAN_PATHS, DEFAULT_DOC_PATHS, DEFAULT_WIKI_PATH, MAX_SNIPPET_CHARS, MAX_RESULTS_PER_QUERY};

/// Configuration for the KnowledgeEngine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeConfig {
    /// Whether to allow web lookups (default: false)
    /// When enabled, only official sources like Arch Wiki are queried
    pub allow_web_lookup: bool,

    /// Paths to search for man pages
    pub man_paths: Vec<PathBuf>,

    /// Paths to search for local documentation
    pub doc_paths: Vec<PathBuf>,

    /// Path to offline Arch Wiki snapshot (optional)
    pub wiki_path: Option<PathBuf>,

    /// Maximum characters per snippet excerpt
    pub max_snippet_chars: usize,

    /// Maximum results per query
    pub max_results_per_query: usize,

    /// Whether to log queries (for debugging)
    pub log_queries: bool,

    /// Timeout for command execution in ms
    pub command_timeout_ms: u64,
}

impl Default for KnowledgeConfig {
    fn default() -> Self {
        Self {
            allow_web_lookup: false,
            man_paths: DEFAULT_MAN_PATHS.iter().map(PathBuf::from).collect(),
            doc_paths: DEFAULT_DOC_PATHS.iter().map(PathBuf::from).collect(),
            wiki_path: check_wiki_path(),
            max_snippet_chars: MAX_SNIPPET_CHARS,
            max_results_per_query: MAX_RESULTS_PER_QUERY,
            log_queries: false,
            command_timeout_ms: super::COMMAND_TIMEOUT_MS,
        }
    }
}

impl KnowledgeConfig {
    /// Create config with custom wiki path
    pub fn with_wiki_path(mut self, path: PathBuf) -> Self {
        self.wiki_path = Some(path);
        self
    }

    /// Enable web lookup
    pub fn with_web_lookup(mut self, enabled: bool) -> Self {
        self.allow_web_lookup = enabled;
        self
    }

    /// Enable query logging
    pub fn with_logging(mut self, enabled: bool) -> Self {
        self.log_queries = enabled;
        self
    }

    /// Add a man path
    pub fn add_man_path(mut self, path: PathBuf) -> Self {
        self.man_paths.push(path);
        self
    }

    /// Add a doc path
    pub fn add_doc_path(mut self, path: PathBuf) -> Self {
        self.doc_paths.push(path);
        self
    }

    /// Set max snippet chars
    pub fn with_max_snippet_chars(mut self, max: usize) -> Self {
        self.max_snippet_chars = max;
        self
    }

    /// Check if wiki is available
    pub fn wiki_available(&self) -> bool {
        self.wiki_path
            .as_ref()
            .map(|p| p.exists())
            .unwrap_or(false)
    }

    /// Get effective man paths (only existing ones)
    pub fn effective_man_paths(&self) -> Vec<&PathBuf> {
        self.man_paths.iter().filter(|p| p.exists()).collect()
    }

    /// Get effective doc paths (only existing ones)
    pub fn effective_doc_paths(&self) -> Vec<&PathBuf> {
        self.doc_paths.iter().filter(|p| p.exists()).collect()
    }
}

/// Check if default wiki path exists
fn check_wiki_path() -> Option<PathBuf> {
    let path = PathBuf::from(DEFAULT_WIKI_PATH);
    if path.exists() {
        Some(path)
    } else {
        // Try user home
        dirs::home_dir()
            .map(|h| h.join(".anna/wiki/arch"))
            .filter(|p| p.exists())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = KnowledgeConfig::default();
        assert!(!config.allow_web_lookup);
        assert!(!config.man_paths.is_empty());
        assert!(!config.doc_paths.is_empty());
        assert!(config.max_snippet_chars > 0);
    }

    #[test]
    fn test_config_builder() {
        let config = KnowledgeConfig::default()
            .with_web_lookup(true)
            .with_logging(true)
            .with_max_snippet_chars(2000);

        assert!(config.allow_web_lookup);
        assert!(config.log_queries);
        assert_eq!(config.max_snippet_chars, 2000);
    }
}
