//! Knowledge Learning Store
//!
//! Storage and persistence for learning data.

use super::types::{LearningStats, ProbeStats, ProposedRecipe, RecipeStatus, SolvedTicketRecord};
use crate::intent_policy::IntentCategory;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Learning store file path
const LEARNING_STORE_PATH: &str = "/var/lib/anna/knowledge_learning.json";

/// User learning store path
const USER_LEARNING_PATH: &str = "~/.anna/learning.json";

/// The knowledge learning store
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeLearningStore {
    /// Solved ticket records
    pub tickets: Vec<SolvedTicketRecord>,
    /// Proposed recipes from learning
    pub proposed_recipes: Vec<ProposedRecipe>,
    /// Probe effectiveness by intent
    pub probe_effectiveness: HashMap<String, HashMap<String, ProbeStats>>,
    /// Learning statistics
    pub stats: LearningStats,
}

impl KnowledgeLearningStore {
    /// Load from disk
    pub fn load() -> Self {
        // Try system path first, then user path
        let paths = [
            PathBuf::from(LEARNING_STORE_PATH),
            expand_path(USER_LEARNING_PATH),
        ];

        for path in &paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(store) = serde_json::from_str(&content) {
                    return store;
                }
            }
        }

        Self::default()
    }

    /// Save to disk
    pub fn save(&self) -> std::io::Result<()> {
        let path = expand_path(USER_LEARNING_PATH);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)
    }

    /// Record a solved ticket
    pub fn record_ticket(&mut self, record: SolvedTicketRecord) {
        // Update probe effectiveness
        let intent_key = record.intent.to_string();
        let probe_stats = self
            .probe_effectiveness
            .entry(intent_key.clone())
            .or_default();

        for probe in &record.probes_used {
            let stats = probe_stats.entry(probe.clone()).or_default();
            stats.use_count += 1;
            if let Some(&eff) = record.probe_effectiveness.get(probe) {
                if eff > 50 {
                    stats.effective_count += 1;
                }
                stats.avg_relevance = (stats.avg_relevance * (stats.use_count - 1) as f32
                    + eff as f32)
                    / stats.use_count as f32;
            }
        }

        // Update stats
        self.stats.tickets_recorded += 1;
        *self.stats.by_intent.entry(intent_key).or_insert(0) += 1;

        // Update averages
        let total = self.stats.tickets_recorded as f32;
        self.stats.avg_confidence =
            (self.stats.avg_confidence * (total - 1.0) + record.answer_confidence as f32) / total;

        let grounded_count = if record.was_grounded { 1.0 } else { 0.0 };
        self.stats.grounding_rate =
            (self.stats.grounding_rate * (total - 1.0) + grounded_count) / total;

        // Keep last 1000 tickets
        self.tickets.push(record);
        if self.tickets.len() > 1000 {
            self.tickets.remove(0);
        }
    }

    /// Get effective probes for an intent
    pub fn effective_probes_for_intent(&self, intent: IntentCategory) -> Vec<String> {
        let intent_key = intent.to_string();
        self.probe_effectiveness
            .get(&intent_key)
            .map(|stats| {
                let mut probes: Vec<_> = stats
                    .iter()
                    .filter(|(_, s)| {
                        s.use_count >= 3 && s.effective_count as f32 / s.use_count as f32 > 0.6
                    })
                    .map(|(p, s)| (p.clone(), s.avg_relevance))
                    .collect();
                probes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                probes.into_iter().map(|(p, _)| p).collect()
            })
            .unwrap_or_default()
    }

    /// Approve a proposed recipe
    pub fn approve_recipe(&mut self, recipe_id: &str, notes: Option<String>) {
        if let Some(recipe) = self.proposed_recipes.iter_mut().find(|r| r.id == recipe_id) {
            recipe.status = RecipeStatus::Approved;
            recipe.review_notes = notes;
            self.stats.recipes_approved += 1;
        }
    }

    /// Reject a proposed recipe
    pub fn reject_recipe(&mut self, recipe_id: &str, reason: &str) {
        if let Some(recipe) = self.proposed_recipes.iter_mut().find(|r| r.id == recipe_id) {
            recipe.status = RecipeStatus::Rejected;
            recipe.review_notes = Some(reason.to_string());
        }
    }

    /// Get approved recipes
    pub fn approved_recipes(&self) -> Vec<&ProposedRecipe> {
        self.proposed_recipes
            .iter()
            .filter(|r| r.status == RecipeStatus::Approved)
            .collect()
    }
}

/// Expand ~ to home directory
fn expand_path(path: &str) -> PathBuf {
    if path.starts_with("~/") || path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}
