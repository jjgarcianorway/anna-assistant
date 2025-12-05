//! Ticket packet for domain-relevant evidence collection (v0.0.36).
//!
//! A TicketPacket bundles all evidence relevant to a specific domain query,
//! providing structured access to probe results, parsed data, and context.
//!
//! v0.0.40: Added MAX_PACKET_BYTES (8KB) limit for minimal packet size.

use crate::budget::ProbeBudget;
use crate::rpc::{ProbeResult, SpecialistDomain};
use crate::teams::Team;
use crate::trace::EvidenceKind;
use serde::{Deserialize, Serialize};

/// Maximum packet size in bytes (8KB) - v0.0.40
pub const MAX_PACKET_BYTES: usize = 8 * 1024;

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

/// Budget tracking for a packet
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PacketBudget {
    /// Number of probes planned
    pub probes_planned: usize,
    /// Number of probes executed
    pub probes_executed: usize,
    /// Number of probes that succeeded
    pub probes_succeeded: usize,
    /// Total bytes collected
    pub bytes_collected: usize,
    /// Whether budget was exceeded
    pub budget_exceeded: bool,
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
        self.probes.iter().find(|p| p.command.contains(command_contains))
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
        let target = MAX_PACKET_BYTES.saturating_sub(self.summary.len() + self.route_class.len() + 256);
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
        self.packet.set_budget_plan(count.min(self.budget.max_probes));
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
        if self.budget.would_exceed(self.packet.budget.bytes_collected, new_bytes) {
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

/// Recommended probes for each domain (v0.0.36)
pub fn recommended_probes_for_domain(domain: SpecialistDomain) -> Vec<&'static str> {
    match domain {
        SpecialistDomain::System => vec!["memory_info", "cpu_info", "failed_services"],
        SpecialistDomain::Storage => vec!["disk_usage", "block_devices"],
        SpecialistDomain::Network => vec!["network_addrs", "network_routes", "listening_ports"],
        SpecialistDomain::Security => vec!["failed_services", "listening_ports"],
        SpecialistDomain::Packages => vec![], // Uses package manager commands
    }
}

/// Recommended evidence kinds for each domain
pub fn evidence_kinds_for_domain(domain: SpecialistDomain) -> Vec<EvidenceKind> {
    match domain {
        SpecialistDomain::System => vec![EvidenceKind::Memory, EvidenceKind::Cpu, EvidenceKind::Services],
        SpecialistDomain::Storage => vec![EvidenceKind::Disk, EvidenceKind::BlockDevices],
        SpecialistDomain::Network => vec![], // Network evidence kind not defined yet
        SpecialistDomain::Security => vec![EvidenceKind::Services],
        SpecialistDomain::Packages => vec![],
    }
}

// === PacketPolicy per team (v0.0.36) ===

use crate::facts::FactKey;

/// Policy for what goes into a packet for a given team (v0.0.36)
#[derive(Debug, Clone)]
pub struct PacketPolicy {
    /// Team this policy applies to
    pub team: Team,
    /// Maximum lines in summary output
    pub max_summary_lines: usize,
    /// Allowed fact keys for this team
    pub allowed_facts: Vec<FactKey>,
    /// Required probes for this team
    pub required_probes: Vec<&'static str>,
    /// Maximum number of probes
    pub max_probes: usize,
}

impl Default for PacketPolicy {
    fn default() -> Self {
        Self {
            team: Team::General,
            max_summary_lines: 12,
            allowed_facts: vec![],
            required_probes: vec![],
            max_probes: 4,
        }
    }
}

impl PacketPolicy {
    /// Create policy for a team
    pub fn for_team(team: Team) -> Self {
        match team {
            Team::Desktop => Self {
                team,
                max_summary_lines: 10,
                allowed_facts: vec![FactKey::PreferredEditor],
                required_probes: vec!["failed_services"],
                max_probes: 3,
            },
            Team::Storage => Self {
                team,
                max_summary_lines: 12,
                allowed_facts: vec![],
                required_probes: vec!["disk_usage", "block_devices"],
                max_probes: 4,
            },
            Team::Network => Self {
                team,
                max_summary_lines: 12,
                allowed_facts: vec![FactKey::NetworkPrimaryInterface],
                required_probes: vec!["network_addrs"],
                max_probes: 4,
            },
            Team::Performance => Self {
                team,
                max_summary_lines: 15,
                allowed_facts: vec![],
                required_probes: vec!["memory_info", "cpu_info", "top_cpu"],
                max_probes: 5,
            },
            Team::Services => Self {
                team,
                max_summary_lines: 12,
                allowed_facts: vec![],
                required_probes: vec!["failed_services"],
                max_probes: 3,
            },
            Team::Security => Self {
                team,
                max_summary_lines: 10,
                allowed_facts: vec![],
                required_probes: vec!["failed_services", "listening_ports"],
                max_probes: 4,
            },
            Team::Hardware => Self {
                team,
                max_summary_lines: 12,
                allowed_facts: vec![],
                required_probes: vec!["cpu_info", "memory_info"],
                max_probes: 3,
            },
            Team::General => Self::default(),
        }
    }

    /// Truncate summary to max lines deterministically
    pub fn truncate_summary(&self, summary: &str) -> String {
        let lines: Vec<&str> = summary.lines().collect();
        if lines.len() <= self.max_summary_lines {
            return summary.to_string();
        }

        let kept = self.max_summary_lines - 1;
        let omitted = lines.len() - kept;
        let mut result: Vec<&str> = lines.into_iter().take(kept).collect();
        result.push(&format!("({} more lines omitted)", omitted));

        // Need to create the string without borrowing
        let truncated: Vec<String> = summary
            .lines()
            .take(self.max_summary_lines - 1)
            .map(|s| s.to_string())
            .collect();
        let omit_count = summary.lines().count() - (self.max_summary_lines - 1);

        format!("{}\n({} more lines omitted)", truncated.join("\n"), omit_count)
    }

    /// Check if a fact key is allowed for this team
    pub fn is_fact_allowed(&self, key: &FactKey) -> bool {
        self.allowed_facts.contains(key)
    }
}

/// Get policy for a team
pub fn policy_for_team(team: Team) -> PacketPolicy {
    PacketPolicy::for_team(team)
}

// Tests moved to tests/ticket_packet_tests.rs
