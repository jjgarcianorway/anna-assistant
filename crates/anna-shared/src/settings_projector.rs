// v0.0.672: Settings Projector (Phase 248)
// Project settings to specific fields/views

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

/// Settings projector
#[derive(Debug, Clone, Default)]
pub struct SettingsProjector {
    /// Config
    config: ProjectorConfig,
    /// Field mappings
    mappings: Vec<FieldMapping>,
    /// Stats
    stats: ProjectorStats,
}

impl SettingsProjector {
    /// Create new projector
    pub fn new(config: ProjectorConfig) -> Self {
        Self {
            config,
            mappings: Vec::new(),
            stats: ProjectorStats::default(),
        }
    }

    /// Add mapping
    pub fn add_mapping(&mut self, mapping: FieldMapping) {
        self.mappings.push(mapping);
    }

    /// Clear mappings
    pub fn clear_mappings(&mut self) {
        self.mappings.clear();
    }

    /// Project include
    pub fn project_include(&mut self, settings: &HashMap<String, String>, fields: &[&str]) -> ProjectionResult {
        let mut result_settings = HashMap::new();
        let mut fields_included = 0;

        for field in fields {
            if let Some(value) = settings.get(*field) {
                result_settings.insert(field.to_string(), value.clone());
                fields_included += 1;
            }
        }

        let result = ProjectionResult::success(result_settings)
            .with_counts(fields_included, 0, 0);
        self.stats.record(&result, ProjectionType::Include);
        result
    }

    /// Project exclude
    pub fn project_exclude(&mut self, settings: &HashMap<String, String>, fields: &[&str]) -> ProjectionResult {
        let exclude_set: std::collections::HashSet<&str> = fields.iter().cloned().collect();
        let mut result_settings = HashMap::new();
        let mut fields_excluded = 0;

        for (key, value) in settings {
            if exclude_set.contains(key.as_str()) {
                fields_excluded += 1;
            } else {
                result_settings.insert(key.clone(), value.clone());
            }
        }

        let result = ProjectionResult::success(result_settings)
            .with_counts(0, fields_excluded, 0);
        self.stats.record(&result, ProjectionType::Exclude);
        result
    }

    /// Project with mappings
    pub fn project(&mut self, settings: &HashMap<String, String>) -> ProjectionResult {
        let mut result_settings = HashMap::new();
        let mut fields_renamed = 0;
        let mut fields_included = 0;

        for mapping in &self.mappings {
            if let Some(value) = settings.get(&mapping.source) {
                let target_key = mapping.target.clone().unwrap_or_else(|| mapping.source.clone());
                if mapping.target.is_some() {
                    fields_renamed += 1;
                } else {
                    fields_included += 1;
                }
                result_settings.insert(target_key, value.clone());
            }
        }

        let result = ProjectionResult::success(result_settings)
            .with_counts(fields_included, 0, fields_renamed);
        self.stats.record(&result, self.config.default_type);
        result
    }

    /// Get stats
    pub fn stats(&self) -> &ProjectorStats {
        &self.stats
    }

    /// Mapping count
    pub fn mapping_count(&self) -> usize {
        self.mappings.len()
    }
}

/// Projector registry
#[derive(Debug, Clone, Default)]
pub struct ProjectorRegistry {
    /// Projectors by ID
    projectors: HashMap<String, SettingsProjector>,
}

impl ProjectorRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register projector
    pub fn register(&mut self, id: impl Into<String>, projector: SettingsProjector) {
        self.projectors.insert(id.into(), projector);
    }

    /// Unregister projector
    pub fn unregister(&mut self, id: &str) -> bool {
        self.projectors.remove(id).is_some()
    }

    /// Get projector
    pub fn get(&self, id: &str) -> Option<&SettingsProjector> {
        self.projectors.get(id)
    }

    /// Get projector mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsProjector> {
        self.projectors.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.projectors.len()
    }
}

/// Format projector registry
pub fn format_projector_registry(registry: &ProjectorRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Projector Registry:\n");
    output.push_str(&format!("  Projectors: {}\n", registry.count()));
    output
}

/// Check if query is about projector
pub fn is_projector_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("project") || lower.contains("select fields") || lower.contains("field projection")
}

/// Fun fact about projector
pub fn projector_fun_fact() -> &'static str {
    "Anna's settings projector creates custom views with only the fields you need!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projection_type_display() {
        assert_eq!(format!("{}", ProjectionType::Include), "include");
        assert_eq!(format!("{}", ProjectionType::Exclude), "exclude");
    }

    #[test]
    fn test_mapping_include() {
        let m = FieldMapping::include("field");
        assert_eq!(m.source, "field");
        assert!(m.target.is_none());
    }

    #[test]
    fn test_mapping_rename() {
        let m = FieldMapping::rename("old", "new");
        assert_eq!(m.source, "old");
        assert_eq!(m.target, Some("new".to_string()));
    }

    #[test]
    fn test_config_new() {
        let c = ProjectorConfig::new(ProjectionType::Include);
        assert!(c.preserve_order);
    }

    #[test]
    fn test_config_builder() {
        let c = ProjectorConfig::new(ProjectionType::Exclude)
            .include_unmatched(true);
        assert!(c.include_unmatched);
    }

    #[test]
    fn test_result_success() {
        let r = ProjectionResult::success(HashMap::new());
        assert!(r.success);
    }

    #[test]
    fn test_result_with_counts() {
        let r = ProjectionResult::success(HashMap::new())
            .with_counts(5, 3, 2);
        assert_eq!(r.total_changes(), 10);
    }

    #[test]
    fn test_stats_record() {
        let mut s = ProjectorStats::default();
        let mut settings = HashMap::new();
        settings.insert("k".to_string(), "v".to_string());
        let r = ProjectionResult::success(settings);
        s.record(&r, ProjectionType::Include);
        assert_eq!(s.total_projections, 1);
    }

    #[test]
    fn test_projector_new() {
        let p = SettingsProjector::new(ProjectorConfig::default());
        assert_eq!(p.mapping_count(), 0);
    }

    #[test]
    fn test_projector_include() {
        let mut p = SettingsProjector::new(ProjectorConfig::default());
        let mut settings = HashMap::new();
        settings.insert("a".to_string(), "1".to_string());
        settings.insert("b".to_string(), "2".to_string());
        settings.insert("c".to_string(), "3".to_string());
        
        let result = p.project_include(&settings, &["a", "b"]);
        assert_eq!(result.settings.len(), 2);
        assert!(result.settings.contains_key("a"));
    }

    #[test]
    fn test_projector_exclude() {
        let mut p = SettingsProjector::new(ProjectorConfig::default());
        let mut settings = HashMap::new();
        settings.insert("a".to_string(), "1".to_string());
        settings.insert("b".to_string(), "2".to_string());
        settings.insert("c".to_string(), "3".to_string());
        
        let result = p.project_exclude(&settings, &["c"]);
        assert_eq!(result.settings.len(), 2);
        assert!(!result.settings.contains_key("c"));
    }

    #[test]
    fn test_projector_with_mappings() {
        let mut p = SettingsProjector::new(ProjectorConfig::default());
        p.add_mapping(FieldMapping::rename("old_name", "new_name"));
        
        let mut settings = HashMap::new();
        settings.insert("old_name".to_string(), "value".to_string());
        
        let result = p.project(&settings);
        assert!(result.settings.contains_key("new_name"));
    }

    #[test]
    fn test_registry_new() {
        let r = ProjectorRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ProjectorRegistry::new();
        r.register("p1", SettingsProjector::new(ProjectorConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_projector_query() {
        assert!(is_projector_query("project settings"));
        assert!(!is_projector_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = projector_fun_fact();
        assert!(fact.contains("projector"));
    }
}
