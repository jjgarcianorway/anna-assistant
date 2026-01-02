//! Knowledge fetcher types (v0.0.432).

use super::super::sources::{Citation, SourceResult};
use super::super::{MAX_SOURCE_LOOKUPS, MIN_CONFIDENCE_THRESHOLD};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
