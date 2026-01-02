// v0.0.599: Settings Resolver Types (Phase 175)
// Type definitions for conflict resolution

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

/// Conflict type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictType {
    /// Value mismatch
    ValueMismatch,
    /// Type mismatch
    TypeMismatch,
    /// Dependency missing
    DependencyMissing,
    /// Circular dependency
    CircularDep,
    /// Mutual exclusion
    MutualExclusion,
    /// Version conflict
    VersionConflict,
}

impl std::fmt::Display for ConflictType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ValueMismatch => write!(f, "value_mismatch"),
            Self::TypeMismatch => write!(f, "type_mismatch"),
            Self::DependencyMissing => write!(f, "dependency_missing"),
            Self::CircularDep => write!(f, "circular_dependency"),
            Self::MutualExclusion => write!(f, "mutual_exclusion"),
            Self::VersionConflict => write!(f, "version_conflict"),
        }
    }
}

/// Resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    /// Use first value
    First,
    /// Use last value
    Last,
    /// Use highest priority
    Priority,
    /// Merge values
    Merge,
    /// Fail on conflict
    Fail,
    /// Ask user
    Ask,
}

impl std::fmt::Display for ResolutionStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::First => write!(f, "first"),
            Self::Last => write!(f, "last"),
            Self::Priority => write!(f, "priority"),
            Self::Merge => write!(f, "merge"),
            Self::Fail => write!(f, "fail"),
            Self::Ask => write!(f, "ask"),
        }
    }
}

/// Conflict description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    /// Conflict type
    pub conflict_type: ConflictType,
    /// Source A
    pub source_a: String,
    /// Source B
    pub source_b: String,
    /// Affected key
    pub key: String,
    /// Category
    pub category: SettingsCategory,
    /// Description
    pub description: String,
}

impl Conflict {
    /// Create new conflict
    pub fn new(
        conflict_type: ConflictType,
        source_a: impl Into<String>,
        source_b: impl Into<String>,
        key: impl Into<String>,
        category: SettingsCategory,
    ) -> Self {
        Self {
            conflict_type,
            source_a: source_a.into(),
            source_b: source_b.into(),
            key: key.into(),
            category,
            description: String::new(),
        }
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
}

/// Resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    /// Conflict
    pub conflict: Conflict,
    /// Strategy used
    pub strategy: ResolutionStrategy,
    /// Resolved value
    pub resolved_value: Option<String>,
    /// Success
    pub success: bool,
    /// Notes
    pub notes: String,
}

impl Resolution {
    /// Create successful resolution
    pub fn success(conflict: Conflict, strategy: ResolutionStrategy, value: impl Into<String>) -> Self {
        Self {
            conflict,
            strategy,
            resolved_value: Some(value.into()),
            success: true,
            notes: String::new(),
        }
    }

    /// Create failed resolution
    pub fn failure(conflict: Conflict, notes: impl Into<String>) -> Self {
        Self {
            conflict,
            strategy: ResolutionStrategy::Fail,
            resolved_value: None,
            success: false,
            notes: notes.into(),
        }
    }
}

/// Dependency entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Source key
    pub source: String,
    /// Depends on
    pub depends_on: String,
    /// Required
    pub required: bool,
    /// Category
    pub category: SettingsCategory,
}

impl Dependency {
    /// Create new dependency
    pub fn new(
        source: impl Into<String>,
        depends_on: impl Into<String>,
        category: SettingsCategory,
    ) -> Self {
        Self {
            source: source.into(),
            depends_on: depends_on.into(),
            required: true,
            category,
        }
    }

    /// Set optional
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }
}
