// v0.0.674: Settings Filter Types (Phase 250)
// Type definitions for settings filter

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Filter type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FilterType {
    /// Include matching
    #[default]
    Include,
    /// Exclude matching
    Exclude,
    /// Allow list
    AllowList,
    /// Block list
    BlockList,
}

impl std::fmt::Display for FilterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Include => write!(f, "include"),
            Self::Exclude => write!(f, "exclude"),
            Self::AllowList => write!(f, "allow_list"),
            Self::BlockList => write!(f, "block_list"),
        }
    }
}

/// Filter predicate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FilterPredicate {
    /// Is empty
    #[default]
    IsEmpty,
    /// Is not empty
    IsNotEmpty,
    /// Is numeric
    IsNumeric,
    /// Is boolean
    IsBoolean,
    /// Has value
    HasValue,
}

impl std::fmt::Display for FilterPredicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IsEmpty => write!(f, "is_empty"),
            Self::IsNotEmpty => write!(f, "is_not_empty"),
            Self::IsNumeric => write!(f, "is_numeric"),
            Self::IsBoolean => write!(f, "is_boolean"),
            Self::HasValue => write!(f, "has_value"),
        }
    }
}

/// Filter config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    /// Default filter type
    pub default_type: FilterType,
    /// Case insensitive
    pub case_insensitive: bool,
    /// Trim values before filtering
    pub trim_values: bool,
    /// Chain filters (AND)
    pub chain_filters: bool,
}

impl FilterConfig {
    /// Create new config
    pub fn new(filter_type: FilterType) -> Self {
        Self {
            default_type: filter_type,
            case_insensitive: true,
            trim_values: true,
            chain_filters: true,
        }
    }

    /// Set case insensitive
    pub fn case_insensitive(mut self, ci: bool) -> Self {
        self.case_insensitive = ci;
        self
    }

    /// Set trim values
    pub fn trim_values(mut self, trim: bool) -> Self {
        self.trim_values = trim;
        self
    }

    /// Set chain filters
    pub fn chain_filters(mut self, chain: bool) -> Self {
        self.chain_filters = chain;
        self
    }
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self::new(FilterType::Include)
    }
}

/// Filter rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRule {
    /// Rule ID
    pub id: String,
    /// Predicate
    pub predicate: FilterPredicate,
    /// Pattern (for pattern-based rules)
    pub pattern: Option<String>,
    /// Enabled
    pub enabled: bool,
}

impl FilterRule {
    /// Create predicate rule
    pub fn predicate(id: impl Into<String>, predicate: FilterPredicate) -> Self {
        Self {
            id: id.into(),
            predicate,
            pattern: None,
            enabled: true,
        }
    }

    /// Create pattern rule
    pub fn pattern(id: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            predicate: FilterPredicate::HasValue,
            pattern: Some(pattern.into()),
            enabled: true,
        }
    }

    /// Evaluate rule
    pub fn evaluate(&self, value: &str) -> bool {
        match self.predicate {
            FilterPredicate::IsEmpty => value.is_empty(),
            FilterPredicate::IsNotEmpty => !value.is_empty(),
            FilterPredicate::IsNumeric => value.parse::<f64>().is_ok(),
            FilterPredicate::IsBoolean => value == "true" || value == "false",
            FilterPredicate::HasValue => {
                if let Some(pattern) = &self.pattern {
                    value.contains(pattern)
                } else {
                    !value.is_empty()
                }
            }
        }
    }

    /// Set enabled
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// Filter result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterResult {
    /// Filtered settings
    pub settings: HashMap<String, String>,
    /// Passed count
    pub passed: usize,
    /// Filtered out count
    pub filtered_out: usize,
    /// Rules applied
    pub rules_applied: Vec<String>,
}

impl FilterResult {
    /// Create new result
    pub fn new(settings: HashMap<String, String>, filtered_out: usize) -> Self {
        let passed = settings.len();
        Self {
            settings,
            passed,
            filtered_out,
            rules_applied: Vec::new(),
        }
    }

    /// With rules
    pub fn with_rules(mut self, rules: Vec<String>) -> Self {
        self.rules_applied = rules;
        self
    }

    /// Filter rate
    pub fn filter_rate(&self) -> f64 {
        let total = self.passed + self.filtered_out;
        if total == 0 {
            0.0
        } else {
            self.filtered_out as f64 / total as f64
        }
    }
}

impl Default for FilterResult {
    fn default() -> Self {
        Self::new(HashMap::new(), 0)
    }
}

/// Filter stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilterStats {
    /// Total filters
    pub total_filters: usize,
    /// Total passed
    pub total_passed: usize,
    /// Total filtered
    pub total_filtered: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl FilterStats {
    /// Record filter
    pub fn record(&mut self, result: &FilterResult, filter_type: FilterType) {
        self.total_filters += 1;
        self.total_passed += result.passed;
        self.total_filtered += result.filtered_out;
        *self.by_type.entry(filter_type.to_string()).or_insert(0) += 1;
    }

    /// Average filter rate
    pub fn average_filter_rate(&self) -> f64 {
        let total = self.total_passed + self.total_filtered;
        if total == 0 {
            0.0
        } else {
            self.total_filtered as f64 / total as f64
        }
    }
}
