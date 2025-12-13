//! LLM Assignment Tracker - Phase 88
//!
//! Tracks which LLM model each specialist uses.
//! VISION.md: "Which specialist uses which LLM"
//! "Models adjusted by Anna based on hardware available"

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Model tier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ModelTier {
    #[default]
    Light,
    Standard,
    Heavy,
    DeepThinking,
}

impl ModelTier {
    pub fn name(&self) -> &'static str {
        match self {
            ModelTier::Light => "Light",
            ModelTier::Standard => "Standard",
            ModelTier::Heavy => "Heavy",
            ModelTier::DeepThinking => "Deep Thinking",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            ModelTier::Light => "L",
            ModelTier::Standard => "S",
            ModelTier::Heavy => "H",
            ModelTier::DeepThinking => "D",
        }
    }
}

/// Model assignment reason
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AssignmentReason {
    #[default]
    Default,
    HardwareLimit,
    UserPreference,
    TaskComplexity,
    PerformanceOptimization,
    Fallback,
}

impl AssignmentReason {
    pub fn name(&self) -> &'static str {
        match self {
            AssignmentReason::Default => "Default",
            AssignmentReason::HardwareLimit => "Hardware Limit",
            AssignmentReason::UserPreference => "User Preference",
            AssignmentReason::TaskComplexity => "Task Complexity",
            AssignmentReason::PerformanceOptimization => "Performance",
            AssignmentReason::Fallback => "Fallback",
        }
    }
}

/// An LLM assignment record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmAssignment {
    /// Specialist ID or role
    pub specialist_id: String,
    /// Model name
    pub model: String,
    /// Model tier
    pub tier: ModelTier,
    /// Why this model was assigned
    pub reason: AssignmentReason,
    /// When assigned
    pub assigned_at: u64,
    /// Is currently active
    pub active: bool,
    /// Parameters used (e.g., temperature)
    pub parameters: HashMap<String, String>,
}

/// LLM assignment tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LlmAssignmentTracker {
    /// All assignments
    pub assignments: Vec<LlmAssignment>,
    /// Count by model
    pub by_model: HashMap<String, u64>,
    /// Count by tier
    pub by_tier: HashMap<String, u64>,
    /// Available models on system
    pub available_models: Vec<String>,
    /// Hardware-detected recommended tier
    pub recommended_tier: Option<ModelTier>,
}

impl LlmAssignmentTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add available model
    pub fn add_available_model(&mut self, model: String) {
        if !self.available_models.contains(&model) {
            self.available_models.push(model);
        }
    }

    /// Set recommended tier based on hardware
    pub fn set_recommended_tier(&mut self, tier: ModelTier) {
        self.recommended_tier = Some(tier);
    }

    /// Assign model to specialist
    pub fn assign(&mut self, assignment: LlmAssignment) {
        // Deactivate previous assignment for same specialist
        for a in &mut self.assignments {
            if a.specialist_id == assignment.specialist_id && a.active {
                a.active = false;
            }
        }

        *self.by_model.entry(assignment.model.clone()).or_insert(0) += 1;
        *self.by_tier.entry(assignment.tier.name().to_string()).or_insert(0) += 1;
        self.assignments.push(assignment);
    }

    /// Get current assignment for specialist
    pub fn get_assignment(&self, specialist_id: &str) -> Option<&LlmAssignment> {
        self.assignments
            .iter()
            .find(|a| a.specialist_id == specialist_id && a.active)
    }

    /// Get all active assignments
    pub fn active_assignments(&self) -> Vec<&LlmAssignment> {
        self.assignments.iter().filter(|a| a.active).collect()
    }

    /// Get assignments by model
    pub fn by_llm_model(&self, model: &str) -> Vec<&LlmAssignment> {
        self.assignments.iter().filter(|a| a.model == model).collect()
    }

    /// Get assignments by tier
    pub fn by_model_tier(&self, tier: ModelTier) -> Vec<&LlmAssignment> {
        self.assignments.iter().filter(|a| a.tier == tier).collect()
    }

    /// Check if model is available
    pub fn is_model_available(&self, model: &str) -> bool {
        self.available_models.iter().any(|m| m == model)
    }

    /// Get model for tier
    pub fn get_model_for_tier(&self, tier: ModelTier) -> Option<&str> {
        // Return first model used for this tier
        self.assignments
            .iter()
            .find(|a| a.tier == tier)
            .map(|a| a.model.as_str())
    }

    /// Total assignment count
    pub fn total_count(&self) -> usize {
        self.assignments.len()
    }

    /// Active assignment count
    pub fn active_count(&self) -> usize {
        self.assignments.iter().filter(|a| a.active).count()
    }

    /// Unique models in use
    pub fn models_in_use(&self) -> Vec<&str> {
        let mut models: Vec<&str> = self.active_assignments()
            .iter()
            .map(|a| a.model.as_str())
            .collect();
        models.sort();
        models.dedup();
        models
    }

    /// Most used model
    pub fn most_used_model(&self) -> Option<(&str, u64)> {
        self.by_model
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, v)| (k.as_str(), *v))
    }
}

/// Common model names
pub const COMMON_MODELS: &[(&str, ModelTier)] = &[
    ("llama3.2:1b", ModelTier::Light),
    ("llama3.2:3b", ModelTier::Light),
    ("llama3.1:8b", ModelTier::Standard),
    ("llama3.1:70b", ModelTier::Heavy),
    ("qwen2.5:0.5b", ModelTier::Light),
    ("qwen2.5:7b", ModelTier::Standard),
    ("qwen2.5:32b", ModelTier::Heavy),
    ("deepseek-r1:8b", ModelTier::DeepThinking),
    ("deepseek-r1:32b", ModelTier::DeepThinking),
];

/// Get tier for model
pub fn get_model_tier(model: &str) -> ModelTier {
    for (name, tier) in COMMON_MODELS {
        if model.contains(name) {
            return *tier;
        }
    }
    ModelTier::Standard
}

/// Format LLM tracker for display
pub fn format_llm_tracker(tracker: &LlmAssignmentTracker) -> String {
    let mut lines = vec!["=== LLM Assignments ===".to_string()];
    lines.push(String::new());

    // Available models
    if !tracker.available_models.is_empty() {
        lines.push(format!("Available models: {}", tracker.available_models.len()));
        for model in &tracker.available_models {
            lines.push(format!("  - {}", model));
        }
    }

    // Recommended tier
    if let Some(tier) = tracker.recommended_tier {
        lines.push(format!("Recommended tier: {}", tier.name()));
    }

    if tracker.assignments.is_empty() {
        lines.push(String::new());
        lines.push("No assignments yet.".to_string());
        return lines.join("\n");
    }

    // Active assignments
    let active = tracker.active_assignments();
    if !active.is_empty() {
        lines.push(String::new());
        lines.push("Active assignments:".to_string());
        for a in active {
            lines.push(format!(
                "  {} -> {} [{}]",
                a.specialist_id, a.model, a.tier.name()
            ));
        }
    }

    // Most used
    if let Some((model, count)) = tracker.most_used_model() {
        lines.push(String::new());
        lines.push(format!("Most used: {} ({} times)", model, count));
    }

    lines.join("\n")
}

/// Format LLM tracker compact
pub fn format_llm_tracker_compact(tracker: &LlmAssignmentTracker) -> String {
    let models = tracker.models_in_use();
    format!(
        "LLM: {} active | {} models | tier: {}",
        tracker.active_count(),
        models.len(),
        tracker.recommended_tier.map(|t| t.name()).unwrap_or("unknown")
    )
}

/// Format LLM tracker one-line
pub fn format_llm_tracker_oneline(tracker: &LlmAssignmentTracker) -> String {
    format!(
        "{} LLM assignments ({} active)",
        tracker.total_count(),
        tracker.active_count()
    )
}

/// Check if query is about LLM assignments
pub fn is_llm_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "llm",
        "model assignment",
        "which model",
        "what model",
        "assigned model",
        "ollama model",
        "specialist model",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about LLM assignments
pub fn llm_fun_fact(tracker: &LlmAssignmentTracker) -> String {
    if tracker.assignments.is_empty() {
        return "No LLM assignments yet!".to_string();
    }

    let facts = [
        format!(
            "Anna has {} active LLM assignments.",
            tracker.active_count()
        ),
        format!(
            "{} different models are available.",
            tracker.available_models.len()
        ),
        {
            if let Some((model, count)) = tracker.most_used_model() {
                format!("{} is the most used model ({} assignments).", model, count)
            } else {
                "No model stats yet.".to_string()
            }
        },
        format!(
            "{} unique models currently in use.",
            tracker.models_in_use().len()
        ),
    ];

    facts[tracker.total_count() % facts.len()].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_assignment(specialist: &str, model: &str, tier: ModelTier) -> LlmAssignment {
        LlmAssignment {
            specialist_id: specialist.to_string(),
            model: model.to_string(),
            tier,
            reason: AssignmentReason::Default,
            assigned_at: 1234567890,
            active: true,
            parameters: HashMap::new(),
        }
    }

    #[test]
    fn test_model_tier() {
        assert_eq!(ModelTier::Light.name(), "Light");
        assert_eq!(ModelTier::DeepThinking.symbol(), "D");
    }

    #[test]
    fn test_assignment_reason() {
        assert_eq!(AssignmentReason::HardwareLimit.name(), "Hardware Limit");
        assert_eq!(AssignmentReason::Default.name(), "Default");
    }

    #[test]
    fn test_assign() {
        let mut tracker = LlmAssignmentTracker::new();
        tracker.assign(make_assignment("junior-desktop", "llama3.1:8b", ModelTier::Standard));

        assert_eq!(tracker.total_count(), 1);
        assert!(tracker.get_assignment("junior-desktop").is_some());
    }

    #[test]
    fn test_reassign_deactivates_old() {
        let mut tracker = LlmAssignmentTracker::new();
        tracker.assign(make_assignment("junior-desktop", "llama3.1:8b", ModelTier::Standard));
        tracker.assign(make_assignment("junior-desktop", "llama3.2:3b", ModelTier::Light));

        assert_eq!(tracker.total_count(), 2);
        assert_eq!(tracker.active_count(), 1);
        let current = tracker.get_assignment("junior-desktop").unwrap();
        assert_eq!(current.model, "llama3.2:3b");
    }

    #[test]
    fn test_add_available_model() {
        let mut tracker = LlmAssignmentTracker::new();
        tracker.add_available_model("llama3.1:8b".to_string());
        tracker.add_available_model("llama3.1:8b".to_string()); // duplicate

        assert_eq!(tracker.available_models.len(), 1);
        assert!(tracker.is_model_available("llama3.1:8b"));
    }

    #[test]
    fn test_set_recommended_tier() {
        let mut tracker = LlmAssignmentTracker::new();
        tracker.set_recommended_tier(ModelTier::Heavy);

        assert_eq!(tracker.recommended_tier, Some(ModelTier::Heavy));
    }

    #[test]
    fn test_by_tier() {
        let mut tracker = LlmAssignmentTracker::new();
        tracker.assign(make_assignment("junior", "llama3.2:3b", ModelTier::Light));
        tracker.assign(make_assignment("senior", "llama3.1:8b", ModelTier::Standard));

        assert_eq!(tracker.by_model_tier(ModelTier::Light).len(), 1);
        assert_eq!(tracker.by_model_tier(ModelTier::Standard).len(), 1);
    }

    #[test]
    fn test_models_in_use() {
        let mut tracker = LlmAssignmentTracker::new();
        tracker.assign(make_assignment("junior", "llama3.2:3b", ModelTier::Light));
        tracker.assign(make_assignment("senior", "llama3.1:8b", ModelTier::Standard));

        let models = tracker.models_in_use();
        assert_eq!(models.len(), 2);
    }

    #[test]
    fn test_get_model_tier() {
        assert_eq!(get_model_tier("llama3.2:1b"), ModelTier::Light);
        assert_eq!(get_model_tier("llama3.1:70b"), ModelTier::Heavy);
        assert_eq!(get_model_tier("deepseek-r1:8b"), ModelTier::DeepThinking);
    }

    #[test]
    fn test_format_llm_tracker() {
        let mut tracker = LlmAssignmentTracker::new();
        tracker.assign(make_assignment("junior", "llama3.2:3b", ModelTier::Light));

        let output = format_llm_tracker(&tracker);
        assert!(output.contains("LLM Assignments"));
        assert!(output.contains("llama3.2:3b"));
    }

    #[test]
    fn test_is_llm_query() {
        assert!(is_llm_query("which model is used?"));
        assert!(is_llm_query("show llm assignments"));
        assert!(is_llm_query("what model does the specialist use?"));
        assert!(!is_llm_query("what is the weather?"));
    }

    #[test]
    fn test_llm_fun_fact() {
        let mut tracker = LlmAssignmentTracker::new();
        tracker.assign(make_assignment("junior", "llama3.2:3b", ModelTier::Light));

        let fact = llm_fun_fact(&tracker);
        assert!(!fact.is_empty());
    }
}
