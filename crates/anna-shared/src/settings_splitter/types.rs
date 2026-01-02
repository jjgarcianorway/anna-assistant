// v0.0.656: Settings Splitter Types (Phase 232)
// Type definitions for settings splitter

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Split criteria
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SplitCriteria {
    /// By category
    #[default]
    ByCategory,
    /// By prefix
    ByPrefix,
    /// By pattern
    ByPattern,
    /// By value type
    ByValueType,
    /// By size
    BySize,
}

impl std::fmt::Display for SplitCriteria {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ByCategory => write!(f, "by_category"),
            Self::ByPrefix => write!(f, "by_prefix"),
            Self::ByPattern => write!(f, "by_pattern"),
            Self::ByValueType => write!(f, "by_value_type"),
            Self::BySize => write!(f, "by_size"),
        }
    }
}

/// Split mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SplitMode {
    /// Even distribution
    #[default]
    Even,
    /// By threshold
    ByThreshold,
    /// By count
    ByCount,
    /// Custom
    Custom,
}

impl std::fmt::Display for SplitMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Even => write!(f, "even"),
            Self::ByThreshold => write!(f, "by_threshold"),
            Self::ByCount => write!(f, "by_count"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

/// Splitter config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitterConfig {
    /// Split criteria
    pub criteria: SplitCriteria,
    /// Split mode
    pub mode: SplitMode,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Max groups
    pub max_groups: usize,
    /// Preserve order
    pub preserve_order: bool,
}

impl SplitterConfig {
    /// Create new config
    pub fn new(criteria: SplitCriteria) -> Self {
        Self {
            criteria,
            mode: SplitMode::Even,
            category: None,
            max_groups: 10,
            preserve_order: true,
        }
    }

    /// Set mode
    pub fn mode(mut self, mode: SplitMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set max groups
    pub fn max_groups(mut self, max: usize) -> Self {
        self.max_groups = max;
        self
    }

    /// Set preserve order
    pub fn preserve_order(mut self, preserve: bool) -> Self {
        self.preserve_order = preserve;
        self
    }
}

impl Default for SplitterConfig {
    fn default() -> Self {
        Self::new(SplitCriteria::ByCategory)
    }
}

/// Split group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitGroup {
    /// Group name
    pub name: String,
    /// Settings in this group
    pub settings: HashMap<String, String>,
    /// Criteria value
    pub criteria_value: String,
}

impl SplitGroup {
    /// Create new group
    pub fn new(name: impl Into<String>, criteria_value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            settings: HashMap::new(),
            criteria_value: criteria_value.into(),
        }
    }

    /// Add setting
    pub fn add(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.settings.insert(key.into(), value.into());
    }

    /// Setting count
    pub fn setting_count(&self) -> usize {
        self.settings.len()
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.settings.is_empty()
    }
}

/// Split result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitResult {
    /// Groups created
    pub groups: Vec<SplitGroup>,
    /// Total keys split
    pub total_keys: usize,
    /// Criteria used
    pub criteria: SplitCriteria,
    /// Unmatched keys
    pub unmatched: Vec<String>,
}

impl SplitResult {
    /// Create new result
    pub fn new(criteria: SplitCriteria) -> Self {
        Self {
            groups: Vec::new(),
            total_keys: 0,
            criteria,
            unmatched: Vec::new(),
        }
    }

    /// Add group
    pub fn add_group(&mut self, group: SplitGroup) {
        self.total_keys += group.setting_count();
        self.groups.push(group);
    }

    /// Add unmatched
    pub fn add_unmatched(&mut self, key: String) {
        self.unmatched.push(key);
    }

    /// Group count
    pub fn group_count(&self) -> usize {
        self.groups.len()
    }

    /// Has unmatched
    pub fn has_unmatched(&self) -> bool {
        !self.unmatched.is_empty()
    }
}

/// Splitter stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SplitterStats {
    /// Total splits
    pub total_splits: usize,
    /// Total groups created
    pub total_groups: usize,
    /// Total keys split
    pub total_keys_split: usize,
    /// By criteria
    pub by_criteria: HashMap<String, usize>,
}

impl SplitterStats {
    /// Record split
    pub fn record(&mut self, criteria: SplitCriteria, groups: usize, keys: usize) {
        self.total_splits += 1;
        self.total_groups += groups;
        self.total_keys_split += keys;
        *self.by_criteria.entry(criteria.to_string()).or_insert(0) += 1;
    }

    /// Average group size
    pub fn average_group_size(&self) -> f64 {
        if self.total_groups == 0 {
            0.0
        } else {
            self.total_keys_split as f64 / self.total_groups as f64
        }
    }
}
