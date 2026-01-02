// v0.0.675: Settings Sorter Config (Phase 251)
// Configuration and criteria for sorting

use serde::{Deserialize, Serialize};
use super::types::{SortField, SortOrder};

/// Sorter config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SorterConfig {
    /// Default sort order
    pub default_order: SortOrder,
    /// Default sort field
    pub default_field: SortField,
    /// Case insensitive
    pub case_insensitive: bool,
    /// Stable sort
    pub stable_sort: bool,
}

impl SorterConfig {
    /// Create new config
    pub fn new(order: SortOrder) -> Self {
        Self {
            default_order: order,
            default_field: SortField::Key,
            case_insensitive: true,
            stable_sort: true,
        }
    }

    /// Set sort field
    pub fn field(mut self, field: SortField) -> Self {
        self.default_field = field;
        self
    }

    /// Set case insensitive
    pub fn case_insensitive(mut self, ci: bool) -> Self {
        self.case_insensitive = ci;
        self
    }

    /// Set stable sort
    pub fn stable_sort(mut self, stable: bool) -> Self {
        self.stable_sort = stable;
        self
    }
}

impl Default for SorterConfig {
    fn default() -> Self {
        Self::new(SortOrder::Ascending)
    }
}

/// Sort criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortCriteria {
    /// Sort field
    pub field: SortField,
    /// Sort order
    pub order: SortOrder,
    /// Priority (for multi-field sorting)
    pub priority: u8,
}

impl SortCriteria {
    /// Create new criteria
    pub fn new(field: SortField, order: SortOrder) -> Self {
        Self {
            field,
            order,
            priority: 0,
        }
    }

    /// With priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Compare two entries
    pub fn compare(&self, a: (&str, &str), b: (&str, &str), case_insensitive: bool) -> std::cmp::Ordering {
        let (a_key, a_val) = a;
        let (b_key, b_val) = b;

        let ordering = match self.field {
            SortField::Key => {
                if case_insensitive {
                    a_key.to_lowercase().cmp(&b_key.to_lowercase())
                } else {
                    a_key.cmp(b_key)
                }
            }
            SortField::Value => {
                if case_insensitive {
                    a_val.to_lowercase().cmp(&b_val.to_lowercase())
                } else {
                    a_val.cmp(b_val)
                }
            }
            SortField::KeyLength => a_key.len().cmp(&b_key.len()),
            SortField::ValueLength => a_val.len().cmp(&b_val.len()),
        };

        match self.order {
            SortOrder::Ascending | SortOrder::Natural => ordering,
            SortOrder::Descending | SortOrder::Reverse => ordering.reverse(),
        }
    }
}
