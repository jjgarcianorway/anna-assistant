// v0.0.598: Settings Transformer (Phase 174)
// Transformation pipeline for settings values

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Transform type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransformType {
    /// Trim whitespace
    Trim,
    /// To lowercase
    Lower,
    /// To uppercase
    Upper,
    /// Default value
    Default,
    /// Replace pattern
    Replace,
    /// Clamp to range
    Clamp,
    /// Custom function
    Custom,
}

impl std::fmt::Display for TransformType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trim => write!(f, "trim"),
            Self::Lower => write!(f, "lower"),
            Self::Upper => write!(f, "upper"),
            Self::Default => write!(f, "default"),
            Self::Replace => write!(f, "replace"),
            Self::Clamp => write!(f, "clamp"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

/// Transform direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransformDirection {
    /// On input (before storing)
    Input,
    /// On output (after reading)
    Output,
    /// Both directions
    Both,
}

impl std::fmt::Display for TransformDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Input => write!(f, "input"),
            Self::Output => write!(f, "output"),
            Self::Both => write!(f, "both"),
        }
    }
}

/// Transform definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformDef {
    /// Unique ID
    pub id: String,
    /// Transform type
    pub transform_type: TransformType,
    /// Direction
    pub direction: TransformDirection,
    /// Name
    pub name: String,
    /// Parameters
    pub params: HashMap<String, String>,
    /// Target categories
    pub categories: Vec<SettingsCategory>,
    /// Priority (lower runs first)
    pub priority: i32,
    /// Enabled
    pub enabled: bool,
}

impl TransformDef {
    /// Create new transform
    pub fn new(id: impl Into<String>, transform_type: TransformType) -> Self {
        Self {
            id: id.into(),
            transform_type,
            direction: TransformDirection::Both,
            name: String::new(),
            params: HashMap::new(),
            categories: Vec::new(),
            priority: 100,
            enabled: true,
        }
    }

    /// Set name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set direction
    pub fn direction(mut self, dir: TransformDirection) -> Self {
        self.direction = dir;
        self
    }

    /// Add parameter
    pub fn param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }

    /// Add category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.categories.push(category);
        self
    }

    /// Set priority
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Enable/disable
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Check if applies to category
    pub fn applies_to(&self, category: SettingsCategory) -> bool {
        self.categories.is_empty() || self.categories.contains(&category)
    }

    /// Check if applies to direction
    pub fn applies_to_direction(&self, dir: TransformDirection) -> bool {
        self.direction == TransformDirection::Both || self.direction == dir
    }
}

/// Transform result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformResult {
    /// Original value
    pub original: String,
    /// Transformed value
    pub transformed: String,
    /// Applied transforms
    pub applied: Vec<String>,
    /// Success
    pub success: bool,
}

impl TransformResult {
    /// Create new result
    pub fn new(original: impl Into<String>, transformed: impl Into<String>) -> Self {
        Self {
            original: original.into(),
            transformed: transformed.into(),
            applied: Vec::new(),
            success: true,
        }
    }

    /// Add applied transform
    pub fn add_applied(&mut self, id: impl Into<String>) {
        self.applied.push(id.into());
    }

    /// Mark as failed
    pub fn fail(mut self) -> Self {
        self.success = false;
        self
    }

    /// Was transformed
    pub fn was_transformed(&self) -> bool {
        self.original != self.transformed
    }
}

/// Transform pipeline
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransformPipeline {
    /// Transforms
    transforms: Vec<TransformDef>,
}

impl TransformPipeline {
    /// Create new pipeline
    pub fn new() -> Self {
        Self::default()
    }

    /// Add transform
    pub fn add(&mut self, transform: TransformDef) {
        self.transforms.push(transform);
        self.transforms.sort_by_key(|t| t.priority);
    }

    /// Remove transform
    pub fn remove(&mut self, id: &str) -> Option<TransformDef> {
        if let Some(pos) = self.transforms.iter().position(|t| t.id == id) {
            Some(self.transforms.remove(pos))
        } else {
            None
        }
    }

    /// Get transform
    pub fn get(&self, id: &str) -> Option<&TransformDef> {
        self.transforms.iter().find(|t| t.id == id)
    }

    /// Enable transform
    pub fn enable(&mut self, id: &str) -> bool {
        if let Some(t) = self.transforms.iter_mut().find(|t| t.id == id) {
            t.enabled = true;
            true
        } else {
            false
        }
    }

    /// Disable transform
    pub fn disable(&mut self, id: &str) -> bool {
        if let Some(t) = self.transforms.iter_mut().find(|t| t.id == id) {
            t.enabled = false;
            true
        } else {
            false
        }
    }

    /// Get transforms for category and direction
    pub fn for_category_dir(
        &self,
        category: SettingsCategory,
        dir: TransformDirection,
    ) -> Vec<&TransformDef> {
        self.transforms
            .iter()
            .filter(|t| t.enabled && t.applies_to(category) && t.applies_to_direction(dir))
            .collect()
    }

    /// Count transforms
    pub fn count(&self) -> usize {
        self.transforms.len()
    }

    /// Count enabled
    pub fn enabled_count(&self) -> usize {
        self.transforms.iter().filter(|t| t.enabled).count()
    }
}

/// Transformer manager
#[derive(Debug, Clone, Default)]
pub struct TransformerManager {
    /// Named pipelines
    pipelines: HashMap<String, TransformPipeline>,
    /// Default pipeline
    default_pipeline: TransformPipeline,
}

impl TransformerManager {
    /// Create new manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Add pipeline
    pub fn add_pipeline(&mut self, name: impl Into<String>, pipeline: TransformPipeline) {
        self.pipelines.insert(name.into(), pipeline);
    }

    /// Get pipeline
    pub fn get_pipeline(&self, name: &str) -> Option<&TransformPipeline> {
        self.pipelines.get(name)
    }

    /// Get pipeline mut
    pub fn get_pipeline_mut(&mut self, name: &str) -> Option<&mut TransformPipeline> {
        self.pipelines.get_mut(name)
    }

    /// Remove pipeline
    pub fn remove_pipeline(&mut self, name: &str) -> Option<TransformPipeline> {
        self.pipelines.remove(name)
    }

    /// Set default pipeline
    pub fn set_default(&mut self, pipeline: TransformPipeline) {
        self.default_pipeline = pipeline;
    }

    /// Get default pipeline
    pub fn default_pipeline(&self) -> &TransformPipeline {
        &self.default_pipeline
    }

    /// List pipeline names
    pub fn pipeline_names(&self) -> Vec<&String> {
        self.pipelines.keys().collect()
    }

    /// Pipeline count
    pub fn pipeline_count(&self) -> usize {
        self.pipelines.len()
    }
}

/// Format transform pipeline
pub fn format_transform_pipeline(pipeline: &TransformPipeline) -> String {
    let mut output = String::new();
    output.push_str("Transform Pipeline:\n");
    output.push_str(&format!("  Transforms: {}\n", pipeline.count()));
    output.push_str(&format!("  Enabled: {}\n", pipeline.enabled_count()));

    for t in &pipeline.transforms {
        let status = if t.enabled { "✓" } else { "✗" };
        output.push_str(&format!(
            "  {} [{}] {} ({}, {})\n",
            status, t.transform_type, t.name, t.direction, t.priority
        ));
    }

    output
}

/// Check if query is about transformer
pub fn is_transformer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("transform")
        || lower.contains("convert settings")
        || lower.contains("normalize")
}

/// Fun fact about transformers
pub fn transformer_fun_fact() -> &'static str {
    "Anna uses transform pipelines to normalize and convert settings values automatically!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_type_display() {
        assert_eq!(format!("{}", TransformType::Trim), "trim");
        assert_eq!(format!("{}", TransformType::Upper), "upper");
    }

    #[test]
    fn test_transform_direction_display() {
        assert_eq!(format!("{}", TransformDirection::Input), "input");
        assert_eq!(format!("{}", TransformDirection::Both), "both");
    }

    #[test]
    fn test_transform_def_new() {
        let t = TransformDef::new("t1", TransformType::Trim);
        assert_eq!(t.id, "t1");
        assert!(t.enabled);
    }

    #[test]
    fn test_transform_def_builder() {
        let t = TransformDef::new("t1", TransformType::Replace)
            .name("Replacer")
            .direction(TransformDirection::Input)
            .param("from", "a")
            .param("to", "b")
            .priority(50);
        assert_eq!(t.priority, 50);
        assert_eq!(t.params.len(), 2);
    }

    #[test]
    fn test_transform_result_new() {
        let r = TransformResult::new("hello", "HELLO");
        assert!(r.was_transformed());
        assert!(r.success);
    }

    #[test]
    fn test_transform_result_not_transformed() {
        let r = TransformResult::new("same", "same");
        assert!(!r.was_transformed());
    }

    #[test]
    fn test_pipeline_new() {
        let p = TransformPipeline::new();
        assert_eq!(p.count(), 0);
    }

    #[test]
    fn test_pipeline_add_remove() {
        let mut p = TransformPipeline::new();
        p.add(TransformDef::new("t1", TransformType::Trim));
        assert_eq!(p.count(), 1);
        p.remove("t1");
        assert_eq!(p.count(), 0);
    }

    #[test]
    fn test_pipeline_enable_disable() {
        let mut p = TransformPipeline::new();
        p.add(TransformDef::new("t1", TransformType::Lower));
        p.disable("t1");
        assert_eq!(p.enabled_count(), 0);
        p.enable("t1");
        assert_eq!(p.enabled_count(), 1);
    }

    #[test]
    fn test_manager_new() {
        let m = TransformerManager::new();
        assert_eq!(m.pipeline_count(), 0);
    }

    #[test]
    fn test_manager_add_pipeline() {
        let mut m = TransformerManager::new();
        m.add_pipeline("test", TransformPipeline::new());
        assert_eq!(m.pipeline_count(), 1);
    }

    #[test]
    fn test_is_transformer_query() {
        assert!(is_transformer_query("transform settings"));
        assert!(!is_transformer_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = transformer_fun_fact();
        assert!(fact.contains("transform"));
    }
}
