// v0.0.681: Settings Iterator (Phase 257)
// Main iterator implementation

use std::collections::HashMap;
use super::types::{IterationFilter, IterationOrder};
use super::config::IteratorConfig;
use super::item::IterationItem;
use super::result::IterationResult;
use super::stats::IteratorStats;

/// Settings iterator
#[derive(Debug, Clone, Default)]
pub struct SettingsIterator {
    /// Config
    config: IteratorConfig,
    /// Stats
    stats: IteratorStats,
}

impl SettingsIterator {
    /// Create new iterator
    pub fn new(config: IteratorConfig) -> Self {
        Self {
            config,
            stats: IteratorStats::default(),
        }
    }

    /// Apply filter
    fn apply_filter(&self, value: &str) -> bool {
        match self.config.filter {
            IterationFilter::None => true,
            IterationFilter::NonEmpty => !value.is_empty(),
            IterationFilter::Numeric => value.parse::<f64>().is_ok(),
            IterationFilter::Boolean => value == "true" || value == "false",
        }
    }

    /// Iterate all
    pub fn iterate(&mut self, settings: &HashMap<String, String>) -> IterationResult {
        let total = settings.len();

        // Collect and sort entries
        let mut entries: Vec<(&String, &String)> = settings.iter().collect();

        match self.config.order {
            IterationOrder::Natural => {}
            IterationOrder::Alphabetical => entries.sort_by(|a, b| a.0.cmp(b.0)),
            IterationOrder::ReverseAlphabetical => entries.sort_by(|a, b| b.0.cmp(a.0)),
            IterationOrder::ByValueLength => entries.sort_by(|a, b| a.1.len().cmp(&b.1.len())),
        }

        // Apply skip, take, and filter
        let items: Vec<IterationItem> = entries.into_iter()
            .filter(|(_, v)| self.apply_filter(v))
            .skip(self.config.skip)
            .take(if self.config.take > 0 { self.config.take } else { usize::MAX })
            .enumerate()
            .map(|(i, (k, v))| IterationItem::new(k.clone(), v.clone(), i))
            .collect();

        let result = IterationResult::new(items, total, self.config.order);
        self.stats.record(&result);
        result
    }

    /// Iterate with callback (returns keys processed)
    pub fn for_each<F>(&mut self, settings: &HashMap<String, String>, mut callback: F) -> usize
    where
        F: FnMut(&IterationItem),
    {
        let result = self.iterate(settings);
        for item in &result.items {
            callback(item);
        }
        result.items.len()
    }

    /// Iterate in batches
    pub fn iterate_batched(&mut self, settings: &HashMap<String, String>) -> Vec<IterationResult> {
        let full_result = self.iterate(settings);
        let mut batches = Vec::new();

        for chunk in full_result.items.chunks(self.config.batch_size) {
            batches.push(IterationResult::new(
                chunk.to_vec(),
                full_result.total_count,
                self.config.order,
            ));
        }

        batches
    }

    /// Get stats
    pub fn stats(&self) -> &IteratorStats {
        &self.stats
    }
}
