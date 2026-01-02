// v0.0.672: Settings Projector Core (Phase 248)
// Main projector implementation

use super::types::{FieldMapping, ProjectionResult, ProjectionType, ProjectorConfig, ProjectorStats};
use std::collections::HashMap;

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
