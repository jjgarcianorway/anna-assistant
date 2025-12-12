//! Knowledge Learning System (v0.0.414).
//!
//! Self-learning from docs and successful tickets.
//!
//! Learning workflow:
//! 1. On every high-confidence solved ticket, record:
//!    - Intent classification
//!    - Probes used and their effectiveness
//!    - Knowledge docs consulted
//!    - Final answer structure (citations, grounding)
//!
//! 2. Periodically (idle-time learning job):
//!    - Cluster tickets by intent
//!    - Extract patterns (probes that work, docs that help)
//!    - Propose new recipes from patterns
//!    - Senior LLM review for safety
//!
//! No hardcoded natural language - all learning is from evidence.

use crate::doc_first_workflow::{CitedAnswer, SpecialistEvidence};
use crate::intent_policy::IntentCategory;
use crate::knowledge_query::KnowledgeSourceKind;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Learning store file path
const LEARNING_STORE_PATH: &str = "/var/lib/anna/knowledge_learning.json";

/// User learning store path
const USER_LEARNING_PATH: &str = "~/.anna/learning.json";

/// A solved ticket record for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolvedTicketRecord {
    /// Ticket ID
    pub ticket_id: String,
    /// Intent classification
    pub intent: IntentCategory,
    /// Domain classification
    pub domain: String,
    /// Normalized query pattern (intent + key tags)
    pub query_pattern: String,
    /// Probes that were executed
    pub probes_used: Vec<String>,
    /// Probe effectiveness scores (0-100)
    pub probe_effectiveness: HashMap<String, u8>,
    /// Knowledge sources consulted
    pub docs_consulted: Vec<DocReference>,
    /// Final answer confidence (0-100)
    pub answer_confidence: u8,
    /// Whether answer was grounded in evidence
    pub was_grounded: bool,
    /// Citation IDs used
    pub citations_used: Vec<String>,
    /// Timestamp (Unix secs)
    pub timestamp: u64,
    /// User feedback (if any)
    pub feedback: Option<UserFeedback>,
}

/// Reference to a consulted document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocReference {
    /// Document ID
    pub doc_id: String,
    /// Source kind
    pub kind: KnowledgeSourceKind,
    /// How relevant it was (0-100)
    pub relevance: u8,
    /// Whether it was actually cited in the answer
    pub was_cited: bool,
}

/// User feedback on a solved ticket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFeedback {
    /// Helpful rating (true/false)
    pub helpful: bool,
    /// Optional comment
    pub comment: Option<String>,
    /// Timestamp
    pub timestamp: u64,
}

/// A proposed recipe from learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedRecipe {
    /// Unique ID
    pub id: String,
    /// Intent category this recipe handles
    pub intent: IntentCategory,
    /// Pattern description (not natural language trigger!)
    pub pattern: String,
    /// Recommended probes
    pub probes: Vec<String>,
    /// Knowledge domains to search
    pub knowledge_domains: Vec<String>,
    /// Answer template with placeholders
    pub answer_template: String,
    /// Confidence in this recipe (0-100)
    pub confidence: u8,
    /// Number of tickets this was learned from
    pub evidence_count: usize,
    /// Status (pending_review, approved, rejected)
    pub status: RecipeStatus,
    /// Review notes
    pub review_notes: Option<String>,
}

/// Recipe approval status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipeStatus {
    /// Pending Senior LLM review
    PendingReview,
    /// Approved for use
    Approved,
    /// Rejected (with reason)
    Rejected,
}

/// Learning statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LearningStats {
    /// Total tickets recorded
    pub tickets_recorded: usize,
    /// Tickets by intent
    pub by_intent: HashMap<String, usize>,
    /// Average confidence
    pub avg_confidence: f32,
    /// Grounding rate (% of grounded answers)
    pub grounding_rate: f32,
    /// Recipes proposed
    pub recipes_proposed: usize,
    /// Recipes approved
    pub recipes_approved: usize,
}

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

/// Probe effectiveness statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProbeStats {
    /// Number of times used
    pub use_count: usize,
    /// Number of times effective (contributed to answer)
    pub effective_count: usize,
    /// Average relevance when used
    pub avg_relevance: f32,
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

    /// Analyze tickets and propose recipes (idle-time learning)
    pub fn analyze_and_propose(&mut self) -> Vec<ProposedRecipe> {
        let mut proposals = Vec::new();

        // Group tickets by intent
        let mut by_intent: HashMap<String, Vec<&SolvedTicketRecord>> = HashMap::new();
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

/// Create a solved ticket record from specialist evidence and answer
pub fn create_ticket_record(
    ticket_id: &str,
    intent: IntentCategory,
    domain: &str,
    query_pattern: &str,
    evidence: &SpecialistEvidence,
    answer: &CitedAnswer,
) -> SolvedTicketRecord {
    let probes_used: Vec<String> = evidence
        .probe_evidence
        .iter()
        .map(|p| p.id.clone())
        .collect();

    let probe_effectiveness: HashMap<String, u8> = evidence
        .probe_evidence
        .iter()
        .map(|p| {
            // Effectiveness based on whether probe was cited
            let eff = if answer.citations.iter().any(|c| c.contains(&p.id)) {
                90
            } else {
                50
            };
            (p.id.clone(), eff)
        })
        .collect();

    let docs_consulted: Vec<DocReference> = evidence
        .doc_evidence
        .iter()
        .map(|d| DocReference {
            doc_id: d.doc_id.clone(),
            kind: d.kind,
            relevance: d.relevance,
            was_cited: answer.citations.contains(&d.origin),
        })
        .collect();

    SolvedTicketRecord {
        ticket_id: ticket_id.to_string(),
        intent,
        domain: domain.to_string(),
        query_pattern: query_pattern.to_string(),
        probes_used,
        probe_effectiveness,
        docs_consulted,
        answer_confidence: answer.confidence,
        was_grounded: answer.grounded,
        citations_used: answer.citations.clone(),
        timestamp: current_secs(),
        feedback: None,
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

fn current_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_ticket() {
        let mut store = KnowledgeLearningStore::default();

        let record = SolvedTicketRecord {
            ticket_id: "TEST-001".to_string(),
            intent: IntentCategory::DiagnoseServiceFailure,
            domain: "services".to_string(),
            query_pattern: "diagnose_service_failure".to_string(),
            probes_used: vec!["systemctl_failed".to_string()],
            probe_effectiveness: [("systemctl_failed".to_string(), 85)].into_iter().collect(),
            docs_consulted: vec![],
            answer_confidence: 90,
            was_grounded: true,
            citations_used: vec!["man systemctl".to_string()],
            timestamp: current_secs(),
            feedback: None,
        };

        store.record_ticket(record);
        assert_eq!(store.stats.tickets_recorded, 1);
        assert!(store.stats.grounding_rate > 0.9);
    }

    #[test]
    fn test_effective_probes() {
        let mut store = KnowledgeLearningStore::default();

        // Add multiple tickets with same probe being effective
        for i in 0..5 {
            let record = SolvedTicketRecord {
                ticket_id: format!("TEST-{:03}", i),
                intent: IntentCategory::DiagnoseServiceFailure,
                domain: "services".to_string(),
                query_pattern: "diagnose_service_failure".to_string(),
                probes_used: vec!["systemctl_failed".to_string()],
                probe_effectiveness: [("systemctl_failed".to_string(), 85)].into_iter().collect(),
                docs_consulted: vec![],
                answer_confidence: 90,
                was_grounded: true,
                citations_used: vec![],
                timestamp: current_secs(),
                feedback: None,
            };
            store.record_ticket(record);
        }

        let effective = store.effective_probes_for_intent(IntentCategory::DiagnoseServiceFailure);
        assert!(effective.contains(&"systemctl_failed".to_string()));
    }

    #[test]
    fn test_analyze_and_propose() {
        let mut store = KnowledgeLearningStore::default();

        // Add enough tickets to trigger proposal
        for i in 0..5 {
            let record = SolvedTicketRecord {
                ticket_id: format!("TEST-{:03}", i),
                intent: IntentCategory::InspectDiskUsage,
                domain: "storage".to_string(),
                query_pattern: "inspect_disk_usage".to_string(),
                probes_used: vec!["df_root".to_string(), "lsblk".to_string()],
                probe_effectiveness: [("df_root".to_string(), 90), ("lsblk".to_string(), 75)]
                    .into_iter()
                    .collect(),
                docs_consulted: vec![],
                answer_confidence: 85,
                was_grounded: true,
                citations_used: vec![],
                timestamp: current_secs(),
                feedback: None,
            };
            store.record_ticket(record);
        }

        let proposals = store.analyze_and_propose();
        assert!(!proposals.is_empty());
        assert!(proposals[0].probes.contains(&"df_root".to_string()));
    }
}
