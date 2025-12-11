//! Recipe Fast Path (v0.0.416).
//!
//! Executes learned recipes before falling back to specialists.
//!
//! Flow:
//! 1. Translator classifies query → intent + domain
//! 2. Fast path checks for matching recipe
//! 3. If recipe exists: run probes, execute recipe, return answer
//! 4. If no recipe or recipe fails: fall back to specialist
//!
//! This allows Anna to answer common queries without LLM calls.

use crate::canonical_intents::{translator_to_canonical, CanonicalIntent};
use crate::knowledge_engine::{KnowledgeContext, KnowledgeEngine, KnowledgeKind, KnowledgeRequest};
use crate::learned_recipes::{execute_recipe, RecipeContext, RecipeResult, RecipeStore};
use crate::recipe_learner::RecipeLearner;
use crate::translator_contract::TranslatorOutput;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Fast path result
#[derive(Debug, Clone)]
pub enum FastPathResult {
    /// Recipe answered the query
    Answered {
        summary: String,
        details: Vec<String>,
        evidence: Vec<String>,
        confidence: f32,
        recipe_id: String,
        knowledge_enrichment: Option<String>,
    },
    /// No recipe available, fall back to specialist
    NoRecipe {
        intent: CanonicalIntent,
        reason: String,
    },
    /// Recipe failed, fall back to specialist
    RecipeFailed {
        recipe_id: String,
        reason: String,
    },
}

/// Fast path executor
pub struct FastPathExecutor {
    learner: RecipeLearner,
    knowledge: KnowledgeEngine,
}

impl FastPathExecutor {
    pub fn new() -> Self {
        Self {
            learner: RecipeLearner::new(),
            knowledge: KnowledgeEngine::new(),
        }
    }

    /// Try to answer using a recipe
    pub fn try_recipe(
        &mut self,
        translator_output: &TranslatorOutput,
        probe_outputs: &HashMap<String, String>,
    ) -> FastPathResult {
        // Map to canonical intent
        let intent = translator_to_canonical(
            &translator_output.intent.to_string(),
            &translator_output.domain.to_string(),
            &translator_output.needs_probes,
        );

        // Check if intent is recipe-eligible
        if !intent.is_recipe_eligible() {
            return FastPathResult::NoRecipe {
                intent,
                reason: "Intent not eligible for recipe".to_string(),
            };
        }

        // Look for matching recipe
        let recipe = match self.learner.get_recipe(intent) {
            Some(r) => r.clone(),
            None => {
                return FastPathResult::NoRecipe {
                    intent,
                    reason: "No learned recipe for this intent".to_string(),
                };
            }
        };

        // Check required probes are available
        for probe in &recipe.required_probes {
            if !probe_outputs.contains_key(probe) {
                return FastPathResult::NoRecipe {
                    intent,
                    reason: format!("Missing required probe: {}", probe),
                };
            }
        }

        // Create execution context
        let mut ctx = RecipeContext::with_probes(probe_outputs.clone());

        // Execute recipe
        let result = execute_recipe(&recipe, &mut ctx);

        match result {
            RecipeResult::Success { answer, confidence } => {
                // Record success
                self.learner.record_recipe_result(&recipe.id, true, confidence);

                // Optionally fetch knowledge for enrichment
                let knowledge_enrichment = self.fetch_knowledge_enrichment(&recipe, &intent);

                FastPathResult::Answered {
                    summary: answer.summary,
                    details: answer.details,
                    evidence: answer.evidence,
                    confidence,
                    recipe_id: recipe.id,
                    knowledge_enrichment,
                }
            }
            RecipeResult::Partial { answer, confidence, missing } => {
                // Partial success - still use it but note the limitations
                self.learner.record_recipe_result(&recipe.id, true, confidence * 0.8);

                FastPathResult::Answered {
                    summary: answer.summary,
                    details: answer.details,
                    evidence: answer.evidence,
                    confidence: confidence * 0.8,
                    recipe_id: recipe.id,
                    knowledge_enrichment: Some(format!("(partial: missing {})", missing.join(", "))),
                }
            }
            RecipeResult::Failed { reason } => {
                // Record failure
                self.learner.record_recipe_result(&recipe.id, false, 0.0);

                FastPathResult::RecipeFailed {
                    recipe_id: recipe.id,
                    reason,
                }
            }
        }
    }

    /// Fetch knowledge enrichment for the recipe
    fn fetch_knowledge_enrichment(
        &self,
        recipe: &crate::learned_recipes::LearnedRecipe,
        intent: &CanonicalIntent,
    ) -> Option<String> {
        if recipe.knowledge_topics.is_empty() {
            return None;
        }

        let request = KnowledgeRequest {
            topic: recipe.knowledge_topics.first()?.clone(),
            context: KnowledgeContext {
                intent: format!("{:?}", intent),
                domain: recipe.domain.clone(),
                commands: vec![],
            },
            sources: vec![KnowledgeKind::ManPage, KnowledgeKind::CliHelp],
            limit: 1,
        };

        let response = self.knowledge.query(&request);
        response.hits.first().map(|h| {
            format!("(ref: {})", h.title)
        })
    }

    /// Get learner reference for recording observations
    pub fn learner_mut(&mut self) -> &mut RecipeLearner {
        &mut self.learner
    }

    /// Get recipe store summary
    pub fn store_summary(&self) -> RecipeStoreSummary {
        let store = self.learner.store();
        let summary = store.stats_summary();

        RecipeStoreSummary {
            total_recipes: summary.total,
            active_recipes: summary.active,
            deprecated_recipes: summary.deprecated,
            total_uses: summary.total_uses,
            success_rate: summary.success_rate,
            covered_intents: self.covered_intents(),
        }
    }

    /// Get list of intents covered by recipes
    fn covered_intents(&self) -> Vec<String> {
        self.learner.store()
            .active_recipes()
            .iter()
            .map(|r| format!("{:?}", r.intent))
            .collect()
    }
}

impl Default for FastPathExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary stats for recipe store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeStoreSummary {
    pub total_recipes: usize,
    pub active_recipes: usize,
    pub deprecated_recipes: usize,
    pub total_uses: u32,
    pub success_rate: f32,
    pub covered_intents: Vec<String>,
}

/// Check if a query should try the fast path
pub fn should_try_fast_path(translator_output: &TranslatorOutput) -> bool {
    // Don't use fast path for explanation queries
    if translator_output.intent == crate::translator_contract::TranslatorIntent::Explain {
        return false;
    }

    // Don't use fast path if clarification is needed
    if translator_output.needs_clarification {
        return false;
    }

    // Don't use fast path for low confidence classification
    if translator_output.confidence < 0.6 {
        return false;
    }

    true
}

/// Create ticket response from fast path result
pub fn fast_path_to_response(
    result: &FastPathResult,
    ticket_id: &str,
) -> Option<FastPathResponse> {
    match result {
        FastPathResult::Answered {
            summary,
            details,
            evidence,
            confidence,
            recipe_id,
            knowledge_enrichment,
        } => {
            Some(FastPathResponse {
                ticket_id: ticket_id.to_string(),
                summary: summary.clone(),
                details: details.clone(),
                evidence: evidence.clone(),
                confidence: *confidence,
                source: ResponseSource::Recipe {
                    recipe_id: recipe_id.clone(),
                },
                enrichment: knowledge_enrichment.clone(),
            })
        }
        _ => None,
    }
}

/// Response from fast path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastPathResponse {
    pub ticket_id: String,
    pub summary: String,
    pub details: Vec<String>,
    pub evidence: Vec<String>,
    pub confidence: f32,
    pub source: ResponseSource,
    pub enrichment: Option<String>,
}

/// Source of the response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseSource {
    /// Answered by learned recipe
    Recipe { recipe_id: String },
    /// Answered by specialist LLM
    Specialist { specialist: String },
    /// Answered by built-in logic
    BuiltIn,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translator_contract::{TranslatorDomain, TranslatorIntent, Priority};

    #[test]
    fn test_should_try_fast_path() {
        let output = TranslatorOutput {
            intent: TranslatorIntent::QueryMetric,
            domain: TranslatorDomain::Storage,
            needs_probes: vec!["disk_usage".to_string()],
            follow_up_questions: vec![],
            needs_clarification: false,
            priority: Priority::Normal,
            confidence: 0.9,
        };

        assert!(should_try_fast_path(&output));
    }

    #[test]
    fn test_should_not_fast_path_explain() {
        let output = TranslatorOutput {
            intent: TranslatorIntent::Explain,
            domain: TranslatorDomain::System,
            needs_probes: vec![],
            follow_up_questions: vec![],
            needs_clarification: false,
            priority: Priority::Normal,
            confidence: 0.9,
        };

        assert!(!should_try_fast_path(&output));
    }

    #[test]
    fn test_should_not_fast_path_low_confidence() {
        let output = TranslatorOutput {
            intent: TranslatorIntent::QueryMetric,
            domain: TranslatorDomain::Storage,
            needs_probes: vec!["disk_usage".to_string()],
            follow_up_questions: vec![],
            needs_clarification: false,
            priority: Priority::Normal,
            confidence: 0.4,
        };

        assert!(!should_try_fast_path(&output));
    }

    #[test]
    fn test_fast_path_no_recipe() {
        let mut executor = FastPathExecutor::new();
        let output = TranslatorOutput {
            intent: TranslatorIntent::QueryMetric,
            domain: TranslatorDomain::Storage,
            needs_probes: vec!["disk_usage".to_string()],
            follow_up_questions: vec![],
            needs_clarification: false,
            priority: Priority::Normal,
            confidence: 0.9,
        };

        let probes = HashMap::new();
        let result = executor.try_recipe(&output, &probes);

        // Should return NoRecipe since we haven't learned any yet
        assert!(matches!(result, FastPathResult::NoRecipe { .. }));
    }
}
