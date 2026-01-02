// v0.0.671: Settings Aggregation - Aggregator Implementation
// Main aggregator implementation for settings

use std::collections::HashMap;

use super::stats::AggregatorStats;
use super::types::{AggregateEntry, AggregateFunction, AggregationResult, AggregatorConfig, GroupByType};

/// Settings aggregator
#[derive(Debug, Clone, Default)]
pub struct SettingsAggregator {
    /// Config
    config: AggregatorConfig,
    /// Stats
    stats: AggregatorStats,
}

impl SettingsAggregator {
    /// Create new aggregator
    pub fn new(config: AggregatorConfig) -> Self {
        Self {
            config,
            stats: AggregatorStats::default(),
        }
    }

    /// Extract group key from setting key
    fn extract_group(&self, key: &str) -> String {
        match self.config.default_group_by {
            GroupByType::Prefix => {
                key.split('.').next().unwrap_or(key).to_string()
            }
            GroupByType::Suffix => {
                key.split('.').last().unwrap_or(key).to_string()
            }
            GroupByType::Value | GroupByType::Pattern => key.to_string(),
        }
    }

    /// Aggregate by count
    pub fn count(&mut self, settings: &HashMap<String, String>) -> AggregationResult {
        let mut groups: HashMap<String, usize> = HashMap::new();

        for key in settings.keys() {
            let group = self.extract_group(key);
            *groups.entry(group).or_insert(0) += 1;
        }

        let mut entries: Vec<AggregateEntry> = groups
            .into_iter()
            .map(|(g, c)| AggregateEntry::new(g, c as f64, c))
            .collect();

        if self.config.sort_results {
            entries.sort_by(|a, b| a.group.cmp(&b.group));
        }

        let result = AggregationResult::new(entries, AggregateFunction::Count);
        self.stats.record(&result);
        result
    }

    /// Aggregate with custom function
    pub fn aggregate(&mut self, settings: &HashMap<String, String>, function: AggregateFunction) -> AggregationResult {
        match function {
            AggregateFunction::Count => self.count(settings),
            _ => {
                // For numeric functions, try to parse values
                let mut groups: HashMap<String, Vec<f64>> = HashMap::new();

                for (key, value) in settings {
                    let group = self.extract_group(key);
                    if let Ok(num) = value.parse::<f64>() {
                        groups.entry(group).or_default().push(num);
                    }
                }

                let entries: Vec<AggregateEntry> = groups
                    .into_iter()
                    .map(|(g, values)| {
                        let count = values.len();
                        let agg_value = match function {
                            AggregateFunction::Sum => values.iter().sum(),
                            AggregateFunction::Avg => values.iter().sum::<f64>() / count as f64,
                            AggregateFunction::Min => values.iter().cloned().fold(f64::INFINITY, f64::min),
                            AggregateFunction::Max => values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
                            AggregateFunction::Count => count as f64,
                        };
                        AggregateEntry::new(g, agg_value, count)
                    })
                    .collect();

                let result = AggregationResult::new(entries, function);
                self.stats.record(&result);
                result
            }
        }
    }

    /// Get stats
    pub fn stats(&self) -> &AggregatorStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregator_new() {
        let a = SettingsAggregator::new(AggregatorConfig::default());
        assert_eq!(a.stats().total_aggregations, 0);
    }

    #[test]
    fn test_aggregator_count() {
        let mut a = SettingsAggregator::new(AggregatorConfig::default());
        let mut settings = HashMap::new();
        settings.insert("app.name".to_string(), "test".to_string());
        settings.insert("app.version".to_string(), "1.0".to_string());
        settings.insert("db.host".to_string(), "localhost".to_string());

        let result = a.count(&settings);
        assert_eq!(result.total_groups, 2); // app, db
    }

    #[test]
    fn test_aggregator_sum() {
        let mut a = SettingsAggregator::new(AggregatorConfig::default());
        let mut settings = HashMap::new();
        settings.insert("metrics.count1".to_string(), "10".to_string());
        settings.insert("metrics.count2".to_string(), "20".to_string());

        let result = a.aggregate(&settings, AggregateFunction::Sum);
        let entry = result.get("metrics");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().value, 30.0);
    }
}
