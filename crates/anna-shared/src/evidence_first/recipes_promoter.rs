//! Recipe promotion manager (v0.0.435).

use super::citations::CitationStore;
use super::recipes_candidate::RecipeCandidate;
use super::recipes_types::RecipeTemplate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Manages recipe candidates and promotion.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecipePromoter {
    /// Candidates awaiting promotion.
    candidates: HashMap<String, RecipeCandidate>,
    /// Promoted recipes.
    promoted: HashMap<String, RecipeTemplate>,
}

impl RecipePromoter {
    /// Create a new promoter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a candidate recipe.
    pub fn add_candidate(&mut self, template: RecipeTemplate) {
        if !self.candidates.contains_key(&template.id) && !self.promoted.contains_key(&template.id)
        {
            self.candidates
                .insert(template.id.clone(), RecipeCandidate::new(template));
        }
    }

    /// Record execution result.
    pub fn record_execution(
        &mut self,
        recipe_id: &str,
        ticket_id: &str,
        success: bool,
        citations: Option<&CitationStore>,
        failure_reason: Option<&str>,
    ) {
        if let Some(candidate) = self.candidates.get_mut(recipe_id) {
            if success {
                if let Some(cites) = citations {
                    candidate.record_success(ticket_id, cites);
                }
            } else if let Some(reason) = failure_reason {
                candidate.record_failure(ticket_id, reason);
            }

            // Check for promotion
            if candidate.ready_for_promotion() {
                self.promote(recipe_id);
            }
        }
    }

    /// Promote a candidate to full recipe.
    fn promote(&mut self, recipe_id: &str) {
        if let Some(candidate) = self.candidates.remove(recipe_id) {
            self.promoted
                .insert(recipe_id.to_string(), candidate.template);
        }
    }

    /// Get a promoted recipe.
    pub fn get_promoted(&self, recipe_id: &str) -> Option<&RecipeTemplate> {
        self.promoted.get(recipe_id)
    }

    /// Get a candidate.
    pub fn get_candidate(&self, recipe_id: &str) -> Option<&RecipeCandidate> {
        self.candidates.get(recipe_id)
    }

    /// Find matching recipes by tags.
    pub fn find_by_tags(&self, tags: &[&str]) -> Vec<&RecipeTemplate> {
        let mut results: Vec<&RecipeTemplate> = self
            .promoted
            .values()
            .filter(|r| tags.iter().any(|t| r.tags.contains(&t.to_string())))
            .collect();

        // Also include candidates with high success rates
        for candidate in self.candidates.values() {
            if candidate.success_rate() > 0.8
                && tags
                    .iter()
                    .any(|t| candidate.template.tags.contains(&t.to_string()))
            {
                results.push(&candidate.template);
            }
        }

        results
    }

    /// List all promoted recipes.
    pub fn list_promoted(&self) -> Vec<&RecipeTemplate> {
        self.promoted.values().collect()
    }

    /// List all candidates.
    pub fn list_candidates(&self) -> Vec<&RecipeCandidate> {
        self.candidates.values().collect()
    }

    /// Get promotion status.
    pub fn status(&self) -> PromoterStatus {
        PromoterStatus {
            promoted_count: self.promoted.len(),
            candidate_count: self.candidates.len(),
            pending_confirmations: self
                .candidates
                .values()
                .map(|c| super::MIN_CONFIRMATIONS_FOR_RECIPE - c.confirmation_count())
                .sum(),
        }
    }
}

/// Status of the recipe promoter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromoterStatus {
    /// Number of promoted recipes.
    pub promoted_count: usize,
    /// Number of candidates.
    pub candidate_count: usize,
    /// Total confirmations needed across all candidates.
    pub pending_confirmations: usize,
}
