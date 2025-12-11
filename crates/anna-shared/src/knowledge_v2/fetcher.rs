//! Knowledge fetcher pipeline (v0.0.422).
//!
//! Orchestrates knowledge fetching from multiple sources:
//! 1. Man pages
//! 2. Help output
//! 3. Arch Wiki (cache)
//! 4. Local docs
//!
//! Produces normalized KnowledgeSnippet objects.

use std::collections::HashSet;

use super::cache::WikiCache;
use super::policy::{get_research_policy, ResearchPriority};
use super::snippet::{KnowledgeSnippet, KnowledgeSource};
use super::sources::{fetch_arch_wiki, fetch_help_output, fetch_local_doc, fetch_man_page, fetch_pacman_info};
use super::{MAX_SNIPPETS_PER_TICKET, FETCH_TIMEOUT_MS};

/// Result of a knowledge fetch operation
#[derive(Debug, Clone)]
pub struct FetchResult {
    /// Fetched snippets
    pub snippets: Vec<KnowledgeSnippet>,
    /// Topics that were searched
    pub topics_searched: Vec<String>,
    /// Sources that were checked
    pub sources_checked: Vec<KnowledgeSource>,
    /// Whether any knowledge was found
    pub has_knowledge: bool,
    /// Fetch duration in milliseconds
    pub duration_ms: u64,
}

impl FetchResult {
    /// Create an empty result
    pub fn empty() -> Self {
        Self {
            snippets: vec![],
            topics_searched: vec![],
            sources_checked: vec![],
            has_knowledge: false,
            duration_ms: 0,
        }
    }

    /// Get primary citation if available
    pub fn primary_citation(&self) -> Option<String> {
        self.snippets.first().map(|s| s.primary_citation())
    }

    /// Get all citations
    pub fn all_citations(&self) -> Vec<String> {
        self.snippets
            .iter()
            .flat_map(|s| s.citations.clone())
            .collect()
    }
}

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
                let mut snippet = KnowledgeSnippet::from_man(
                    &format!("k{}", snippet_id),
                    topic,
                    &result.content,
                );
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
                let mut snippet = KnowledgeSnippet::from_man(
                    &format!("k{}", snippet_id),
                    topic,
                    &result.content,
                );
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
    fn enhance_snippet(&self, mut snippet: KnowledgeSnippet, keywords: &[String]) -> KnowledgeSnippet {
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

/// Extract keywords from question
fn extract_keywords(question: &str) -> Vec<String> {
    let stop_words: HashSet<&str> = [
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
        "have", "has", "had", "do", "does", "did", "will", "would", "could",
        "should", "may", "might", "must", "i", "my", "me", "you", "your",
        "we", "our", "they", "their", "it", "its", "this", "that", "what",
        "which", "who", "whom", "how", "why", "when", "where", "to", "of",
        "in", "on", "at", "by", "for", "with", "about", "into", "through",
        "during", "before", "after", "above", "below", "from", "up", "down",
        "out", "off", "over", "under", "again", "further", "then", "once",
    ]
    .into_iter()
    .collect();

    question
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2 && !stop_words.contains(w))
        .map(String::from)
        .collect()
}

/// Extract summary from content (first N sentences)
fn extract_summary(content: &str, sentences: usize) -> String {
    let mut result = String::new();
    let mut count = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Skip section headers (all caps)
        if trimmed.chars().all(|c| c.is_ascii_uppercase() || c.is_whitespace()) && trimmed.len() < 30 {
            continue;
        }

        result.push_str(trimmed);
        result.push(' ');
        count += 1;

        if count >= sentences {
            break;
        }
    }

    result.trim().to_string()
}

/// Extract key points containing keywords
fn extract_key_points(content: &str, keywords: &[String], max: usize) -> Vec<String> {
    let mut points = vec![];

    for line in content.lines() {
        if points.len() >= max {
            break;
        }

        let line_lower = line.to_lowercase();
        let has_keyword = keywords.iter().any(|k| line_lower.contains(k));

        if has_keyword && line.len() > 10 && line.len() < 200 {
            points.push(line.trim().to_string());
        }
    }

    points
}

/// Count keyword matches in content
fn count_keyword_matches(content: &str, keywords: &[String]) -> usize {
    let content_lower = content.to_lowercase();
    keywords
        .iter()
        .filter(|k| content_lower.contains(k.as_str()))
        .count()
}

/// Check if topic looks like a command
fn is_command_like(topic: &str) -> bool {
    let topic_lower = topic.to_lowercase();
    topic_lower
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
        && !topic_lower.contains(' ')
        && topic.len() < 30
}

/// Check if topic looks like a package name
fn is_package_like(topic: &str) -> bool {
    is_command_like(topic) && !topic.starts_with('-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_keywords() {
        let keywords = extract_keywords("How do I enable vim syntax highlighting?");
        assert!(keywords.contains(&"enable".to_string()));
        assert!(keywords.contains(&"vim".to_string()));
        assert!(keywords.contains(&"syntax".to_string()));
        assert!(!keywords.contains(&"how".to_string()));
        assert!(!keywords.contains(&"do".to_string()));
    }

    #[test]
    fn test_extract_summary() {
        let content = "First sentence here.\nSecond sentence.\nThird sentence.";
        let summary = extract_summary(content, 2);
        assert!(summary.contains("First"));
        assert!(summary.contains("Second"));
    }

    #[test]
    fn test_is_command_like() {
        assert!(is_command_like("systemctl"));
        assert!(is_command_like("vim"));
        assert!(is_command_like("python3.11"));
        assert!(!is_command_like("how to"));
        assert!(!is_command_like("some long topic name"));
    }

    #[test]
    fn test_fetch_result_empty() {
        let result = FetchResult::empty();
        assert!(!result.has_knowledge);
        assert!(result.snippets.is_empty());
    }

    #[test]
    fn test_fetcher_new() {
        let fetcher = KnowledgeFetcher::new();
        assert_eq!(fetcher.max_snippets, MAX_SNIPPETS_PER_TICKET);
    }
}
