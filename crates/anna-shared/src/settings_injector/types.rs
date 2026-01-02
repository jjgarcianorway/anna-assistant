// v0.0.654: Settings Injector Types
// Type definitions for settings injection

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Injection type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum InjectionType {
    /// Insert new value
    #[default]
    Insert,
    /// Update existing value
    Update,
    /// Upsert (insert or update)
    Upsert,
    /// Replace all
    Replace,
    /// Append to existing
    Append,
}

impl std::fmt::Display for InjectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Insert => write!(f, "insert"),
            Self::Update => write!(f, "update"),
            Self::Upsert => write!(f, "upsert"),
            Self::Replace => write!(f, "replace"),
            Self::Append => write!(f, "append"),
        }
    }
}

/// Injection strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum InjectionStrategy {
    /// Fail on conflict
    #[default]
    FailOnConflict,
    /// Skip on conflict
    SkipOnConflict,
    /// Overwrite on conflict
    OverwriteOnConflict,
    /// Merge on conflict
    MergeOnConflict,
}

impl std::fmt::Display for InjectionStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FailOnConflict => write!(f, "fail_on_conflict"),
            Self::SkipOnConflict => write!(f, "skip_on_conflict"),
            Self::OverwriteOnConflict => write!(f, "overwrite_on_conflict"),
            Self::MergeOnConflict => write!(f, "merge_on_conflict"),
        }
    }
}

/// Injector config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectorConfig {
    /// Injection type
    pub injection_type: InjectionType,
    /// Injection strategy
    pub strategy: InjectionStrategy,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Validate before inject
    pub validate_before: bool,
    /// Dry run mode
    pub dry_run: bool,
}

impl InjectorConfig {
    /// Create new config
    pub fn new(injection_type: InjectionType) -> Self {
        Self {
            injection_type,
            strategy: InjectionStrategy::FailOnConflict,
            category: None,
            validate_before: true,
            dry_run: false,
        }
    }

    /// Set strategy
    pub fn strategy(mut self, strategy: InjectionStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set validate before
    pub fn validate_before(mut self, validate: bool) -> Self {
        self.validate_before = validate;
        self
    }

    /// Set dry run
    pub fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }
}

impl Default for InjectorConfig {
    fn default() -> Self {
        Self::new(InjectionType::Upsert)
    }
}

/// Injection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionResult {
    /// Keys inserted
    pub inserted: Vec<String>,
    /// Keys updated
    pub updated: Vec<String>,
    /// Keys skipped
    pub skipped: Vec<String>,
    /// Keys failed
    pub failed: Vec<String>,
    /// Injection type used
    pub injection_type: InjectionType,
}

impl InjectionResult {
    /// Create new result
    pub fn new(injection_type: InjectionType) -> Self {
        Self {
            inserted: Vec::new(),
            updated: Vec::new(),
            skipped: Vec::new(),
            failed: Vec::new(),
            injection_type,
        }
    }

    /// Add inserted
    pub fn add_inserted(&mut self, key: String) {
        self.inserted.push(key);
    }

    /// Add updated
    pub fn add_updated(&mut self, key: String) {
        self.updated.push(key);
    }

    /// Add skipped
    pub fn add_skipped(&mut self, key: String) {
        self.skipped.push(key);
    }

    /// Add failed
    pub fn add_failed(&mut self, key: String) {
        self.failed.push(key);
    }

    /// Total affected
    pub fn total_affected(&self) -> usize {
        self.inserted.len() + self.updated.len()
    }

    /// Has failures
    pub fn has_failures(&self) -> bool {
        !self.failed.is_empty()
    }
}

/// Injector stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InjectorStats {
    /// Total injections
    pub total_injections: usize,
    /// Total inserted
    pub total_inserted: usize,
    /// Total updated
    pub total_updated: usize,
    /// Total skipped
    pub total_skipped: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl InjectorStats {
    /// Record injection
    pub fn record(&mut self, injection_type: InjectionType, inserted: usize, updated: usize, skipped: usize) {
        self.total_injections += 1;
        self.total_inserted += inserted;
        self.total_updated += updated;
        self.total_skipped += skipped;
        *self.by_type.entry(injection_type.to_string()).or_insert(0) += 1;
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.total_inserted + self.total_updated + self.total_skipped;
        if total == 0 {
            0.0
        } else {
            (self.total_inserted + self.total_updated) as f64 / total as f64
        }
    }
}
