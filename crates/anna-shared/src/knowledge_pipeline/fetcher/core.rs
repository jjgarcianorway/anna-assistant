//! Knowledge fetcher with priority ordering (v0.0.432).
//!
//! Fetches knowledge from sources in strict priority order.
//! Stops when a sufficiently trusted result is found.

use super::super::sources::{Citation, KnowledgeSource, SourcePriority, SourceResult};
use super::super::MIN_CONFIDENCE_THRESHOLD;
use super::handlers::{fetch_cached_wiki, fetch_doc_file, fetch_help, fetch_man, fetch_probe};
use super::types::{FetchConfig, FetchResult};
use super::utils::is_likely_command;

/// Knowledge fetcher with priority-based lookup.
pub struct KnowledgeFetcher {
    config: FetchConfig,
}

impl KnowledgeFetcher {
    /// Create a new fetcher with default config.
    pub fn new() -> Self {
        Self {
            config: FetchConfig::default(),
        }
    }

    /// Create a new fetcher with custom config.
    pub fn with_config(config: FetchConfig) -> Self {
        Self { config }
    }

    /// Fetch knowledge for a query from prioritized sources.
    pub fn fetch(&self, query: &str, sources: &[KnowledgeSource]) -> FetchResult {
        let start = std::time::Instant::now();
        let mut results = Vec::new();
        let mut failed = Vec::new();
        let mut lookups = 0;

        // Sort sources by priority
        let mut sorted_sources: Vec<_> = sources.to_vec();
        sorted_sources.sort_by_key(|s| s.priority());

        for source in sorted_sources {
            if lookups >= self.config.max_lookups {
                break;
            }

            // Skip remote if disabled
            if source.priority() == SourcePriority::Remote && !self.config.allow_remote {
                continue;
            }

            lookups += 1;

            match self.fetch_from_source(&source, query) {
                Ok(result) => {
                    if result.relevance >= self.config.min_relevance {
                        results.push(result);

                        // Stop early if we have a confident result from a trusted source
                        if let Some(best) = results.first() {
                            if best.trust_score() >= MIN_CONFIDENCE_THRESHOLD {
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    failed.push(format!("{}: {}", source.description(), e));
                }
            }
        }

        // Sort by trust score
        results.sort_by(|a, b| {
            b.trust_score()
                .partial_cmp(&a.trust_score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Generate citations
        let citations: Vec<Citation> = results
            .iter()
            .take(3) // Top 3 sources
            .map(|r| Citation::new(r.source.clone(), r.relevance))
            .collect();

        let confident = results
            .first()
            .map(|r| r.trust_score() >= MIN_CONFIDENCE_THRESHOLD)
            .unwrap_or(false);

        FetchResult {
            results,
            citations,
            confident,
            failed_sources: failed,
            lookup_time_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Fetch from a specific source.
    fn fetch_from_source(
        &self,
        source: &KnowledgeSource,
        query: &str,
    ) -> Result<SourceResult, String> {
        match source {
            KnowledgeSource::Probe { name, command } => {
                fetch_probe(name, command.as_deref(), query)
            }
            KnowledgeSource::ManPage { name, section } => fetch_man(name, *section, query),
            KnowledgeSource::HelpOutput { command } => fetch_help(command, query),
            KnowledgeSource::DocFile { path } => fetch_doc_file(path, query),
            KnowledgeSource::ArchWiki { article, cached } => {
                if *cached {
                    fetch_cached_wiki(&self.config, article, query)
                } else {
                    Err("Remote wiki fetch disabled".to_string())
                }
            }
            KnowledgeSource::Wiki { name, article } => {
                fetch_cached_wiki(&self.config, &format!("{}:{}", name, article), query)
            }
            KnowledgeSource::RemoteUrl { .. } => Err("Remote URLs disabled".to_string()),
        }
    }

    /// Suggest sources for a topic.
    pub fn suggest_sources(&self, topic: &str) -> Vec<KnowledgeSource> {
        let mut sources = Vec::new();
        let topic_lower = topic.to_lowercase();

        // System-related queries → probes first
        if topic_lower.contains("memory") || topic_lower.contains("ram") {
            sources.push(KnowledgeSource::probe("meminfo"));
        }
        if topic_lower.contains("cpu") || topic_lower.contains("processor") {
            sources.push(KnowledgeSource::probe("cpuinfo"));
        }
        if topic_lower.contains("uptime") || topic_lower.contains("boot") {
            sources.push(KnowledgeSource::probe("uptime"));
            sources.push(KnowledgeSource::probe_with_cmd(
                "boot_time",
                "systemd-analyze",
            ));
        }
        if topic_lower.contains("load") {
            sources.push(KnowledgeSource::probe("loadavg"));
        }

        // Command-related → man pages and help
        let words: Vec<&str> = topic.split_whitespace().collect();
        for word in &words {
            if is_likely_command(word) {
                sources.push(KnowledgeSource::man(word));
                sources.push(KnowledgeSource::help(word));
            }
        }

        // Wiki for general topics
        if sources.is_empty() {
            sources.push(KnowledgeSource::arch_wiki(topic));
        }

        sources
    }
}

impl Default for KnowledgeFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_disables_remote() {
        let config = FetchConfig::default();
        assert!(!config.allow_remote);
    }

    #[test]
    fn test_source_suggestions() {
        let fetcher = KnowledgeFetcher::new();

        let mem_sources = fetcher.suggest_sources("memory usage");
        assert!(mem_sources
            .iter()
            .any(|s| matches!(s, KnowledgeSource::Probe { name, .. } if name == "meminfo")));

        let cpu_sources = fetcher.suggest_sources("cpu info");
        assert!(cpu_sources
            .iter()
            .any(|s| matches!(s, KnowledgeSource::Probe { name, .. } if name == "cpuinfo")));
    }
}
