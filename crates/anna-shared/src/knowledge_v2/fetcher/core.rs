//! Knowledge fetcher core implementation.
//!
//! Orchestrates knowledge fetching from multiple sources:
//! 1. Man pages
//! 2. Help output
//! 3. Arch Wiki (cache)
//! 4. Local docs

use std::collections::HashSet;

use crate::knowledge_v2::cache::WikiCache;
use crate::knowledge_v2::policy::get_research_policy;
use crate::knowledge_v2::snippet::{KnowledgeSnippet, KnowledgeSource};
use crate::knowledge_v2::sources::{
    fetch_arch_wiki, fetch_help_output, fetch_local_doc, fetch_man_page, fetch_pacman_info,
};
use crate::knowledge_v2::{FETCH_TIMEOUT_MS, MAX_SNIPPETS_PER_TICKET};

use super::helpers::{
    count_keyword_matches, extract_key_points, extract_keywords, extract_summary,
    is_command_like, is_package_like,
};
use super::types::FetchResult;

/// Knowledge fetcher - orchestrates knowledge gathering
pub struct KnowledgeFetcher {
    /// Wiki cache
    wiki_cache: WikiCache,
    /// Maximum snippets to return
    max_snippets: usize,
    /// Timeout per fetch in ms
    timeout_ms: u64,
}

impl Default for KnowledgeFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeFetcher {
    /// Create a new fetcher with defaults
    pub fn new() -> Self {
        Self {
            wiki_cache: WikiCache::new(),
            max_snippets: MAX_SNIPPETS_PER_TICKET,
            timeout_ms: FETCH_TIMEOUT_MS,
        }
    }

    /// Set custom wiki cache
    pub fn with_cache(mut self, cache: WikiCache) -> Self {
        self.wiki_cache = cache;
        self
    }

    /// Set max snippets
    pub fn with_max_snippets(mut self, max: usize) -> Self {
        self.max_snippets = max;
        self
    }

    /// Get max snippets
    pub fn max_snippets(&self) -> usize {
        self.max_snippets
    }

    /// Fetch knowledge for a ticket
    pub fn fetch(
        &self,
        intent: &str,
        domain: &str,
        entities: &[String],
        question: &str,
    ) -> FetchResult {
        let start = std::time::Instant::now();

        // Get research policy
        let policy = get_research_policy(intent, domain, entities);

        // If no knowledge needed, return empty
        if !policy.needs_knowledge {
            return FetchResult {
                snippets: vec![],
                topics_searched: vec![],
                sources_checked: vec![],
                has_knowledge: false,
                duration_ms: start.elapsed().as_millis() as u64,
            };
        }

        // Collect snippets from all sources
        let mut snippets = vec![];
        let mut sources_checked = HashSet::new();
        let mut snippet_id = 0;

        // Extract keywords from question for relevance scoring
        let keywords = extract_keywords(question);

        // Fetch from each topic
        for topic in &policy.topics {
            if snippets.len() >= self.max_snippets {
                break;
            }

            // 1. Try man page
            if let Some(result) = fetch_man_page(topic) {
                sources_checked.insert(KnowledgeSource::ManPage);
                let mut snippet =
                    KnowledgeSnippet::from_man(&format!("k{}", snippet_id), topic, &result.content);
                snippet = self.enhance_snippet(snippet, &keywords);
                snippets.push(snippet);
                snippet_id += 1;
            }

            // 2. Try help output (if it looks like a command)
            if is_command_like(topic) {
                if let Some(result) = fetch_help_output(topic) {
                    sources_checked.insert(KnowledgeSource::Help);
                    let mut snippet = KnowledgeSnippet::from_help(
                        &format!("k{}", snippet_id),
                        topic,
                        &result.content,
                    );
                    snippet = self.enhance_snippet(snippet, &keywords);
                    snippets.push(snippet);
                    snippet_id += 1;
                }
            }

            // 3. Try Arch Wiki
            if let Some(result) = fetch_arch_wiki(topic) {
                sources_checked.insert(KnowledgeSource::ArchWiki);
                let mut snippet = KnowledgeSnippet::from_wiki(
                    &format!("k{}", snippet_id),
                    topic,
                    None,
                    &result.content,
                );
                snippet = self.enhance_snippet(snippet, &keywords);
                snippets.push(snippet);
                snippet_id += 1;
            }

            // 4. Try local docs
            if let Some(result) = fetch_local_doc(topic) {
                sources_checked.insert(KnowledgeSource::LocalDoc);
                let mut snippet = KnowledgeSnippet::from_local_doc(
                    &format!("k{}", snippet_id),
                    &result.source_path,
                    &result.content,
                );
                snippet = self.enhance_snippet(snippet, &keywords);
                snippets.push(snippet);
                snippet_id += 1;
            }

            // 5. Try pacman for package-like topics
            if is_package_like(topic) {
                if let Some(result) = fetch_pacman_info(topic) {
                    sources_checked.insert(KnowledgeSource::PacmanDoc);
                    let mut snippet = KnowledgeSnippet::from_pacman(
                        &format!("k{}", snippet_id),
                        topic,
                        &result.content,
                    );
                    snippet = self.enhance_snippet(snippet, &keywords);
                    snippets.push(snippet);
                    snippet_id += 1;
                }
            }
        }

        // Sort by relevance
        snippets.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Truncate to max
        snippets.truncate(self.max_snippets);

        FetchResult {
            has_knowledge: !snippets.is_empty(),
            snippets,
            topics_searched: policy.topics,
            sources_checked: sources_checked.into_iter().collect(),
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Fetch for specific topics (bypasses policy)
    pub fn fetch_topics(&self, topics: &[String], question: &str) -> FetchResult {
        let start = std::time::Instant::now();
        let mut snippets = vec![];
        let mut sources_checked = HashSet::new();
        let mut snippet_id = 0;

        let keywords = extract_keywords(question);

        for topic in topics {
            if snippets.len() >= self.max_snippets {
                break;
            }

            // Try all sources
            if let Some(result) = fetch_man_page(topic) {
                sources_checked.insert(KnowledgeSource::ManPage);
                let mut snippet =
                    KnowledgeSnippet::from_man(&format!("k{}", snippet_id), topic, &result.content);
                snippet = self.enhance_snippet(snippet, &keywords);
                snippets.push(snippet);
                snippet_id += 1;
            }

            if let Some(result) = fetch_arch_wiki(topic) {
                sources_checked.insert(KnowledgeSource::ArchWiki);
                let mut snippet = KnowledgeSnippet::from_wiki(
                    &format!("k{}", snippet_id),
                    topic,
                    None,
                    &result.content,
                );
                snippet = self.enhance_snippet(snippet, &keywords);
                snippets.push(snippet);
                snippet_id += 1;
            }
        }

        snippets.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        snippets.truncate(self.max_snippets);

        FetchResult {
            has_knowledge: !snippets.is_empty(),
            snippets,
            topics_searched: topics.to_vec(),
            sources_checked: sources_checked.into_iter().collect(),
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Enhance snippet with summary and key points (heuristic)
    fn enhance_snippet(
        &self,
        mut snippet: KnowledgeSnippet,
        keywords: &[String],
    ) -> KnowledgeSnippet {
        // Extract summary from first few sentences
        let summary = extract_summary(&snippet.raw_excerpt, 3);
        snippet = snippet.with_summary(&summary);

        // Extract key points (lines containing keywords)
        let key_points = extract_key_points(&snippet.raw_excerpt, keywords, 5);
        snippet = snippet.with_key_points(key_points);

        // Boost relevance based on keyword matches
        let keyword_matches = count_keyword_matches(&snippet.raw_excerpt, keywords);
        let boost = (keyword_matches as f32 * 0.05).min(0.2);
        snippet.relevance = (snippet.relevance + boost).min(1.0);

        snippet
    }
}
