//! Knowledge Engine Implementation
//!
//! Core engine logic for fetching and caching knowledge from local sources.

use super::fetchers::{self, KnowledgeFetcher};
use super::types::{
    CacheEntry, KnowledgeContext, KnowledgeEngineHit, KnowledgeKind, KnowledgeRequest,
    KnowledgeResponse,
};
use super::utils::{is_safe_command, TOPIC_COMMANDS};
use std::path::PathBuf;
use std::time::Duration;

/// Knowledge Engine - fetches and caches knowledge
pub struct KnowledgeEngine {
    /// Cache directory
    cache_dir: PathBuf,
    /// Max snippet length
    max_snippet_len: usize,
    /// Command timeout
    timeout: Duration,
    /// Enabled sources
    enabled_sources: Vec<KnowledgeKind>,
}

impl Default for KnowledgeEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeFetcher for KnowledgeEngine {
    fn max_snippet_len(&self) -> usize {
        self.max_snippet_len
    }

    fn extract_snippet_with_lines(
        &self,
        content: &str,
        keyword: Option<&str>,
    ) -> (String, Option<(usize, usize)>) {
        let lines: Vec<&str> = content.lines().collect();

        // If keyword provided, try to find relevant section
        if let Some(kw) = keyword {
            let kw_lower = kw.to_lowercase();
            for (i, line) in lines.iter().enumerate() {
                if line.to_lowercase().contains(&kw_lower) {
                    // Return context around match (line numbers are 1-indexed)
                    let start = i.saturating_sub(2);
                    let end = (i + 5).min(lines.len());
                    let snippet: String = lines[start..end].join("\n");
                    return (
                        super::utils::truncate(&snippet, self.max_snippet_len),
                        Some((start + 1, end)),
                    );
                }
            }
        }

        // Otherwise, return beginning (skip empty lines)
        let meaningful: Vec<&str> = lines
            .iter()
            .filter(|l| !l.trim().is_empty())
            .take(10)
            .cloned()
            .collect();
        let end_line = meaningful.len().min(10);
        (
            super::utils::truncate(&meaningful.join("\n"), self.max_snippet_len),
            Some((1, end_line)),
        )
    }
}

impl KnowledgeEngine {
    /// Create new knowledge engine
    pub fn new() -> Self {
        let cache_dir = std::env::var("ANNA_STATE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/var/lib/anna"))
            .join("knowledge_cache");

        Self {
            cache_dir,
            max_snippet_len: 500,
            timeout: Duration::from_secs(3),
            enabled_sources: vec![
                KnowledgeKind::ManPage,
                KnowledgeKind::CliHelp,
                KnowledgeKind::LocalDoc,
            ],
        }
    }

    /// Query knowledge sources
    pub fn query(&self, request: &KnowledgeRequest) -> KnowledgeResponse {
        let start = std::time::Instant::now();
        let mut response = KnowledgeResponse::default();
        let mut hits = Vec::new();

        // Determine commands to search
        let commands = if request.context.commands.is_empty() {
            self.commands_for_topic(&request.topic, &request.context.domain)
        } else {
            request.context.commands.clone()
        };

        // Search each requested source
        for source in &request.sources {
            if !self.enabled_sources.contains(source) {
                continue;
            }

            response.sources_searched.push(*source);

            match source {
                KnowledgeKind::ManPage => {
                    for cmd in &commands {
                        // Check cache first
                        if let Some(cached) = self.get_cached(&format!("man:{}", cmd)) {
                            hits.push(cached);
                        } else if let Ok(hit) = fetchers::fetch_man_page(self, cmd) {
                            self.cache_hit(&hit);
                            hits.push(hit);
                        } else {
                            // Silently skip errors
                        }
                    }
                }
                KnowledgeKind::CliHelp => {
                    for cmd in &commands {
                        // Check cache first
                        if let Some(cached) = self.get_cached(&format!("help:{}", cmd)) {
                            hits.push(cached);
                        } else if let Ok(hit) = fetchers::fetch_help(self, cmd) {
                            self.cache_hit(&hit);
                            hits.push(hit);
                        } else {
                            // Silently skip errors
                        }
                    }
                }
                KnowledgeKind::LocalDoc => {
                    if let Ok(doc_hits) = fetchers::search_local_docs(self, &request.topic) {
                        hits.extend(doc_hits);
                    }
                }
                KnowledgeKind::ArchWiki => {
                    if let Ok(wiki_hits) = fetchers::search_arch_wiki(self, &request.topic) {
                        hits.extend(wiki_hits);
                    }
                }
                KnowledgeKind::BuiltIn => {
                    // Built-in handled by doc_brain
                }
            }
        }

        // Score and sort by relevance
        for hit in &mut hits {
            hit.relevance = self.score_relevance(hit, &request.topic, &request.context);
        }
        hits.sort_by(|a, b| b.relevance.cmp(&a.relevance));
        hits.truncate(request.limit);

        response.hits = hits;
        response.query_time_ms = start.elapsed().as_millis() as u64;
        response
    }

    /// Score relevance of a hit
    fn score_relevance(&self, hit: &KnowledgeEngineHit, topic: &str, ctx: &KnowledgeContext) -> u8 {
        let mut score = hit.relevance;

        // Boost for domain match
        if hit.doc_id.to_lowercase().contains(&ctx.domain) {
            score = score.saturating_add(10);
        }

        // Boost for topic keywords in snippet
        let keywords: Vec<&str> = topic.split_whitespace().collect();
        for kw in keywords {
            if hit.snippet.to_lowercase().contains(&kw.to_lowercase()) {
                score = score.saturating_add(5);
            }
        }

        score.min(100)
    }

    /// Get commands relevant to a topic
    fn commands_for_topic(&self, topic: &str, domain: &str) -> Vec<String> {
        TOPIC_COMMANDS
            .get(domain)
            .map(|cmds| cmds.iter().map(|s| s.to_string()).collect())
            .unwrap_or_else(|| {
                // Extract command-like words from topic
                topic
                    .split_whitespace()
                    .filter(|w| is_safe_command(w))
                    .map(String::from)
                    .collect()
            })
    }

    /// Get cached hit
    fn get_cached(&self, doc_id: &str) -> Option<KnowledgeEngineHit> {
        let path = self.cache_path(doc_id);
        let content = std::fs::read_to_string(&path).ok()?;
        let entry: CacheEntry = serde_json::from_str(&content).ok()?;

        // Check expiry (1 week)
        let now = super::utils::current_secs();
        if now > entry.cached_at + 7 * 24 * 3600 {
            return None;
        }

        Some(entry.hit)
    }

    /// Cache a hit
    fn cache_hit(&self, hit: &KnowledgeEngineHit) {
        let _ = std::fs::create_dir_all(&self.cache_dir);
        let entry = CacheEntry {
            hit: hit.clone(),
            cached_at: super::utils::current_secs(),
        };
        let path = self.cache_path(&hit.doc_id);
        let _ = std::fs::write(path, serde_json::to_string(&entry).unwrap_or_default());
    }

    /// Get cache path for doc_id
    fn cache_path(&self, doc_id: &str) -> PathBuf {
        let safe_name = doc_id.replace([':', '/'], "_");
        self.cache_dir.join(format!("{}.json", safe_name))
    }
}
