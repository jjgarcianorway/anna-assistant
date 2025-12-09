//! Model registry core (v0.0.201).

use crate::specialists::SpecialistRole;
use crate::teams::Team;
use serde::{Deserialize, Serialize};

use super::types::{recommended_model_for_tier, HardwareTier, ModelState, RoleBinding};

/// Model registry containing all role bindings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelRegistry {
    /// Role-model bindings
    pub bindings: Vec<RoleBinding>,
    /// Detected hardware tier
    pub hardware_tier: Option<HardwareTier>,
    /// Model states from Ollama
    pub states: Vec<(String, ModelState)>,
    /// Last benchmark result (epoch seconds)
    pub last_benchmark_ts: Option<u64>,
}

impl ModelRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Create registry with default bindings for a hardware tier
    pub fn with_defaults(tier: HardwareTier) -> Self {
        let model = recommended_model_for_tier(tier);
        let reason = format!("hardware_tier={}", tier);

        let teams = [
            Team::Desktop,
            Team::Storage,
            Team::Network,
            Team::Performance,
            Team::Services,
            Team::Security,
            Team::Hardware,
            Team::General,
        ];

        let roles = [
            SpecialistRole::Translator,
            SpecialistRole::Junior,
            SpecialistRole::Senior,
        ];

        let mut bindings = Vec::new();
        for team in teams {
            for role in roles {
                bindings.push(RoleBinding::new(team, role, model.clone()).with_reason(&reason));
            }
        }

        Self {
            bindings,
            hardware_tier: Some(tier),
            states: Vec::new(),
            last_benchmark_ts: None,
        }
    }

    /// Get binding for a team and role
    pub fn get_binding(&self, team: Team, role: SpecialistRole) -> Option<&RoleBinding> {
        self.bindings
            .iter()
            .find(|b| b.team == team && b.role == role)
    }

    /// Get model name for a team and role
    pub fn get_model_name(&self, team: Team, role: SpecialistRole) -> Option<&str> {
        self.get_binding(team, role).map(|b| b.model.name.as_str())
    }

    /// Update or add a binding
    pub fn set_binding(&mut self, binding: RoleBinding) {
        if let Some(existing) = self
            .bindings
            .iter_mut()
            .find(|b| b.team == binding.team && b.role == binding.role)
        {
            *existing = binding;
        } else {
            self.bindings.push(binding);
        }
    }

    /// Update model state
    pub fn update_state(&mut self, model_name: &str, state: ModelState) {
        if let Some((_, existing)) = self.states.iter_mut().find(|(n, _)| n == model_name) {
            *existing = state;
        } else {
            self.states.push((model_name.to_string(), state));
        }
    }

    /// Get model state
    pub fn get_state(&self, model_name: &str) -> Option<&ModelState> {
        self.states
            .iter()
            .find(|(n, _)| n == model_name)
            .map(|(_, s)| s)
    }

    /// Check if model is present
    pub fn is_model_present(&self, model_name: &str) -> bool {
        self.get_state(model_name)
            .map(|s| s.present)
            .unwrap_or(false)
    }

    /// Get all unique model names from bindings
    pub fn required_models(&self) -> Vec<&str> {
        let mut models: Vec<&str> = self
            .bindings
            .iter()
            .map(|b| b.model.name.as_str())
            .collect();
        models.sort();
        models.dedup();
        models
    }

    /// Get missing models (required but not present)
    pub fn missing_models(&self) -> Vec<&str> {
        self.required_models()
            .into_iter()
            .filter(|m| !self.is_model_present(m))
            .collect()
    }

    /// Check if all required models are present
    pub fn all_models_present(&self) -> bool {
        self.missing_models().is_empty()
    }

    /// Clear all bindings and states (for reset)
    pub fn clear(&mut self) {
        self.bindings.clear();
        self.states.clear();
        self.hardware_tier = None;
        self.last_benchmark_ts = None;
    }
}
