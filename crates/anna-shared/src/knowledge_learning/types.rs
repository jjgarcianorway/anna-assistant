//! Knowledge Learning System Types
//!
//! Core data structures for the learning system.

use crate::intent_policy::IntentCategory;
use crate::knowledge_query::KnowledgeSourceKind;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
