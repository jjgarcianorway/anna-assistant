//! Learning candidates storage and management.

use super::observation::TicketObservation;
use super::utils::candidates_path;
use crate::canonical_intents::CanonicalIntent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Learning candidates store
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LearningCandidates {
    /// Observations grouped by intent
    pub by_intent: HashMap<String, Vec<TicketObservation>>,
    /// Minimum observations to create recipe
    pub min_observations: usize,
    /// Minimum success rate to create recipe
    pub min_success_rate: f32,
}

impl LearningCandidates {
    pub fn new() -> Self {
        Self {
            by_intent: HashMap::new(),
            min_observations: 2,
            min_success_rate: 0.8,
        }
    }

    /// Load from disk
    pub fn load() -> Self {
        let path = candidates_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::new()
        }
    }

    /// Save to disk
    pub fn save(&self) -> Result<(), String> {
        let path = candidates_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }

    /// Record a ticket observation
    pub fn record(&mut self, observation: TicketObservation) {
        let key = format!("{:?}", observation.intent);
        let observations = self.by_intent.entry(key).or_default();

        // Keep last 20 observations per intent
        if observations.len() >= 20 {
            observations.remove(0);
        }
        observations.push(observation);
    }

    /// Check if ready to learn a recipe for intent
    pub fn ready_to_learn(&self, intent: CanonicalIntent) -> bool {
        let key = format!("{:?}", intent);
        if let Some(observations) = self.by_intent.get(&key) {
            if observations.len() < self.min_observations {
                return false;
            }

            let successful = observations.iter().filter(|o| o.successful).count();
            let rate = successful as f32 / observations.len() as f32;
            rate >= self.min_success_rate
        } else {
            false
        }
    }

    /// Get observations for intent
    pub fn get_observations(&self, intent: CanonicalIntent) -> Vec<&TicketObservation> {
        let key = format!("{:?}", intent);
        self.by_intent
            .get(&key)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }
}
