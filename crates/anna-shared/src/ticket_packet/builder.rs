//! TicketPacketBuilder for creating domain-specific packets (v0.0.216).

use crate::budget::ProbeBudget;
use crate::rpc::{ProbeResult, SpecialistDomain};
use crate::teams::Team;
use crate::trace::EvidenceKind;

use super::packet::TicketPacket;

/// Builder for creating domain-specific packets
pub struct TicketPacketBuilder {
    packet: TicketPacket,
    budget: ProbeBudget,
}

impl TicketPacketBuilder {
    /// Create a builder for a domain
    pub fn new(route_class: &str, domain: SpecialistDomain, team: Team) -> Self {
        Self {
            packet: TicketPacket::new(route_class, domain, team),
            budget: ProbeBudget::default(),
        }
    }

    /// Use fast path budget
    pub fn fast_path(mut self) -> Self {
        self.budget = ProbeBudget::fast_path();
        self
    }

    /// Use standard budget
    pub fn standard(mut self) -> Self {
        self.budget = ProbeBudget::standard();
        self
    }

    /// Use extended budget
    pub fn extended(mut self) -> Self {
        self.budget = ProbeBudget::extended();
        self
    }

    /// Plan probes for collection
    pub fn plan_probes(mut self, count: usize) -> Self {
        self.packet
            .set_budget_plan(count.min(self.budget.max_probes));
        self
    }

    /// Add evidence kind
    pub fn with_evidence(mut self, kind: EvidenceKind) -> Self {
        self.packet.add_evidence_kind(kind);
        self
    }

    /// Add a probe result (respects budget)
    pub fn add_probe(mut self, result: ProbeResult) -> Self {
        // Check budget before adding
        if self.packet.budget.probes_executed >= self.budget.max_probes {
            self.packet.mark_budget_exceeded();
            return self;
        }

        let new_bytes = result.stdout.len() + result.stderr.len();
        if self
            .budget
            .would_exceed(self.packet.budget.bytes_collected, new_bytes)
        {
            self.packet.mark_budget_exceeded();
            return self;
        }

        self.packet.add_probe(result);
        self
    }

    /// Build the final packet (v0.0.40: enforces MAX_PACKET_BYTES limit)
    pub fn build(mut self) -> TicketPacket {
        self.packet.build_summary();
        // Enforce 8KB limit
        self.packet.truncate_to_limit();
        self.packet
    }
}
