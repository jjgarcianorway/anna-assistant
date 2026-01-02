//! Type definitions for ticket resolution tracking

use serde::{Deserialize, Serialize};

/// Who resolved the ticket
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Resolver {
    #[default]
    Anna,
    Junior,
    Senior,
    Escalated,
    User,
    Unknown,
}

impl Resolver {
    pub fn name(&self) -> &'static str {
        match self {
            Resolver::Anna => "Anna",
            Resolver::Junior => "Junior",
            Resolver::Senior => "Senior",
            Resolver::Escalated => "Escalated",
            Resolver::User => "User",
            Resolver::Unknown => "Unknown",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Resolver::Anna => "A",
            Resolver::Junior => "J",
            Resolver::Senior => "S",
            Resolver::Escalated => "E",
            Resolver::User => "U",
            Resolver::Unknown => "?",
        }
    }

    pub fn is_specialist(&self) -> bool {
        matches!(self, Resolver::Junior | Resolver::Senior | Resolver::Escalated)
    }
}

/// Resolution method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ResolutionMethod {
    #[default]
    Recipe,
    Specialist,
    DirectAnswer,
    UserSelfHelp,
    Escalation,
    Timeout,
}

impl ResolutionMethod {
    pub fn name(&self) -> &'static str {
        match self {
            ResolutionMethod::Recipe => "Recipe",
            ResolutionMethod::Specialist => "Specialist",
            ResolutionMethod::DirectAnswer => "Direct Answer",
            ResolutionMethod::UserSelfHelp => "User Self-Help",
            ResolutionMethod::Escalation => "Escalation",
            ResolutionMethod::Timeout => "Timeout",
        }
    }
}

/// A resolution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionRecord {
    /// Ticket ID
    pub ticket_id: String,
    /// Who resolved it
    pub resolver: Resolver,
    /// Method used
    pub method: ResolutionMethod,
    /// Department/team involved
    pub department: Option<String>,
    /// Specialist name (if specialist)
    pub specialist_name: Option<String>,
    /// Resolution timestamp
    pub resolved_at: u64,
    /// Time to resolution (seconds)
    pub resolution_time_secs: u64,
    /// Was a recipe learned from this?
    pub recipe_learned: bool,
    /// Confidence score
    pub confidence: Option<u8>,
}
