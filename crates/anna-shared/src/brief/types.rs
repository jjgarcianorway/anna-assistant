//! Brief types and structures (v0.0.229).

use crate::rpc::ProbeResult;
use crate::trace::EvidenceKind;
use serde::{Deserialize, Serialize};

use super::filtering::{evidence_kind_for_probe, is_probe_relevant};
use crate::teams::Team;

/// Filtered view of a ticket for team review (v0.0.32)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketBrief {
    /// Original user request
    pub user_request: String,
    /// Domain classification
    pub domain: String,
    /// Intent classification
    pub intent: String,
    /// Route class
    pub route_class: String,
    /// Filtered probe results (only team-relevant)
    pub relevant_probes: Vec<ProbeResult>,
    /// Evidence kinds present
    pub evidence_kinds: Vec<EvidenceKind>,
    /// Count of probes filtered out
    pub filtered_count: usize,
    /// Any facts learned during ticket processing
    pub facts_learned: Vec<String>,
}

impl TicketBrief {
    /// Build a brief from ticket data and probe results
    pub fn build(
        user_request: &str,
        domain: &str,
        intent: &str,
        route_class: &str,
        team: Team,
        probe_results: &[ProbeResult],
        facts_learned: &[String],
    ) -> Self {
        let relevant: Vec<ProbeResult> = probe_results
            .iter()
            .filter(|p| is_probe_relevant(&p.command, team))
            .cloned()
            .collect();

        let filtered_count = probe_results.len() - relevant.len();

        // Collect unique evidence kinds from relevant probes
        let mut evidence_kinds: Vec<EvidenceKind> = relevant
            .iter()
            .filter_map(|p| evidence_kind_for_probe(&p.command))
            .collect();
        evidence_kinds.sort_by_key(|k| k.to_string());
        evidence_kinds.dedup();

        Self {
            user_request: user_request.to_string(),
            domain: domain.to_string(),
            intent: intent.to_string(),
            route_class: route_class.to_string(),
            relevant_probes: relevant,
            evidence_kinds,
            filtered_count,
            facts_learned: facts_learned.to_vec(),
        }
    }

    /// Check if brief has any evidence
    pub fn has_evidence(&self) -> bool {
        !self.relevant_probes.is_empty()
    }

    /// Get summary line for debug output
    pub fn summary(&self) -> String {
        let kinds: Vec<_> = self.evidence_kinds.iter().map(|k| k.to_string()).collect();
        if kinds.is_empty() {
            format!("{} probes (none classified)", self.relevant_probes.len())
        } else {
            format!(
                "{} probes ({}), {} filtered",
                self.relevant_probes.len(),
                kinds.join(", "),
                self.filtered_count
            )
        }
    }
}

/// Build brief from a ticket (convenience wrapper)
pub fn build_brief_from_ticket(
    ticket: &crate::ticket::Ticket,
    probe_results: &[ProbeResult],
) -> TicketBrief {
    TicketBrief::build(
        &ticket.user_request,
        &ticket.domain,
        &ticket.intent,
        &ticket.route_class,
        ticket.team,
        probe_results,
        &ticket.facts_learned,
    )
}
