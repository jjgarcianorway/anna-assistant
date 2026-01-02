// v0.0.682: Settings Collector (Phase 258)
// Main collector implementation for gathering settings from multiple sources

use std::collections::HashMap;
use crate::settings_collector::config::CollectorConfig;
use crate::settings_collector::source::SettingsSource;
use crate::settings_collector::result::{CollectResult, CollectorStats};
use crate::settings_collector::types::CollectMode;

/// Settings collector
#[derive(Debug, Clone, Default)]
pub struct SettingsCollector {
    /// Config
    config: CollectorConfig,
    /// Stats
    stats: CollectorStats,
    /// Sources
    sources: Vec<SettingsSource>,
}

impl SettingsCollector {
    /// Create new collector
    pub fn new(config: CollectorConfig) -> Self {
        Self {
            config,
            stats: CollectorStats::default(),
            sources: Vec::new(),
        }
    }

    /// Add source
    pub fn add_source(&mut self, source: SettingsSource) {
        self.sources.push(source);
    }

    /// Clear sources
    pub fn clear_sources(&mut self) {
        self.sources.clear();
    }

    /// Source count
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }

    /// Collect all sources
    pub fn collect(&mut self) -> CollectResult {
        let mut collected = HashMap::new();
        let mut conflicts = 0;

        // Sort by priority if configured
        let mut sources = self.sources.clone();
        if self.config.respect_priority {
            sources.sort_by(|a, b| a.priority.cmp(&b.priority));
        }

        match self.config.mode {
            CollectMode::Merge => {
                for source in &sources {
                    for (key, value) in &source.settings {
                        if collected.contains_key(key) {
                            conflicts += 1;
                        }
                        collected.insert(key.clone(), value.clone());
                    }
                }
            }
            CollectMode::Union => {
                for source in &sources {
                    for (key, value) in &source.settings {
                        if !collected.contains_key(key) {
                            collected.insert(key.clone(), value.clone());
                        }
                    }
                }
            }
            CollectMode::Intersect => {
                if let Some(first) = sources.first() {
                    for (key, value) in &first.settings {
                        if sources.iter().skip(1).all(|s| s.settings.contains_key(key)) {
                            collected.insert(key.clone(), value.clone());
                        }
                    }
                }
            }
            CollectMode::Append => {
                for source in &sources {
                    for (key, value) in &source.settings {
                        let mut final_key = key.clone();
                        let mut counter = 1;
                        while collected.contains_key(&final_key) {
                            final_key = format!("{}{}{}", key, self.config.append_suffix, counter);
                            counter += 1;
                        }
                        collected.insert(final_key, value.clone());
                    }
                }
            }
        }

        let result = CollectResult::new(collected, sources.len(), conflicts, self.config.mode);
        self.stats.record(&result);
        result
    }

    /// Get stats
    pub fn stats(&self) -> &CollectorStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_collector::types::SourcePriority;

    #[test]
    fn test_collector_new() {
        let c = SettingsCollector::new(CollectorConfig::default());
        assert_eq!(c.source_count(), 0);
    }

    #[test]
    fn test_collector_add_source() {
        let mut c = SettingsCollector::new(CollectorConfig::default());
        c.add_source(SettingsSource::new("s1", "Source 1"));
        assert_eq!(c.source_count(), 1);
    }

    #[test]
    fn test_collector_merge() {
        let mut c = SettingsCollector::new(CollectorConfig::new(CollectMode::Merge));

        let mut s1 = SettingsSource::new("s1", "Source 1");
        s1.add("a", "1");
        s1.add("b", "2");

        let mut s2 = SettingsSource::new("s2", "Source 2");
        s2.add("b", "3");
        s2.add("c", "4");

        c.add_source(s1);
        c.add_source(s2);

        let result = c.collect();
        assert_eq!(result.keys_collected, 3);
        assert_eq!(result.get("b").unwrap(), "3"); // s2 overwrites
    }

    #[test]
    fn test_collector_union() {
        let mut c = SettingsCollector::new(CollectorConfig::new(CollectMode::Union));

        let mut s1 = SettingsSource::new("s1", "Source 1");
        s1.add("a", "1");

        let mut s2 = SettingsSource::new("s2", "Source 2");
        s2.add("a", "2");

        c.add_source(s1);
        c.add_source(s2);

        let result = c.collect();
        assert_eq!(result.get("a").unwrap(), "1"); // s1 wins (first)
    }
}
