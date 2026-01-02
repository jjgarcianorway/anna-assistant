// v0.0.675: Settings Sorter Types (Phase 251)
// Sort order and field enums

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Sort order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SortOrder {
    /// Ascending order
    #[default]
    Ascending,
    /// Descending order
    Descending,
    /// Natural order (human-friendly)
    Natural,
    /// Reverse order
    Reverse,
}

impl std::fmt::Display for SortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ascending => write!(f, "ascending"),
            Self::Descending => write!(f, "descending"),
            Self::Natural => write!(f, "natural"),
            Self::Reverse => write!(f, "reverse"),
        }
    }
}

/// Sort field
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SortField {
    /// Sort by key
    #[default]
    Key,
    /// Sort by value
    Value,
    /// Sort by key length
    KeyLength,
    /// Sort by value length
    ValueLength,
}

impl std::fmt::Display for SortField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Key => write!(f, "key"),
            Self::Value => write!(f, "value"),
            Self::KeyLength => write!(f, "key_length"),
            Self::ValueLength => write!(f, "value_length"),
        }
    }
}

/// Sort result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortResult {
    /// Sorted entries
    pub entries: Vec<(String, String)>,
    /// Total sorted
    pub total_sorted: usize,
    /// Criteria used
    pub criteria_used: Vec<super::config::SortCriteria>,
}

impl SortResult {
    /// Create new result
    pub fn new(entries: Vec<(String, String)>) -> Self {
        let total_sorted = entries.len();
        Self {
            entries,
            total_sorted,
            criteria_used: Vec::new(),
        }
    }

    /// With criteria
    pub fn with_criteria(mut self, criteria: Vec<super::config::SortCriteria>) -> Self {
        self.criteria_used = criteria;
        self
    }

    /// Is sorted
    pub fn is_sorted(&self) -> bool {
        !self.entries.is_empty()
    }

    /// Get keys
    pub fn keys(&self) -> Vec<&str> {
        self.entries.iter().map(|(k, _)| k.as_str()).collect()
    }
}

impl Default for SortResult {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

/// Sorter stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SorterStats {
    /// Total sorts
    pub total_sorts: usize,
    /// Total entries sorted
    pub total_entries: usize,
    /// By order
    pub by_order: HashMap<String, usize>,
    /// By field
    pub by_field: HashMap<String, usize>,
}

impl SorterStats {
    /// Record sort
    pub fn record(&mut self, result: &SortResult, order: SortOrder, field: SortField) {
        self.total_sorts += 1;
        self.total_entries += result.total_sorted;
        *self.by_order.entry(order.to_string()).or_insert(0) += 1;
        *self.by_field.entry(field.to_string()).or_insert(0) += 1;
    }

    /// Average entries per sort
    pub fn average_entries(&self) -> f64 {
        if self.total_sorts == 0 {
            0.0
        } else {
            self.total_entries as f64 / self.total_sorts as f64
        }
    }
}
