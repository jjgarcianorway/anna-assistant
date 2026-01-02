//! Knowledge Learning Analysis
//!
//! Pattern extraction and recipe proposal from learned tickets.

use super::store::KnowledgeLearningStore;
use super::types::{ProposedRecipe, RecipeStatus};
use std::collections::HashMap;

impl KnowledgeLearningStore {
    /// Analyze tickets and propose recipes (idle-time learning)
    pub fn analyze_and_propose(&mut self) -> Vec<ProposedRecipe> {
        let mut proposals = Vec::new();

        // Group tickets by intent
        let mut by_intent: HashMap<String, Vec<_>> = HashMap::new();
        for ticket in &self.tickets {
            by_intent
                .entry(ticket.intent.to_string())
                .or_default()
                .push(ticket);
        }

        // For each intent with enough tickets, try to extract a pattern
        for (intent_str, tickets) in &by_intent {
            if tickets.len() < 3 {
                continue; // Need at least 3 examples
            }

            // Find commonly effective probes
            let mut probe_counts: HashMap<String, usize> = HashMap::new();
            for ticket in tickets {
                for probe in &ticket.probes_used {
                    if ticket.probe_effectiveness.get(probe).copied().unwrap_or(0) > 50 {
                        *probe_counts.entry(probe.clone()).or_insert(0) += 1;
                    }
                }
            }

            let common_probes: Vec<String> = probe_counts
                .into_iter()
                .filter(|(_, count)| *count as f32 / tickets.len() as f32 > 0.6)
                .map(|(probe, _)| probe)
                .collect();

            if common_probes.is_empty() {
                continue;
            }

            // Find common knowledge domains
            let mut domain_counts: HashMap<String, usize> = HashMap::new();
            for ticket in tickets {
                domain_counts
                    .entry(ticket.domain.clone())
                    .and_modify(|c| *c += 1)
                    .or_insert(1);
            }

            let common_domains: Vec<String> = domain_counts
                .into_iter()
                .filter(|(_, count)| *count as f32 / tickets.len() as f32 > 0.5)
                .map(|(domain, _)| domain)
                .collect();

            // Calculate average confidence
            let avg_conf = tickets
                .iter()
                .map(|t| t.answer_confidence as f32)
                .sum::<f32>()
                / tickets.len() as f32;

            // Create proposal
            let proposal = ProposedRecipe {
                id: format!("learned_{}", intent_str),
                intent: tickets[0].intent,
                pattern: format!(
                    "Pattern for {} (from {} examples)",
                    intent_str,
                    tickets.len()
                ),
                probes: common_probes,
                knowledge_domains: common_domains,
                answer_template: "Answer based on evidence from probes and docs.".to_string(),
                confidence: avg_conf as u8,
                evidence_count: tickets.len(),
                status: RecipeStatus::PendingReview,
                review_notes: None,
            };

            proposals.push(proposal);
        }

        // Store proposals
        for proposal in &proposals {
            if !self.proposed_recipes.iter().any(|p| p.id == proposal.id) {
                self.proposed_recipes.push(proposal.clone());
                self.stats.recipes_proposed += 1;
            }
        }

        proposals
    }
}
