//! Query types and utilities (v0.0.429).
//!
//! Statistics and helper functions for doc queries.

use super::super::{DocSnippet, DocSourceKind};

/// Score a snippet against query words
pub fn score_snippet(snippet: &DocSnippet, query_words: &[String]) -> u8 {
    let mut score: u32 = 0;

    for word in query_words {
        // Name match
        if snippet.name.to_lowercase().contains(word) {
            score += 30;
        }

        // Summary match
        if snippet.summary.to_lowercase().contains(word) {
            score += 20;
        }

        // Content match
        if snippet.content.to_lowercase().contains(word) {
            score += 10;
        }

        // Keyword match
        if snippet.keywords.iter().any(|k| k.contains(word)) {
            score += 15;
        }
    }

    // Source bonus
    score += match snippet.source {
        DocSourceKind::ArchWiki => 10,
        DocSourceKind::ManPage => 8,
        DocSourceKind::ToolHelp => 5,
        DocSourceKind::LocalDoc => 3,
    };

    (score.min(100)) as u8
}

/// Index statistics
#[derive(Debug, Clone, Default)]
pub struct IndexStats {
    pub total: usize,
    pub man_pages: usize,
    pub wiki_pages: usize,
    pub help_outputs: usize,
    pub local_docs: usize,
}

/// Refresh statistics
#[derive(Debug, Clone, Default)]
pub struct RefreshStats {
    pub man_pages: usize,
    pub wiki_pages: usize,
    pub help_outputs: usize,
    pub local_docs: usize,
    pub total_snippets: usize,
    pub duration_ms: u64,
}

/// Engine runtime statistics
#[derive(Debug, Clone, Default)]
pub struct DocEngineStats {
    pub queries: usize,
    pub cache_hits: usize,
    pub fetches: usize,
    pub total_snippets: usize,
    pub last_refresh_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_snippet() {
        let snippet = DocSnippet::new(
            DocSourceKind::ManPage,
            "systemctl",
            Some("1"),
            "Control the systemd system and service manager",
            "systemctl is used to control systemd and services.",
        );

        let score = score_snippet(&snippet, &["systemctl".to_string()]);
        assert!(score > 50); // Should score high

        let score = score_snippet(&snippet, &["unknown".to_string()]);
        assert!(score < 20); // Should score low
    }
}
