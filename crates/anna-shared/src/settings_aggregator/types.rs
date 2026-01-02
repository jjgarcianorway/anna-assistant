// v0.0.600: Settings Aggregator Types (Phase 176)
// Type definitions for settings aggregation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Aggregation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationType {
    /// Count values
    Count,
    /// Sum numeric values
    Sum,
    /// Average numeric values
    Average,
    /// Min value
    Min,
    /// Max value
    Max,
    /// List all values
    List,
    /// Group by value
    GroupBy,
}

impl std::fmt::Display for AggregationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Count => write!(f, "count"),
            Self::Sum => write!(f, "sum"),
            Self::Average => write!(f, "average"),
            Self::Min => write!(f, "min"),
            Self::Max => write!(f, "max"),
            Self::List => write!(f, "list"),
            Self::GroupBy => write!(f, "group_by"),
        }
    }
}

/// Aggregation scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationScope {
    /// All settings
    All,
    /// Single category
    Category,
    /// Multiple categories
    Categories,
    /// By key pattern
    Pattern,
}

impl std::fmt::Display for AggregationScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "all"),
            Self::Category => write!(f, "category"),
            Self::Categories => write!(f, "categories"),
            Self::Pattern => write!(f, "pattern"),
        }
    }
}

/// Aggregation definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationDef {
    /// Unique ID
    pub id: String,
    /// Name
    pub name: String,
    /// Aggregation type
    pub agg_type: AggregationType,
    /// Scope
    pub scope: AggregationScope,
    /// Target categories
    pub categories: Vec<SettingsCategory>,
    /// Key pattern
    pub pattern: Option<String>,
    /// Description
    pub description: String,
}

impl AggregationDef {
    /// Create new aggregation
    pub fn new(id: impl Into<String>, agg_type: AggregationType) -> Self {
        Self {
            id: id.into(),
            name: String::new(),
            agg_type,
            scope: AggregationScope::All,
            categories: Vec::new(),
            pattern: None,
            description: String::new(),
        }
    }

    /// Set name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set scope
    pub fn scope(mut self, scope: AggregationScope) -> Self {
        self.scope = scope;
        self
    }

    /// Add category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.categories.push(category);
        self.scope = if self.categories.len() == 1 {
            AggregationScope::Category
        } else {
            AggregationScope::Categories
        };
        self
    }

    /// Set pattern
    pub fn pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self.scope = AggregationScope::Pattern;
        self
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
}

/// Aggregation result value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggValue {
    /// Integer
    Int(i64),
    /// Float
    Float(f64),
    /// String
    String(String),
    /// List
    List(Vec<String>),
    /// Map
    Map(HashMap<String, usize>),
}

impl std::fmt::Display for AggValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(v) => write!(f, "{}", v),
            Self::Float(v) => write!(f, "{:.2}", v),
            Self::String(v) => write!(f, "{}", v),
            Self::List(v) => write!(f, "[{}]", v.join(", ")),
            Self::Map(v) => {
                let items: Vec<_> = v.iter().map(|(k, c)| format!("{}: {}", k, c)).collect();
                write!(f, "{{{}}}", items.join(", "))
            }
        }
    }
}

/// Aggregation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationResult {
    /// Aggregation ID
    pub agg_id: String,
    /// Result value
    pub value: AggValue,
    /// Item count
    pub count: usize,
    /// Success
    pub success: bool,
}

impl AggregationResult {
    /// Create new result
    pub fn new(agg_id: impl Into<String>, value: AggValue, count: usize) -> Self {
        Self {
            agg_id: agg_id.into(),
            value,
            count,
            success: true,
        }
    }

    /// Mark as failed
    pub fn fail(agg_id: impl Into<String>) -> Self {
        Self {
            agg_id: agg_id.into(),
            value: AggValue::Int(0),
            count: 0,
            success: false,
        }
    }
}

/// Settings summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SettingsSummary {
    /// Total settings count
    pub total: usize,
    /// Per-category counts
    pub by_category: HashMap<SettingsCategory, usize>,
    /// Modified count
    pub modified: usize,
    /// Default count
    pub defaults: usize,
}

impl SettingsSummary {
    /// Create new summary
    pub fn new() -> Self {
        Self::default()
    }

    /// Add category count
    pub fn add_category(&mut self, category: SettingsCategory, count: usize) {
        self.by_category.insert(category, count);
        self.total += count;
    }

    /// Set modified count
    pub fn modified(mut self, count: usize) -> Self {
        self.modified = count;
        self
    }

    /// Set defaults count
    pub fn defaults(mut self, count: usize) -> Self {
        self.defaults = count;
        self
    }

    /// Category count
    pub fn category_count(&self) -> usize {
        self.by_category.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregation_type_display() {
        assert_eq!(format!("{}", AggregationType::Count), "count");
        assert_eq!(format!("{}", AggregationType::Average), "average");
    }

    #[test]
    fn test_aggregation_scope_display() {
        assert_eq!(format!("{}", AggregationScope::All), "all");
        assert_eq!(format!("{}", AggregationScope::Pattern), "pattern");
    }

    #[test]
    fn test_aggregation_def_new() {
        let a = AggregationDef::new("a1", AggregationType::Count);
        assert_eq!(a.id, "a1");
    }

    #[test]
    fn test_aggregation_def_builder() {
        let a = AggregationDef::new("a1", AggregationType::Sum)
            .name("Total")
            .category(SettingsCategory::Personality);
        assert_eq!(a.scope, AggregationScope::Category);
    }

    #[test]
    fn test_agg_value_display() {
        assert_eq!(format!("{}", AggValue::Int(42)), "42");
        assert_eq!(format!("{}", AggValue::Float(3.14)), "3.14");
    }

    #[test]
    fn test_aggregation_result_new() {
        let r = AggregationResult::new("a1", AggValue::Int(10), 5);
        assert!(r.success);
        assert_eq!(r.count, 5);
    }

    #[test]
    fn test_aggregation_result_fail() {
        let r = AggregationResult::fail("a1");
        assert!(!r.success);
    }

    #[test]
    fn test_summary_new() {
        let s = SettingsSummary::new();
        assert_eq!(s.total, 0);
    }

    #[test]
    fn test_summary_add_category() {
        let mut s = SettingsSummary::new();
        s.add_category(SettingsCategory::Personality, 10);
        assert_eq!(s.total, 10);
        assert_eq!(s.category_count(), 1);
    }
}
