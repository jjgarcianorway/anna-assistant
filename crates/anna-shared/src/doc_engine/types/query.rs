//! Documentation query and result types

use super::snippet::DocSnippet;
use super::source::DocSourceKind;

/// Query for documentation
#[derive(Debug, Clone)]
pub struct DocQuery {
    /// Search query text
    pub query: String,
    /// Preferred sources (in order)
    pub preferred_sources: Vec<DocSourceKind>,
    /// Maximum results to return
    pub limit: usize,
    /// Specific name to search (e.g., "systemctl")
    pub name_filter: Option<String>,
    /// Minimum relevance score (0-100)
    pub min_relevance: u8,
}

impl DocQuery {
    /// Create a new query
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
            preferred_sources: vec![
                DocSourceKind::ArchWiki,
                DocSourceKind::ManPage,
                DocSourceKind::ToolHelp,
            ],
            limit: 5,
            name_filter: None,
            min_relevance: 0,
        }
    }

    /// Set preferred sources
    pub fn with_sources(mut self, sources: Vec<DocSourceKind>) -> Self {
        self.preferred_sources = sources;
        self
    }

    /// Set result limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Filter by name
    pub fn with_name(mut self, name: &str) -> Self {
        self.name_filter = Some(name.to_string());
        self
    }

    /// Set minimum relevance
    pub fn with_min_relevance(mut self, min: u8) -> Self {
        self.min_relevance = min;
        self
    }

    /// Query for man pages only
    pub fn man_page(command: &str) -> Self {
        Self::new(command)
            .with_sources(vec![DocSourceKind::ManPage])
            .with_name(command)
    }

    /// Query for Arch Wiki only
    pub fn arch_wiki(topic: &str) -> Self {
        Self::new(topic).with_sources(vec![DocSourceKind::ArchWiki])
    }

    /// Query for help output only
    pub fn tool_help(command: &str) -> Self {
        Self::new(command)
            .with_sources(vec![DocSourceKind::ToolHelp])
            .with_name(command)
    }
}

/// Result of a documentation query
#[derive(Debug, Clone, Default)]
pub struct DocResult {
    /// Matching snippets (sorted by relevance)
    pub snippets: Vec<DocSnippet>,
    /// Whether results came from cache
    pub from_cache: bool,
    /// Query time in milliseconds
    pub query_time_ms: u64,
    /// Sources that were searched
    pub sources_searched: Vec<DocSourceKind>,
}

impl DocResult {
    /// Create empty result
    pub fn empty() -> Self {
        Self::default()
    }

    /// Check if result has any snippets
    pub fn is_empty(&self) -> bool {
        self.snippets.is_empty()
    }

    /// Get best snippet (highest relevance)
    pub fn best(&self) -> Option<&DocSnippet> {
        self.snippets.first()
    }

    /// Get all citations
    pub fn citations(&self) -> Vec<String> {
        self.snippets.iter().map(|s| s.citation()).collect()
    }

    /// Merge with another result
    pub fn merge(mut self, other: DocResult) -> Self {
        self.snippets.extend(other.snippets);
        self.from_cache = self.from_cache || other.from_cache;
        self.query_time_ms += other.query_time_ms;

        for source in other.sources_searched {
            if !self.sources_searched.contains(&source) {
                self.sources_searched.push(source);
            }
        }

        // Re-sort by relevance
        self.snippets.sort_by(|a, b| b.relevance.cmp(&a.relevance));

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_query_builders() {
        let q = DocQuery::man_page("systemctl");
        assert_eq!(q.preferred_sources, vec![DocSourceKind::ManPage]);
        assert_eq!(q.name_filter, Some("systemctl".to_string()));

        let q = DocQuery::arch_wiki("solid state drive");
        assert_eq!(q.preferred_sources, vec![DocSourceKind::ArchWiki]);
    }
}
