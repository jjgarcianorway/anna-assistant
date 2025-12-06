//! Service Desk Theatre - Cinematic narrative rendering (v0.0.81).
//!
//! Transforms technical pipeline events into natural dialogue between
//! named IT department personas. Makes users feel like they're watching
//! a real IT team solve their problems.
//!
//! Two modes:
//! - Normal: Cinematic narrative with named personas
//! - Debug: Full technical pipeline visibility

use crate::roster::{person_for, Tier};
use crate::teams::Team;

/// A segment of narrative dialogue for streaming display.
#[derive(Debug, Clone)]
pub struct NarrativeSegment {
    /// Who is speaking (Anna, Michael, Sofia, etc.)
    pub speaker: Speaker,
    /// The dialogue text
    pub text: String,
    /// Suggested delay before showing (ms) - for theatrical pacing
    pub delay_ms: u32,
    /// Is this an internal IT communication? (shown differently)
    pub internal: bool,
}

/// Who is speaking in the narrative
#[derive(Debug, Clone, PartialEq)]
pub enum Speaker {
    /// Anna - the front desk / service coordinator
    Anna,
    /// The user
    You,
    /// A named team member (with their profile)
    TeamMember {
        name: String,
        role: String,
        team: String,
    },
    /// System narrator (for stage transitions)
    Narrator,
}

impl Speaker {
    /// Create a team member speaker from team + tier
    pub fn from_team(team: Team, tier: Tier) -> Self {
        let person = person_for(team, tier);
        Speaker::TeamMember {
            name: person.display_name.to_string(),
            role: person.role_title.to_string(),
            team: format!("{:?}", team),
        }
    }

    /// Get display name for the speaker
    pub fn display_name(&self) -> &str {
        match self {
            Speaker::Anna => "Anna",
            Speaker::You => "you",
            Speaker::TeamMember { name, .. } => name,
            Speaker::Narrator => "",
        }
    }

    /// Get full display with role
    pub fn display_with_role(&self) -> String {
        match self {
            Speaker::Anna => "Anna (Service Desk)".to_string(),
            Speaker::You => "you".to_string(),
            Speaker::TeamMember { name, role, .. } => format!("{} ({})", name, role),
            Speaker::Narrator => String::new(),
        }
    }
}

impl NarrativeSegment {
    /// Create Anna speaking
    pub fn anna(text: impl Into<String>) -> Self {
        Self {
            speaker: Speaker::Anna,
            text: text.into(),
            delay_ms: 0,
            internal: false,
        }
    }

    /// Create Anna speaking internally (to team)
    pub fn anna_internal(text: impl Into<String>) -> Self {
        Self {
            speaker: Speaker::Anna,
            text: text.into(),
            delay_ms: 100,
            internal: true,
        }
    }

    /// Create a team member speaking
    pub fn team_member(team: Team, tier: Tier, text: impl Into<String>) -> Self {
        Self {
            speaker: Speaker::from_team(team, tier),
            text: text.into(),
            delay_ms: 150,
            internal: true,
        }
    }

    /// Create user speaking
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            speaker: Speaker::You,
            text: text.into(),
            delay_ms: 0,
            internal: false,
        }
    }

    /// Create narrator text (stage transitions, etc.)
    pub fn narrator(text: impl Into<String>) -> Self {
        Self {
            speaker: Speaker::Narrator,
            text: text.into(),
            delay_ms: 50,
            internal: false,
        }
    }

    /// Set custom delay
    pub fn with_delay(mut self, ms: u32) -> Self {
        self.delay_ms = ms;
        self
    }
}

/// Builder for constructing narrative from pipeline events
#[derive(Debug, Default)]
pub struct NarrativeBuilder {
    segments: Vec<NarrativeSegment>,
    current_team: Option<Team>,
    show_internal: bool,
}

impl NarrativeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable showing internal IT communications
    pub fn with_internal_comms(mut self) -> Self {
        self.show_internal = true;
        self
    }

    /// Add Anna greeting based on domain
    pub fn add_greeting(&mut self, domain: &str) {
        let greeting = match domain.to_lowercase().as_str() {
            "storage" | "disk" => "Let me check that storage information for you.",
            "memory" | "ram" => "I'll look into the memory right away.",
            "network" | "wifi" => "Let me examine your network configuration.",
            "performance" | "cpu" | "slow" => "I'll analyze the system performance.",
            "service" | "services" => "Let me check those service statuses.",
            "security" => "I'll review the security information carefully.",
            "hardware" | "audio" => "Let me gather the hardware details.",
            "desktop" | "editor" => "I'll check that for you.",
            _ => "Let me look into that for you.",
        };
        self.segments.push(NarrativeSegment::anna(greeting));
    }

    /// Add probe activity narration
    pub fn add_checking(&mut self, description: &str) {
        self.segments.push(NarrativeSegment::narrator(
            format!("Checking {}...", description)
        ));
    }

    /// Add Anna dispatching to a team
    pub fn add_dispatch(&mut self, team: Team, case_id: &str) {
        self.current_team = Some(team);

        if self.show_internal {
            let person = person_for(team, Tier::Junior);
            self.segments.push(NarrativeSegment::anna_internal(
                format!(
                    "Hey {}! I have a case for you. {}",
                    person.display_name,
                    &case_id[..8.min(case_id.len())]
                )
            ));
        }
    }

    /// Add junior review narration
    pub fn add_junior_review(&mut self, team: Team, approved: bool, score: u8) {
        let person = person_for(team, Tier::Junior);

        if self.show_internal {
            let response = if approved {
                format!(
                    "Got it, Anna. I've reviewed the data. Looks good, confidence {}%.",
                    score
                )
            } else {
                format!(
                    "Anna, I need to escalate this one. Score is {}%, not confident enough.",
                    score
                )
            };
            self.segments.push(NarrativeSegment::team_member(team, Tier::Junior, response));
        }
    }

    /// Add senior escalation narration
    pub fn add_escalation(&mut self, team: Team, reason: &str) {
        if self.show_internal {
            let senior = person_for(team, Tier::Senior);
            self.segments.push(NarrativeSegment::team_member(
                team, Tier::Junior,
                format!("{}, can you take a look? {}", senior.display_name, reason)
            ));
        }
    }

    /// Add senior response
    pub fn add_senior_response(&mut self, team: Team, guidance: &str) {
        if self.show_internal {
            self.segments.push(NarrativeSegment::team_member(
                team, Tier::Senior,
                guidance.to_string()
            ));
        }
    }

    /// Add Anna's apology for wait time
    pub fn add_wait_apology(&mut self) {
        self.segments.push(NarrativeSegment::anna(
            "Apologies for the wait. I've consulted with the team."
        ));
    }

    /// Add Anna presenting the answer
    pub fn add_answer_intro(&mut self, confidence: u8) {
        let intro = match confidence {
            90..=100 => "Here's what I found:",
            80..=89 => "Based on my checks:",
            70..=79 => "From what I can tell:",
            _ => "I found some information, though I'm not fully certain:",
        };
        self.segments.push(NarrativeSegment::anna(intro));
    }

    /// Add clarification request narration
    pub fn add_clarification(&mut self, question: &str) {
        self.segments.push(NarrativeSegment::anna(question.to_string()));
    }

    /// Build the final narrative
    pub fn build(self) -> Vec<NarrativeSegment> {
        self.segments
    }

    /// Get segment count
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }
}

/// Describes what probes are checking in human-friendly terms
pub fn describe_check(probe_ids: &[String]) -> String {
    let mut checks = Vec::new();

    for id in probe_ids {
        let desc = match id.as_str() {
            "df" => "disk space",
            "free" => "memory",
            "lscpu" => "CPU info",
            "sensors" => "temperatures",
            "systemctl" | "systemctl_failed" => "services",
            "journalctl_errors" | "journalctl_warnings" => "system logs",
            "ip_addr" | "ip" => "network interfaces",
            "ss" | "listening_ports" => "network ports",
            "lspci_audio" | "pactl_cards" => "audio hardware",
            "lsblk" => "block devices",
            "uname" => "kernel info",
            "top_cpu" => "CPU usage",
            "top_memory" => "memory usage",
            "command_v" => "installed tools",
            _ if id.contains("editor") => "editors",
            _ => continue,
        };
        if !checks.contains(&desc) {
            checks.push(desc);
        }
    }

    if checks.is_empty() {
        "system data".to_string()
    } else {
        checks.join(", ")
    }
}

/// Format case ID in service desk style
pub fn format_case_id(request_id: &str) -> String {
    let short = &request_id[..8.min(request_id.len())];
    format!("CN-{}", short.to_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speaker_display() {
        assert_eq!(Speaker::Anna.display_name(), "Anna");
        assert_eq!(Speaker::You.display_name(), "you");

        let member = Speaker::from_team(Team::Network, Tier::Junior);
        assert_eq!(member.display_name(), "Michael");
    }

    #[test]
    fn test_speaker_display_with_role() {
        let member = Speaker::from_team(Team::Storage, Tier::Senior);
        assert_eq!(member.display_with_role(), "Ines (Storage Architect)");
    }

    #[test]
    fn test_narrative_segment_anna() {
        let seg = NarrativeSegment::anna("Hello!");
        assert_eq!(seg.speaker, Speaker::Anna);
        assert!(!seg.internal);
    }

    #[test]
    fn test_narrative_segment_team() {
        let seg = NarrativeSegment::team_member(Team::Desktop, Tier::Junior, "On it!");
        if let Speaker::TeamMember { name, .. } = &seg.speaker {
            assert_eq!(name, "Sofia");
        } else {
            panic!("Expected TeamMember speaker");
        }
        assert!(seg.internal);
    }

    #[test]
    fn test_narrative_builder() {
        let mut builder = NarrativeBuilder::new().with_internal_comms();
        builder.add_greeting("storage");
        builder.add_checking("disk space");
        builder.add_dispatch(Team::Storage, "abc12345");

        let narrative = builder.build();
        assert_eq!(narrative.len(), 3);
    }

    #[test]
    fn test_describe_check() {
        let probes = vec!["df".to_string(), "free".to_string()];
        let desc = describe_check(&probes);
        assert!(desc.contains("disk"));
        assert!(desc.contains("memory"));
    }

    #[test]
    fn test_format_case_id() {
        let case = format_case_id("abc123456789");
        assert_eq!(case, "CN-ABC12345");
    }

    #[test]
    fn test_empty_describe_check() {
        let probes: Vec<String> = vec![];
        let desc = describe_check(&probes);
        assert_eq!(desc, "system data");
    }
}
