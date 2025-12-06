//! Presentation events for UX realism layer (v0.0.75).
//!
//! Internal protocol for streaming presentation updates from annad to annactl.
//! Shows progressive updates without leaking raw probes when debug is OFF.

use serde::{Deserialize, Serialize};

/// Presentation event types for user-facing updates
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PresentationEvent {
    /// Request started
    RequestStarted { request_id: String },

    /// Ticket created and assigned to a team
    TicketCreated {
        ticket_id: String,
        team: String,
        summary: String,
    },

    /// Stage started (translator, probes, specialist, etc.)
    StageStart {
        stage: PresentationStage,
        description: String,
    },

    /// Stage completed
    StageDone {
        stage: PresentationStage,
        outcome: StageOutcome,
        duration_ms: u64,
    },

    /// Evidence gathered (summarized, not raw)
    EvidenceGathered {
        source_count: usize,
        summary: String,
    },

    /// Ticket escalated to higher tier
    Escalated {
        from_role: String,
        to_role: String,
        reason: String,
    },

    /// Waiting for user confirmation
    WaitingConfirmation {
        action_id: String,
        description: String,
    },

    /// Recipe being applied
    RecipeApplied {
        recipe_id: String,
        description: String,
    },

    /// Request completed
    RequestComplete {
        success: bool,
        reliability_score: u8,
    },

    /// Heartbeat for long operations
    Heartbeat { elapsed_ms: u64 },
}

/// Pipeline stages for presentation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PresentationStage {
    /// Understanding the request
    Understanding,
    /// Gathering evidence (probes)
    Gathering,
    /// Analyzing findings
    Analyzing,
    /// Preparing response
    Preparing,
    /// Validating answer
    Validating,
}

impl PresentationStage {
    /// Get user-friendly description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Understanding => "Understanding your request",
            Self::Gathering => "Gathering system information",
            Self::Analyzing => "Analyzing findings",
            Self::Preparing => "Preparing response",
            Self::Validating => "Validating answer",
        }
    }

    /// Get spinner message
    pub fn spinner_message(&self) -> &'static str {
        match self {
            Self::Understanding => "Reviewing your request...",
            Self::Gathering => "Checking system status...",
            Self::Analyzing => "Reviewing the evidence...",
            Self::Preparing => "Drafting response...",
            Self::Validating => "Verifying accuracy...",
        }
    }
}

/// Stage outcome for presentation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StageOutcome {
    Success,
    NeedsMoreInfo,
    Escalated,
    Timeout,
    Error { message: String },
}

/// Technician persona by domain (v0.0.75 role-play realism)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Technician {
    pub name: String,
    pub role: String,
    pub domain: String,
}

impl Technician {
    /// Get technician for a domain
    pub fn for_domain(domain: &str) -> Self {
        match domain.to_lowercase().as_str() {
            "network" => Self {
                name: "Alex".to_string(),
                role: "Network Technician".to_string(),
                domain: "Network".to_string(),
            },
            "desktop" | "system" => Self {
                name: "Sam".to_string(),
                role: "Desktop Support".to_string(),
                domain: "Desktop".to_string(),
            },
            "audio" | "hardware" => Self {
                name: "Jordan".to_string(),
                role: "Hardware Specialist".to_string(),
                domain: "Hardware".to_string(),
            },
            "performance" | "memory" | "cpu" => Self {
                name: "Casey".to_string(),
                role: "Performance Analyst".to_string(),
                domain: "Performance".to_string(),
            },
            "storage" | "disk" => Self {
                name: "Morgan".to_string(),
                role: "Storage Engineer".to_string(),
                domain: "Storage".to_string(),
            },
            "security" => Self {
                name: "Riley".to_string(),
                role: "Security Analyst".to_string(),
                domain: "Security".to_string(),
            },
            _ => Self {
                name: "Taylor".to_string(),
                role: "IT Support".to_string(),
                domain: "General".to_string(),
            },
        }
    }

    /// Get display string for technician
    pub fn display(&self) -> String {
        format!("{} ({})", self.name, self.role)
    }
}

/// Presentation state accumulator
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PresentationState {
    /// Accumulated events
    pub events: Vec<PresentationEvent>,
    /// Current stage
    pub current_stage: Option<PresentationStage>,
    /// Assigned technician
    pub technician: Option<Technician>,
    /// Whether escalation occurred
    pub escalated: bool,
    /// Total elapsed time
    pub elapsed_ms: u64,
}

impl PresentationState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, event: PresentationEvent) {
        // Update state based on event
        match &event {
            PresentationEvent::StageStart { stage, .. } => {
                self.current_stage = Some(*stage);
            }
            PresentationEvent::StageDone { duration_ms, .. } => {
                self.elapsed_ms += duration_ms;
            }
            PresentationEvent::Escalated { .. } => {
                self.escalated = true;
            }
            _ => {}
        }
        self.events.push(event);
    }

    /// Get last N events
    pub fn last_events(&self, n: usize) -> &[PresentationEvent] {
        let start = self.events.len().saturating_sub(n);
        &self.events[start..]
    }
}

/// Role hierarchy for escalation (junior -> senior -> manager)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TechnicianTier {
    Junior,
    Senior,
    Manager,
}

impl TechnicianTier {
    pub fn display(&self) -> &'static str {
        match self {
            Self::Junior => "Junior Technician",
            Self::Senior => "Senior Technician",
            Self::Manager => "Team Lead",
        }
    }

    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Junior => Some(Self::Senior),
            Self::Senior => Some(Self::Manager),
            Self::Manager => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_technician_for_domain() {
        let tech = Technician::for_domain("network");
        assert_eq!(tech.name, "Alex");
        assert_eq!(tech.domain, "Network");
    }

    #[test]
    fn test_presentation_stage_messages() {
        assert!(!PresentationStage::Understanding.description().is_empty());
        assert!(!PresentationStage::Gathering.spinner_message().is_empty());
    }

    #[test]
    fn test_presentation_state_accumulation() {
        let mut state = PresentationState::new();
        state.push(PresentationEvent::StageStart {
            stage: PresentationStage::Understanding,
            description: "Test".to_string(),
        });
        assert_eq!(state.current_stage, Some(PresentationStage::Understanding));
        assert_eq!(state.events.len(), 1);
    }

    #[test]
    fn test_tier_escalation() {
        assert_eq!(TechnicianTier::Junior.next(), Some(TechnicianTier::Senior));
        assert_eq!(TechnicianTier::Senior.next(), Some(TechnicianTier::Manager));
        assert_eq!(TechnicianTier::Manager.next(), None);
    }
}
