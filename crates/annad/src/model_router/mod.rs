//! Model Router - Routes tasks to appropriate models based on complexity.
//!
//! Selects the optimal model for each task based on:
//! - Task complexity (simple, standard, complex, very complex)
//! - Agent model tier preference (fast, standard, deep)
//! - Available hardware resources

mod complexity;

pub use complexity::{Complexity, ComplexityClassifier};

use anna_shared::agent::{AgentTask, ModelTier};
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Model information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model name (e.g., "qwen2.5:7b")
    pub name: String,
    /// Model tier
    pub tier: ModelTier,
    /// Context window size
    pub context_length: u32,
    /// Estimated tokens per second
    pub avg_tokens_per_sec: f32,
    /// Memory required in GB
    pub memory_required_gb: f32,
}

impl ModelInfo {
    pub fn fast(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tier: ModelTier::Fast,
            context_length: 32768,
            avg_tokens_per_sec: 50.0,
            memory_required_gb: 4.0,
        }
    }

    pub fn standard(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tier: ModelTier::Standard,
            context_length: 32768,
            avg_tokens_per_sec: 30.0,
            memory_required_gb: 8.0,
        }
    }

    pub fn deep(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tier: ModelTier::Deep,
            context_length: 32768,
            avg_tokens_per_sec: 15.0,
            memory_required_gb: 20.0,
        }
    }
}

/// Model configuration from config.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMappings {
    /// Fast model for simple queries
    #[serde(default = "default_fast_model")]
    pub fast: String,
    /// Standard model for balanced tasks
    #[serde(default = "default_standard_model")]
    pub standard: String,
    /// Deep model for complex debugging
    #[serde(default = "default_deep_model")]
    pub deep: String,
}

fn default_fast_model() -> String { "qwen2.5:7b".to_string() }
fn default_standard_model() -> String { "qwen2.5:14b".to_string() }
fn default_deep_model() -> String { "qwen2.5:32b".to_string() }

impl Default for ModelMappings {
    fn default() -> Self {
        Self {
            fast: default_fast_model(),
            standard: default_standard_model(),
            deep: default_deep_model(),
        }
    }
}

/// Routes tasks to appropriate models.
pub struct ModelRouter {
    /// Available models by tier
    models: Vec<ModelInfo>,
    /// Complexity classifier
    classifier: ComplexityClassifier,
    /// Model mappings from config
    mappings: ModelMappings,
}

impl ModelRouter {
    /// Create a new model router with default models.
    pub fn new() -> Self {
        Self::with_mappings(ModelMappings::default())
    }

    /// Create with custom model mappings.
    pub fn with_mappings(mappings: ModelMappings) -> Self {
        let models = vec![
            ModelInfo::fast(&mappings.fast),
            ModelInfo::standard(&mappings.standard),
            ModelInfo::deep(&mappings.deep),
        ];

        Self {
            models,
            classifier: ComplexityClassifier::new(),
            mappings,
        }
    }

    /// Select the optimal model for a task.
    pub fn select_model(&self, task: &AgentTask, agent_tier: ModelTier) -> &ModelInfo {
        let complexity = self.classifier.classify(&task.question);
        let target_tier = self.determine_tier(agent_tier, complexity);

        debug!(
            "Task complexity: {:?}, Agent tier: {:?}, Target tier: {:?}",
            complexity, agent_tier, target_tier
        );

        self.get_model_for_tier(target_tier)
    }

    /// Select model by name.
    pub fn get_model(&self, name: &str) -> Option<&ModelInfo> {
        self.models.iter().find(|m| m.name == name)
    }

    /// Get model for a specific tier.
    pub fn get_model_for_tier(&self, tier: ModelTier) -> &ModelInfo {
        self.models
            .iter()
            .find(|m| m.tier == tier)
            .unwrap_or_else(|| self.models.first().expect("No models configured"))
    }

    /// Determine the target tier based on agent preference and task complexity.
    fn determine_tier(&self, agent_tier: ModelTier, complexity: Complexity) -> ModelTier {
        match (agent_tier, complexity) {
            // Fast agents always use fast model
            (ModelTier::Fast, _) => ModelTier::Fast,

            // Standard agents: downgrade simple tasks, upgrade complex
            (ModelTier::Standard, Complexity::Simple) => ModelTier::Fast,
            (ModelTier::Standard, Complexity::Standard) => ModelTier::Standard,
            (ModelTier::Standard, Complexity::Complex) => ModelTier::Standard,
            (ModelTier::Standard, Complexity::VeryComplex) => ModelTier::Deep,

            // Deep agents: downgrade simple, keep complex
            (ModelTier::Deep, Complexity::Simple) => ModelTier::Standard,
            (ModelTier::Deep, Complexity::Standard) => ModelTier::Standard,
            (ModelTier::Deep, Complexity::Complex) => ModelTier::Deep,
            (ModelTier::Deep, Complexity::VeryComplex) => ModelTier::Deep,
        }
    }

    /// Get the model name for a tier.
    pub fn model_name_for_tier(&self, tier: ModelTier) -> &str {
        match tier {
            ModelTier::Fast => &self.mappings.fast,
            ModelTier::Standard => &self.mappings.standard,
            ModelTier::Deep => &self.mappings.deep,
        }
    }

    /// Classify task complexity.
    pub fn classify_complexity(&self, question: &str) -> Complexity {
        self.classifier.classify(question)
    }
}

impl Default for ModelRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_selection_fast_agent() {
        let router = ModelRouter::new();
        let task = AgentTask::new("what is my IP?");
        let model = router.select_model(&task, ModelTier::Fast);
        assert_eq!(model.tier, ModelTier::Fast);
    }

    #[test]
    fn test_model_selection_deep_agent_simple_task() {
        let router = ModelRouter::new();
        let task = AgentTask::new("what is my IP?");
        let model = router.select_model(&task, ModelTier::Deep);
        // Deep agent on simple task should use standard model
        assert_eq!(model.tier, ModelTier::Standard);
    }

    #[test]
    fn test_model_selection_standard_agent_complex_task() {
        let router = ModelRouter::new();
        let task = AgentTask::new("why is my system slow and how do I optimize it?");
        let model = router.select_model(&task, ModelTier::Standard);
        // Standard agent on very complex task should use deep model
        assert_eq!(model.tier, ModelTier::Deep);
    }
}
