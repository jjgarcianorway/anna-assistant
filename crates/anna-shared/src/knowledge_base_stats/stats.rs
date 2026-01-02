//! Knowledge base statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::entry::KnowledgeEntry;

/// Knowledge base statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeBaseStats {
    /// Entries by type
    pub by_type: HashMap<String, u64>,
    /// Entries by source
    pub by_source: HashMap<String, u64>,
    /// Entries by topic
    pub by_topic: HashMap<String, u64>,
    /// Total entries
    pub total_entries: u64,
    /// Total uses
    pub total_uses: u64,
    /// Stale entries count
    pub stale_count: u64,
    /// Recent entries (last 20)
    pub recent: Vec<KnowledgeEntry>,
    /// Most used entries (top 10)
    pub most_used: Vec<KnowledgeEntry>,
    /// Last acquisition timestamp
    pub last_acquisition: u64,
}

impl KnowledgeBaseStats {
    /// Create new empty stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a knowledge entry
    pub fn record(&mut self, entry: KnowledgeEntry, now: u64) {
        // Update type counts
        *self.by_type.entry(entry.knowledge_type.display().to_string()).or_insert(0) += 1;

        // Update source counts
        *self.by_source.entry(entry.source.display().to_string()).or_insert(0) += 1;

        // Update topic counts
        if let Some(ref topic) = entry.topic {
            *self.by_topic.entry(topic.clone()).or_insert(0) += 1;
        }

        self.total_entries += 1;
        self.total_uses += entry.use_count;

        if entry.is_stale(now) {
            self.stale_count += 1;
        }

        if entry.acquired_at > self.last_acquisition {
            self.last_acquisition = entry.acquired_at;
        }

        // Add to recent
        self.recent.insert(0, entry.clone());
        if self.recent.len() > 20 {
            self.recent.truncate(20);
        }

        // Update most used
        self.most_used.push(entry);
        self.most_used.sort_by(|a, b| b.use_count.cmp(&a.use_count));
        self.most_used.truncate(10);
    }

    /// Get recipe count
    pub fn recipe_count(&self) -> u64 {
        *self.by_type.get("Recipe").unwrap_or(&0)
    }

    /// Get fact count
    pub fn fact_count(&self) -> u64 {
        *self.by_type.get("Fact").unwrap_or(&0)
    }

    /// Get cached documentation count
    pub fn doc_cache_count(&self) -> u64 {
        let wiki = *self.by_type.get("Wiki Page").unwrap_or(&0);
        let man = *self.by_type.get("Man Page").unwrap_or(&0);
        let help = *self.by_type.get("Help Cache").unwrap_or(&0);
        wiki + man + help
    }

    /// Get user-taught count
    pub fn user_taught_count(&self) -> u64 {
        *self.by_type.get("User Taught").unwrap_or(&0)
    }

    /// Get learned (non-seed) count
    pub fn learned_count(&self) -> u64 {
        self.total_entries - *self.by_source.get("Seed").unwrap_or(&0)
    }

    /// Average uses per entry
    pub fn avg_uses(&self) -> f64 {
        if self.total_entries == 0 {
            return 0.0;
        }
        self.total_uses as f64 / self.total_entries as f64
    }

    /// Stale percentage
    pub fn stale_percent(&self) -> f64 {
        if self.total_entries == 0 {
            return 0.0;
        }
        (self.stale_count as f64 / self.total_entries as f64) * 100.0
    }

    /// Top topic
    pub fn top_topic(&self) -> Option<(&String, u64)> {
        self.by_topic.iter().max_by_key(|(_, count)| *count).map(|(k, v)| (k, *v))
    }

    /// Top source
    pub fn top_source(&self) -> Option<(&String, u64)> {
        self.by_source.iter().max_by_key(|(_, count)| *count).map(|(k, v)| (k, *v))
    }
}
