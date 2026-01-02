// v0.0.598: Settings Transformer Manager (Phase 174)
// Manages multiple transform pipelines

use std::collections::HashMap;

use super::pipeline::TransformPipeline;

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
