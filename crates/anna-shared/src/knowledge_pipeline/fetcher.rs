//! Knowledge fetcher with priority ordering (v0.0.432).
//!
//! Fetches knowledge from sources in strict priority order.
//! Stops when a sufficiently trusted result is found.

use super::sources::{Citation, KnowledgeSource, SourcePriority, SourceResult};
use super::{MAX_SOURCE_LOOKUPS, MIN_CONFIDENCE_THRESHOLD};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

/// Configuration for knowledge fetching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchConfig {
    /// Allow remote sources (default: false).
    pub allow_remote: bool,
    /// Minimum relevance threshold.
    pub min_relevance: f32,
    /// Maximum sources to try.
    pub max_lookups: usize,
    /// Base path for wiki cache.
    pub wiki_cache_path: Option<PathBuf>,
    /// Custom doc paths to search.
    pub doc_paths: Vec<PathBuf>,
}

impl Default for FetchConfig {
    fn default() -> Self {
        Self {
            allow_remote: false, // Remote disabled by default
            min_relevance: 0.5,
            max_lookups: MAX_SOURCE_LOOKUPS,
            wiki_cache_path: None,
            doc_paths: vec![
                PathBuf::from("/usr/share/doc"),
                PathBuf::from("/usr/share/man"),
            ],
        }
    }
}

/// Result of a fetch operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchResult {
    /// Results from all consulted sources, sorted by trust.
    pub results: Vec<SourceResult>,
    /// Citations generated from results.
    pub citations: Vec<Citation>,
    /// Whether a confident answer was found.
    pub confident: bool,
    /// Sources that were tried but failed.
    pub failed_sources: Vec<String>,
    /// Total lookup time in milliseconds.
    pub lookup_time_ms: u64,
}

impl FetchResult {
    /// Get the best result (highest trust score).
    pub fn best(&self) -> Option<&SourceResult> {
        self.results.first()
    }

    /// Check if any results were found.
    pub fn has_results(&self) -> bool {
        !self.results.is_empty()
    }

    /// Merge another fetch result into this one.
    pub fn merge(&mut self, other: FetchResult) {
        self.results.extend(other.results);
        self.citations.extend(other.citations);
        self.failed_sources.extend(other.failed_sources);
        self.lookup_time_ms += other.lookup_time_ms;

        // Re-sort by trust score
        self.results.sort_by(|a, b| {
            b.trust_score()
                .partial_cmp(&a.trust_score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Update confidence based on best result
        if let Some(best) = self.results.first() {
            self.confident = best.trust_score() >= MIN_CONFIDENCE_THRESHOLD;
        }
    }
}

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
    fn fetch_from_source(&self, source: &KnowledgeSource, query: &str) -> Result<SourceResult, String> {
        match source {
            KnowledgeSource::Probe { name, command } => {
                self.fetch_probe(name, command.as_deref(), query)
            }
            KnowledgeSource::ManPage { name, section } => {
                self.fetch_man(name, *section, query)
            }
            KnowledgeSource::HelpOutput { command } => {
                self.fetch_help(command, query)
            }
            KnowledgeSource::DocFile { path } => {
                self.fetch_doc_file(path, query)
            }
            KnowledgeSource::ArchWiki { article, cached } => {
                if *cached {
                    self.fetch_cached_wiki(article, query)
                } else {
                    Err("Remote wiki fetch disabled".to_string())
                }
            }
            KnowledgeSource::Wiki { name, article } => {
                self.fetch_cached_wiki(&format!("{}:{}", name, article), query)
            }
            KnowledgeSource::RemoteUrl { .. } => {
                Err("Remote URLs disabled".to_string())
            }
        }
    }

    /// Fetch from a probe.
    fn fetch_probe(&self, name: &str, cmd: Option<&str>, query: &str) -> Result<SourceResult, String> {
        let content = if let Some(cmd) = cmd {
            run_command(cmd)?
        } else {
            // Common probe mappings
            match name {
                "meminfo" | "memory" => std::fs::read_to_string("/proc/meminfo")
                    .map_err(|e| e.to_string())?,
                "cpuinfo" | "cpu" => std::fs::read_to_string("/proc/cpuinfo")
                    .map_err(|e| e.to_string())?,
                "uptime" => std::fs::read_to_string("/proc/uptime")
                    .map_err(|e| e.to_string())?,
                "loadavg" => std::fs::read_to_string("/proc/loadavg")
                    .map_err(|e| e.to_string())?,
                _ => return Err(format!("Unknown probe: {}", name)),
            }
        };

        let relevance = compute_relevance(&content, query);
        Ok(SourceResult::new(
            KnowledgeSource::probe(name),
            content,
            relevance,
        ))
    }

    /// Fetch from a man page.
    fn fetch_man(&self, name: &str, section: Option<u8>, query: &str) -> Result<SourceResult, String> {
        let cmd = match section {
            Some(s) => format!("man {} {} 2>/dev/null | col -b", s, name),
            None => format!("man {} 2>/dev/null | col -b", name),
        };

        let content = run_command(&cmd)?;
        if content.trim().is_empty() {
            return Err(format!("Man page not found: {}", name));
        }

        let relevance = compute_relevance(&content, query);
        Ok(SourceResult::new(
            KnowledgeSource::man(name),
            content,
            relevance,
        ))
    }

    /// Fetch from command help.
    fn fetch_help(&self, command: &str, query: &str) -> Result<SourceResult, String> {
        let content = run_command(&format!("{} --help 2>&1", command))?;
        if content.trim().is_empty() {
            return Err(format!("No help output: {}", command));
        }

        let relevance = compute_relevance(&content, query);
        Ok(SourceResult::new(
            KnowledgeSource::help(command),
            content,
            relevance,
        ))
    }

    /// Fetch from a doc file.
    fn fetch_doc_file(&self, path: &str, query: &str) -> Result<SourceResult, String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let relevance = compute_relevance(&content, query);
        Ok(SourceResult::new(
            KnowledgeSource::DocFile { path: path.to_string() },
            content,
            relevance,
        ))
    }

    /// Fetch from cached wiki.
    fn fetch_cached_wiki(&self, article: &str, query: &str) -> Result<SourceResult, String> {
        let cache_path = self.config.wiki_cache_path.as_ref()
            .ok_or("Wiki cache not configured")?;

        let article_path = cache_path.join(format!("{}.txt", sanitize_filename(article)));
        if !article_path.exists() {
            return Err(format!("Wiki article not cached: {}", article));
        }

        let content = std::fs::read_to_string(&article_path).map_err(|e| e.to_string())?;
        let relevance = compute_relevance(&content, query);
        Ok(SourceResult::new(
            KnowledgeSource::arch_wiki(article),
            content,
            relevance,
        ))
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
            sources.push(KnowledgeSource::probe_with_cmd("boot_time", "systemd-analyze"));
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

/// Run a shell command and return output.
fn run_command(cmd: &str) -> Result<String, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !stdout.is_empty() {
        Ok(stdout)
    } else if !stderr.is_empty() {
        Ok(stderr)
    } else {
        Err("No output".to_string())
    }
}

/// Compute relevance of content to query (simple keyword matching).
fn compute_relevance(content: &str, query: &str) -> f32 {
    let content_lower = content.to_lowercase();
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower
        .split_whitespace()
        .filter(|w| w.len() > 2)
        .collect();

    if query_words.is_empty() {
        return 0.5;
    }

    let matches = query_words.iter()
        .filter(|w| content_lower.contains(*w))
        .count();

    (matches as f32 / query_words.len() as f32).min(1.0)
}

/// Check if a word looks like a command.
fn is_likely_command(word: &str) -> bool {
    word.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        && word.len() >= 2
        && word.chars().next().map(|c| c.is_ascii_lowercase()).unwrap_or(false)
}

/// Sanitize a string for use as a filename.
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
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
    fn test_relevance_computation() {
        let content = "MemTotal: 32000000 kB\nMemFree: 16000000 kB\nAvailable memory: plenty";
        let relevance = compute_relevance(content, "memory free available");
        // "memory", "free", "available" are all > 2 chars and present in content
        assert!(relevance > 0.5, "relevance was {}", relevance);
    }

    #[test]
    fn test_source_suggestions() {
        let fetcher = KnowledgeFetcher::new();

        let mem_sources = fetcher.suggest_sources("memory usage");
        assert!(mem_sources.iter().any(|s| matches!(s, KnowledgeSource::Probe { name, .. } if name == "meminfo")));

        let cpu_sources = fetcher.suggest_sources("cpu info");
        assert!(cpu_sources.iter().any(|s| matches!(s, KnowledgeSource::Probe { name, .. } if name == "cpuinfo")));
    }
}
