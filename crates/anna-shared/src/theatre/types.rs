//! Theatre types - Speaker and NarrativeSegment (v0.0.226).
//! v0.0.451: Added metadata for fly-on-the-wall rendering per VISION.md.

use std::collections::HashMap;

use crate::roster::{person_for, Tier};
use crate::teams::Team;

/// A segment of narrative dialogue for streaming display.
/// v0.0.451: Added metadata field for "to" recipient in fly-on-the-wall view.
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
    /// v0.0.451: Additional metadata (e.g., "to" for recipient)
    pub metadata: Option<HashMap<String, String>>,
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
            metadata: None,
        }
    }

    /// Create Anna speaking internally (to team)
    pub fn anna_internal(text: impl Into<String>) -> Self {
        Self {
            speaker: Speaker::Anna,
            text: text.into(),
            delay_ms: 100,
            internal: true,
            metadata: None,
        }
    }

    /// v0.0.451: Create Anna speaking to a specific recipient (fly-on-the-wall)
    pub fn anna_to(recipient: &str, text: impl Into<String>) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("to".to_string(), recipient.to_string());
        Self {
            speaker: Speaker::Anna,
            text: text.into(),
            delay_ms: 100,
            internal: true,
            metadata: Some(metadata),
        }
    }

    /// Create a team member speaking
    pub fn team_member(team: Team, tier: Tier, text: impl Into<String>) -> Self {
        Self {
            speaker: Speaker::from_team(team, tier),
            text: text.into(),
            delay_ms: 150,
            internal: true,
            metadata: None,
        }
    }

    /// Create user speaking
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            speaker: Speaker::You,
            text: text.into(),
            delay_ms: 0,
            internal: false,
            metadata: None,
        }
    }

    /// Create narrator text (stage transitions, etc.)
    pub fn narrator(text: impl Into<String>) -> Self {
        Self {
            speaker: Speaker::Narrator,
            text: text.into(),
            delay_ms: 50,
            internal: false,
            metadata: None,
        }
    }

    /// Set custom delay
    pub fn with_delay(mut self, ms: u32) -> Self {
        self.delay_ms = ms;
        self
    }

    /// v0.0.451: Set recipient for fly-on-the-wall rendering
    pub fn with_to(mut self, recipient: &str) -> Self {
        let metadata = self.metadata.get_or_insert_with(HashMap::new);
        metadata.insert("to".to_string(), recipient.to_string());
        self
    }
}
