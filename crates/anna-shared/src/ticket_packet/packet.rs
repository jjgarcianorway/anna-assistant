//! TicketPacket struct definition (v0.0.216).

use serde::{Deserialize, Serialize};

use crate::rpc::{ProbeResult, SpecialistDomain};
use crate::teams::Team;
use crate::trace::EvidenceKind;

use super::types::{PacketBudget, MAX_PACKET_BYTES};

/// A packet of evidence for a ticket (v0.0.36)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TicketPacket {
    /// Query class that triggered this packet
    pub route_class: String,
    /// Domain for the query
    pub domain: SpecialistDomain,
    /// Team assigned to handle this query
    pub team: Team,
    /// Evidence kinds collected
    pub evidence_kinds: Vec<EvidenceKind>,
    /// Probe results collected
    pub probes: Vec<ProbeResult>,
    /// Budget used for probes
    pub budget: PacketBudget,
    /// Summary of evidence (for display)
    pub summary: String,
}

impl TicketPacket {
    /// Create a new packet for a domain query
    pub fn new(route_class: &str, domain: SpecialistDomain, team: Team) -> Self {
        Self {
            route_class: route_class.to_string(),
            domain,
            team,
            evidence_kinds: Vec::new(),
            probes: Vec::new(),
            budget: PacketBudget::default(),
            summary: String::new(),
        }
    }

    /// Add a probe result to the packet
    pub fn add_probe(&mut self, result: ProbeResult) {
        self.budget.bytes_collected += result.stdout.len() + result.stderr.len();
        self.budget.probes_executed += 1;
        if result.exit_code == 0 {
            self.budget.probes_succeeded += 1;
        }
        self.probes.push(result);
    }

    /// Add evidence kind
    pub fn add_evidence_kind(&mut self, kind: EvidenceKind) {
        if !self.evidence_kinds.contains(&kind) {
            self.evidence_kinds.push(kind);
        }
    }

    /// Set the budget plan
    pub fn set_budget_plan(&mut self, probes_planned: usize) {
        self.budget.probes_planned = probes_planned;
    }

    /// Mark budget as exceeded
    pub fn mark_budget_exceeded(&mut self) {
        self.budget.budget_exceeded = true;
    }

    /// Check if all planned probes succeeded
    pub fn all_probes_succeeded(&self) -> bool {
        self.budget.probes_succeeded == self.budget.probes_planned
    }

    /// Get probe success rate (0.0-1.0)
    pub fn probe_success_rate(&self) -> f32 {
        if self.budget.probes_executed == 0 {
            0.0
        } else {
            self.budget.probes_succeeded as f32 / self.budget.probes_executed as f32
        }
    }

    /// Build summary from collected evidence
    pub fn build_summary(&mut self) {
        let kinds: Vec<String> = self.evidence_kinds.iter().map(|k| k.to_string()).collect();
        self.summary = format!(
            "{} probes ({} succeeded), {} bytes | Evidence: [{}]",
            self.budget.probes_executed,
            self.budget.probes_succeeded,
            self.budget.bytes_collected,
            kinds.join(", ")
        );
    }

    /// Find a probe result by command substring
    pub fn find_probe(&self, command_contains: &str) -> Option<&ProbeResult> {
        self.probes
            .iter()
            .find(|p| p.command.contains(command_contains))
    }

    /// Get all successful probes
    pub fn successful_probes(&self) -> Vec<&ProbeResult> {
        self.probes.iter().filter(|p| p.exit_code == 0).collect()
    }

    /// Check if packet has any evidence
    pub fn has_evidence(&self) -> bool {
        !self.probes.is_empty() || !self.evidence_kinds.is_empty()
    }

    /// Estimate packet size in bytes (v0.0.40)
    pub fn estimated_size(&self) -> usize {
        self.budget.bytes_collected + self.summary.len() + self.route_class.len()
    }

    /// Check if packet exceeds MAX_PACKET_BYTES limit (v0.0.40)
    pub fn exceeds_limit(&self) -> bool {
        self.estimated_size() > MAX_PACKET_BYTES
    }

    /// Truncate probe outputs to fit within MAX_PACKET_BYTES (v0.0.40)
    pub fn truncate_to_limit(&mut self) {
        if !self.exceeds_limit() {
            return;
        }

        // Truncate probe outputs proportionally
        let target =
            MAX_PACKET_BYTES.saturating_sub(self.summary.len() + self.route_class.len() + 256);
        let per_probe = if self.probes.is_empty() {
            0
        } else {
            target / self.probes.len()
        };

        for probe in &mut self.probes {
            if probe.stdout.len() > per_probe {
                probe.stdout = format!(
                    "{}...(truncated {} bytes)",
                    &probe.stdout[..per_probe.saturating_sub(30)],
                    probe.stdout.len() - per_probe
                );
            }
            if probe.stderr.len() > per_probe / 4 {
                let limit = per_probe / 4;
                probe.stderr = format!(
                    "{}...(truncated)",
                    &probe.stderr[..limit.saturating_sub(20)]
                );
            }
        }

        // Recalculate bytes collected
        self.budget.bytes_collected = self
            .probes
            .iter()
            .map(|p| p.stdout.len() + p.stderr.len())
            .sum();
    }
}
