//! Knowledge aggregator - Unified retrieval from multiple sources.
//!
//! Aggregates knowledge from:
//! - Wiki articles
//! - Man pages
//! - Command help (--help)
//! - Learned recipes
//! - Agent memories

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A piece of knowledge from any source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeItem {
    /// Source of the knowledge
    pub source: KnowledgeSource,
    /// Title or command name
    pub title: String,
    /// Content or summary
    pub content: String,
    /// Relevance score (0.0-1.0)
    pub relevance: f32,
    /// Keywords for matching
    pub keywords: Vec<String>,
}

/// Source of knowledge.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum KnowledgeSource {
    Wiki,
    ManPage,
    CommandHelp,
    Recipe,
    AgentMemory,
    SystemInfo,
}

impl KnowledgeSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Wiki => "wiki",
            Self::ManPage => "man",
            Self::CommandHelp => "help",
            Self::Recipe => "recipe",
            Self::AgentMemory => "memory",
            Self::SystemInfo => "system",
        }
    }
}

/// Knowledge aggregator configuration.
#[derive(Debug, Clone)]
pub struct AggregatorConfig {
    /// Wiki articles directory
    pub wiki_dir: PathBuf,
    /// Man pages cache directory
    pub man_dir: PathBuf,
    /// Command help cache directory
    pub help_dir: PathBuf,
    /// Maximum results per source
    pub max_per_source: usize,
}

impl Default for AggregatorConfig {
    fn default() -> Self {
        Self {
            wiki_dir: PathBuf::from("/var/lib/anna/wiki"),
            man_dir: PathBuf::from("/var/lib/anna/docs/man"),
            help_dir: PathBuf::from("/var/lib/anna/docs/help"),
            max_per_source: 3,
        }
    }
}

/// Knowledge aggregator for unified retrieval.
pub struct KnowledgeAggregator {
    config: AggregatorConfig,
}

impl KnowledgeAggregator {
    pub fn new() -> Self {
        Self::with_config(AggregatorConfig::default())
    }

    pub fn with_config(config: AggregatorConfig) -> Self {
        Self { config }
    }

    /// Search all knowledge sources for a query.
    pub fn search(&self, query: &str, limit: usize) -> Vec<KnowledgeItem> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();
        let keywords: Vec<&str> = query_lower.split_whitespace().collect();

        // Search each source
        results.extend(self.search_wiki(&keywords));
        results.extend(self.search_man_pages(&keywords));
        results.extend(self.search_help_cache(&keywords));

        // Sort by relevance
        results.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));

        // Limit results
        results.truncate(limit);
        results
    }

    /// Search wiki articles.
    fn search_wiki(&self, keywords: &[&str]) -> Vec<KnowledgeItem> {
        let mut results = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&self.config.wiki_dir) {
            for entry in entries.filter_map(|e| e.ok()).take(100) {
                let path = entry.path();
                if path.extension().map(|e| e == "md" || e == "txt").unwrap_or(false) {
                    if let Some(item) = self.score_file(&path, keywords, KnowledgeSource::Wiki) {
                        if item.relevance > 0.2 {
                            results.push(item);
                        }
                    }
                }
            }
        }

        results.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(self.config.max_per_source);
        results
    }

    /// Search man page cache.
    fn search_man_pages(&self, keywords: &[&str]) -> Vec<KnowledgeItem> {
        let mut results = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&self.config.man_dir) {
            for entry in entries.filter_map(|e| e.ok()).take(50) {
                let path = entry.path();
                if let Some(item) = self.score_file(&path, keywords, KnowledgeSource::ManPage) {
                    if item.relevance > 0.2 {
                        results.push(item);
                    }
                }
            }
        }

        results.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(self.config.max_per_source);
        results
    }

    /// Search command help cache.
    fn search_help_cache(&self, keywords: &[&str]) -> Vec<KnowledgeItem> {
        let mut results = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&self.config.help_dir) {
            for entry in entries.filter_map(|e| e.ok()).take(50) {
                let path = entry.path();
                if let Some(item) = self.score_file(&path, keywords, KnowledgeSource::CommandHelp) {
                    if item.relevance > 0.2 {
                        results.push(item);
                    }
                }
            }
        }

        results.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(self.config.max_per_source);
        results
    }

    /// Score a file based on keyword matches.
    fn score_file(&self, path: &PathBuf, keywords: &[&str], source: KnowledgeSource) -> Option<KnowledgeItem> {
        let filename = path.file_stem()?.to_string_lossy().to_lowercase();
        let content = std::fs::read_to_string(path).ok()?;
        let content_lower = content.to_lowercase();

        // Score based on filename and content matches
        let mut score = 0.0;
        let mut matched_keywords = Vec::new();

        for keyword in keywords {
            if filename.contains(keyword) {
                score += 0.4;
                matched_keywords.push(keyword.to_string());
            }
            if content_lower.contains(keyword) {
                score += 0.2;
                if !matched_keywords.contains(&keyword.to_string()) {
                    matched_keywords.push(keyword.to_string());
                }
            }
        }

        // Normalize score
        let relevance = (score / keywords.len() as f32).min(1.0);

        // Extract summary (first 200 chars)
        let summary: String = content.lines()
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
            .take(3)
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .take(200)
            .collect();

        Some(KnowledgeItem {
            source,
            title: filename,
            content: summary,
            relevance,
            keywords: matched_keywords,
        })
    }

    /// Get knowledge item by exact title.
    pub fn get_by_title(&self, title: &str, source: KnowledgeSource) -> Option<KnowledgeItem> {
        let dir = match source {
            KnowledgeSource::Wiki => &self.config.wiki_dir,
            KnowledgeSource::ManPage => &self.config.man_dir,
            KnowledgeSource::CommandHelp => &self.config.help_dir,
            _ => return None,
        };

        // Try common extensions
        for ext in ["", ".md", ".txt"] {
            let path = dir.join(format!("{}{}", title, ext));
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    return Some(KnowledgeItem {
                        source,
                        title: title.to_string(),
                        content,
                        relevance: 1.0,
                        keywords: vec![title.to_string()],
                    });
                }
            }
        }

        None
    }
}

impl Default for KnowledgeAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_source_str() {
        assert_eq!(KnowledgeSource::Wiki.as_str(), "wiki");
        assert_eq!(KnowledgeSource::ManPage.as_str(), "man");
    }

    #[test]
    fn test_aggregator_creation() {
        let agg = KnowledgeAggregator::new();
        assert_eq!(agg.config.max_per_source, 3);
    }
}
