//! PacketPolicy for team-specific packet configuration (v0.0.216).

use crate::facts::FactKey;
use crate::teams::Team;

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
            // v0.0.401: Increased from 12 to 100 to avoid truncating useful output
            max_summary_lines: 100,
            allowed_facts: vec![],
            required_probes: vec![],
            max_probes: 4,
        }
    }
}

impl PacketPolicy {
    /// Create policy for a team
    /// v0.0.401: Increased max_summary_lines from 10-12 to 100 to avoid truncation
    pub fn for_team(team: Team) -> Self {
        match team {
            Team::Desktop => Self {
                team,
                max_summary_lines: 100,
                allowed_facts: vec![FactKey::PreferredEditor],
                required_probes: vec!["failed_services"],
                max_probes: 3,
            },
            Team::Storage => Self {
                team,
                max_summary_lines: 100,
                allowed_facts: vec![],
                required_probes: vec!["disk_usage", "block_devices"],
                max_probes: 4,
            },
            Team::Network => Self {
                team,
                max_summary_lines: 100,
                allowed_facts: vec![FactKey::NetworkPrimaryInterface],
                required_probes: vec!["network_addrs"],
                max_probes: 4,
            },
            Team::Performance => Self {
                team,
                max_summary_lines: 100,
                allowed_facts: vec![],
                required_probes: vec!["memory_info", "cpu_info", "top_cpu"],
                max_probes: 5,
            },
            Team::Services => Self {
                team,
                max_summary_lines: 100,
                allowed_facts: vec![],
                required_probes: vec!["failed_services"],
                max_probes: 3,
            },
            Team::Security => Self {
                team,
                max_summary_lines: 100,
                allowed_facts: vec![],
                required_probes: vec!["failed_services", "listening_ports"],
                max_probes: 4,
            },
            Team::Hardware => Self {
                team,
                max_summary_lines: 100,
                allowed_facts: vec![],
                required_probes: vec!["cpu_info", "memory_info"],
                max_probes: 3,
            },
            Team::Logs => Self {
                team,
                max_summary_lines: 100,
                allowed_facts: vec![],
                required_probes: vec!["journal_errors"],
                max_probes: 4,
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

        format!(
            "{}\n({} more lines omitted)",
            truncated.join("\n"),
            omit_count
        )
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
