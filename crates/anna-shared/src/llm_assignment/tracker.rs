//! LLM Assignment Tracker
//!
//! Tracks which LLM model each specialist uses.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{LlmAssignment, ModelTier};

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
