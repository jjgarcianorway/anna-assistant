//! Knowledge Learning Utilities
//!
//! Helper functions for creating learning records.

use super::types::{DocReference, SolvedTicketRecord};
use crate::doc_first_workflow::{CitedAnswer, SpecialistEvidence};
use crate::intent_policy::IntentCategory;
use std::collections::HashMap;

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

fn current_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
