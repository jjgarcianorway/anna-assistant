// v0.0.672: Settings Projector Types (Phase 248)
// Core types for settings projection

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Projection type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ProjectionType {
    /// Include specified fields
    #[default]
    Include,
    /// Exclude specified fields
    Exclude,
    /// Rename fields
    Rename,
    /// Compute new fields
    Compute,
}

impl std::fmt::Display for ProjectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Include => write!(f, "include"),
            Self::Exclude => write!(f, "exclude"),
            Self::Rename => write!(f, "rename"),
            Self::Compute => write!(f, "compute"),
        }
    }
}

/// Field mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMapping {
    /// Source field
    pub source: String,
    /// Target field (for rename)
    pub target: Option<String>,
    /// Transform expression
    pub transform: Option<String>,
}

impl FieldMapping {
    /// Create include mapping
    pub fn include(field: impl Into<String>) -> Self {
        Self {
            source: field.into(),
            target: None,
            transform: None,
        }
    }

    /// Create rename mapping
    pub fn rename(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            target: Some(target.into()),
            transform: None,
        }
    }

    /// With transform
    pub fn with_transform(mut self, transform: impl Into<String>) -> Self {
        self.transform = Some(transform.into());
        self
    }
}

/// Projector config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectorConfig {
    /// Default projection type
    pub default_type: ProjectionType,
    /// Preserve order
    pub preserve_order: bool,
    /// Include unmatched
    pub include_unmatched: bool,
}

impl ProjectorConfig {
    /// Create new config
    pub fn new(projection_type: ProjectionType) -> Self {
        Self {
            default_type: projection_type,
            preserve_order: true,
            include_unmatched: false,
        }
    }

    /// Set preserve order
    pub fn preserve_order(mut self, preserve: bool) -> Self {
        self.preserve_order = preserve;
        self
    }

    /// Set include unmatched
    pub fn include_unmatched(mut self, include: bool) -> Self {
        self.include_unmatched = include;
        self
    }
}

impl Default for ProjectorConfig {
    fn default() -> Self {
        Self::new(ProjectionType::Include)
    }
}

/// Projection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionResult {
    /// Projected settings
    pub settings: HashMap<String, String>,
    /// Fields included
    pub fields_included: usize,
    /// Fields excluded
    pub fields_excluded: usize,
    /// Fields renamed
    pub fields_renamed: usize,
    /// Success
    pub success: bool,
}

impl ProjectionResult {
    /// Create success result
    pub fn success(settings: HashMap<String, String>) -> Self {
        Self {
            settings,
            fields_included: 0,
            fields_excluded: 0,
            fields_renamed: 0,
            success: true,
        }
    }

    /// With counts
    pub fn with_counts(mut self, included: usize, excluded: usize, renamed: usize) -> Self {
        self.fields_included = included;
        self.fields_excluded = excluded;
        self.fields_renamed = renamed;
        self
    }

    /// Total changes
    pub fn total_changes(&self) -> usize {
        self.fields_included + self.fields_excluded + self.fields_renamed
    }
}

impl Default for ProjectionResult {
    fn default() -> Self {
        Self::success(HashMap::new())
    }
}

/// Projector stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectorStats {
    /// Total projections
    pub total_projections: usize,
    /// Total fields processed
    pub total_fields: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ProjectorStats {
    /// Record projection
    pub fn record(&mut self, result: &ProjectionResult, proj_type: ProjectionType) {
        self.total_projections += 1;
        self.total_fields += result.settings.len();
        *self.by_type.entry(proj_type.to_string()).or_insert(0) += 1;
    }

    /// Fields per projection
    pub fn fields_per_projection(&self) -> f64 {
        if self.total_projections == 0 {
            0.0
        } else {
            self.total_fields as f64 / self.total_projections as f64
        }
    }
}
