// v0.0.671: Settings Aggregation - Type Definitions
// Type definitions for settings aggregation

use serde::{Deserialize, Serialize};

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
}
