// v0.0.671: Settings Aggregation (Phase 247)
// Aggregate settings values with various functions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Aggregation function
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AggregateFunction {
    /// Count values
    #[default]
    Count,
    /// Sum numeric values
    Sum,
    /// Average numeric values
    Avg,
    /// Minimum value
    Min,
    /// Maximum value
    Max,
}

impl std::fmt::Display for AggregateFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Count => write!(f, "count"),
            Self::Sum => write!(f, "sum"),
            Self::Avg => write!(f, "avg"),
            Self::Min => write!(f, "min"),
            Self::Max => write!(f, "max"),
        }
    }
}

/// Group by type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GroupByType {
    /// Group by key prefix
    #[default]
    Prefix,
    /// Group by key suffix
    Suffix,
    /// Group by value
    Value,
    /// Group by pattern
    Pattern,
}

impl std::fmt::Display for GroupByType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Prefix => write!(f, "prefix"),
            Self::Suffix => write!(f, "suffix"),
            Self::Value => write!(f, "value"),
            Self::Pattern => write!(f, "pattern"),
        }
    }
}

/// Aggregator config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatorConfig {
    /// Default function
    pub default_function: AggregateFunction,
    /// Default group by
    pub default_group_by: GroupByType,
    /// Include empty groups
    pub include_empty: bool,
    /// Sort results
    pub sort_results: bool,
}

impl AggregatorConfig {
    /// Create new config
    pub fn new(function: AggregateFunction) -> Self {
        Self {
            default_function: function,
            default_group_by: GroupByType::Prefix,
            include_empty: false,
            sort_results: true,
        }
    }

    /// Set group by
    pub fn group_by(mut self, group_by: GroupByType) -> Self {
        self.default_group_by = group_by;
        self
    }

    /// Set include empty
    pub fn include_empty(mut self, include: bool) -> Self {
        self.include_empty = include;
        self
    }

    /// Set sort results
    pub fn sort_results(mut self, sort: bool) -> Self {
        self.sort_results = sort;
        self
    }
}

impl Default for AggregatorConfig {
    fn default() -> Self {
        Self::new(AggregateFunction::Count)
    }
}

/// Aggregation result entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateEntry {
    /// Group key
    pub group: String,
    /// Aggregated value
    pub value: f64,
    /// Count in group
    pub count: usize,
}

impl AggregateEntry {
    /// Create new entry
    pub fn new(group: impl Into<String>, value: f64, count: usize) -> Self {
        Self {
            group: group.into(),
            value,
            count,
        }
    }
}

/// Aggregation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationResult {
    /// Entries
    pub entries: Vec<AggregateEntry>,
    /// Function used
    pub function: AggregateFunction,
    /// Total groups
    pub total_groups: usize,
    /// Total items
    pub total_items: usize,
}

impl AggregationResult {
    /// Create new result
    pub fn new(entries: Vec<AggregateEntry>, function: AggregateFunction) -> Self {
        let total_groups = entries.len();
        let total_items = entries.iter().map(|e| e.count).sum();
        Self {
            entries,
            function,
            total_groups,
            total_items,
        }
    }

    /// Get entry by group
    pub fn get(&self, group: &str) -> Option<&AggregateEntry> {
        self.entries.iter().find(|e| e.group == group)
    }

    /// Has entries
    pub fn has_entries(&self) -> bool {
        !self.entries.is_empty()
    }
}

impl Default for AggregationResult {
    fn default() -> Self {
        Self::new(Vec::new(), AggregateFunction::Count)
    }
}

/// Aggregator stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AggregatorStats {
    /// Total aggregations
    pub total_aggregations: usize,
    /// Total groups created
    pub total_groups: usize,
    /// Total items processed
    pub total_items: usize,
    /// By function
    pub by_function: HashMap<String, usize>,
}

impl AggregatorStats {
    /// Record aggregation
    pub fn record(&mut self, result: &AggregationResult) {
        self.total_aggregations += 1;
        self.total_groups += result.total_groups;
        self.total_items += result.total_items;
        *self.by_function.entry(result.function.to_string()).or_insert(0) += 1;
    }

    /// Groups per aggregation
    pub fn groups_per_aggregation(&self) -> f64 {
        if self.total_aggregations == 0 {
            0.0
        } else {
            self.total_groups as f64 / self.total_aggregations as f64
        }
    }
}

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

/// Aggregator registry
#[derive(Debug, Clone, Default)]
pub struct AggregatorRegistry {
    /// Aggregators by ID
    aggregators: HashMap<String, SettingsAggregator>,
}

impl AggregatorRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register aggregator
    pub fn register(&mut self, id: impl Into<String>, aggregator: SettingsAggregator) {
        self.aggregators.insert(id.into(), aggregator);
    }

    /// Unregister aggregator
    pub fn unregister(&mut self, id: &str) -> bool {
        self.aggregators.remove(id).is_some()
    }

    /// Get aggregator
    pub fn get(&self, id: &str) -> Option<&SettingsAggregator> {
        self.aggregators.get(id)
    }

    /// Get aggregator mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsAggregator> {
        self.aggregators.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.aggregators.len()
    }
}

/// Format aggregator registry
pub fn format_aggregator_registry(registry: &AggregatorRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Aggregator Registry:\n");
    output.push_str(&format!("  Aggregators: {}\n", registry.count()));
    output
}

/// Check if query is about aggregator
pub fn is_aggregator_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("aggregate") || lower.contains("group by") || lower.contains("sum settings")
}

/// Fun fact about aggregator
pub fn aggregator_fun_fact() -> &'static str {
    "Anna's settings aggregator can group and summarize settings with various functions!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_display() {
        assert_eq!(format!("{}", AggregateFunction::Count), "count");
        assert_eq!(format!("{}", AggregateFunction::Sum), "sum");
    }

    #[test]
    fn test_group_by_display() {
        assert_eq!(format!("{}", GroupByType::Prefix), "prefix");
        assert_eq!(format!("{}", GroupByType::Suffix), "suffix");
    }

    #[test]
    fn test_config_new() {
        let c = AggregatorConfig::new(AggregateFunction::Count);
        assert!(c.sort_results);
    }

    #[test]
    fn test_config_builder() {
        let c = AggregatorConfig::new(AggregateFunction::Sum)
            .group_by(GroupByType::Suffix)
            .include_empty(true);
        assert_eq!(c.default_group_by, GroupByType::Suffix);
        assert!(c.include_empty);
    }

    #[test]
    fn test_entry_new() {
        let e = AggregateEntry::new("group", 10.0, 5);
        assert_eq!(e.group, "group");
        assert_eq!(e.count, 5);
    }

    #[test]
    fn test_result_new() {
        let entries = vec![AggregateEntry::new("g1", 5.0, 5)];
        let r = AggregationResult::new(entries, AggregateFunction::Count);
        assert!(r.has_entries());
        assert_eq!(r.total_groups, 1);
    }

    #[test]
    fn test_result_get() {
        let entries = vec![AggregateEntry::new("g1", 5.0, 5)];
        let r = AggregationResult::new(entries, AggregateFunction::Count);
        assert!(r.get("g1").is_some());
        assert!(r.get("g2").is_none());
    }

    #[test]
    fn test_stats_record() {
        let mut s = AggregatorStats::default();
        let entries = vec![AggregateEntry::new("g1", 5.0, 5)];
        let r = AggregationResult::new(entries, AggregateFunction::Count);
        s.record(&r);
        assert_eq!(s.total_aggregations, 1);
        assert_eq!(s.total_groups, 1);
    }

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

    #[test]
    fn test_registry_new() {
        let r = AggregatorRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = AggregatorRegistry::new();
        r.register("a1", SettingsAggregator::new(AggregatorConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_aggregator_query() {
        assert!(is_aggregator_query("aggregate settings"));
        assert!(!is_aggregator_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = aggregator_fun_fact();
        assert!(fact.contains("aggregator"));
    }
}
