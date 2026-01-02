// v0.0.539: Team Consultation Tracker - Types
// Department/team types, outcomes, and seniority levels

use serde::{Deserialize, Serialize};

/// Department/team type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TeamDepartment {
    Network,
    Storage,
    Audio,
    Video,
    Desktop,
    Security,
    Package,
    Service,
    Shell,
    Hardware,
    Kernel,
    Custom(String),
}

impl Default for TeamDepartment {
    fn default() -> Self {
        Self::Desktop
    }
}

impl std::fmt::Display for TeamDepartment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network => write!(f, "Network"),
            Self::Storage => write!(f, "Storage"),
            Self::Audio => write!(f, "Audio"),
            Self::Video => write!(f, "Video"),
            Self::Desktop => write!(f, "Desktop"),
            Self::Security => write!(f, "Security"),
            Self::Package => write!(f, "Package"),
            Self::Service => write!(f, "Service"),
            Self::Shell => write!(f, "Shell"),
            Self::Hardware => write!(f, "Hardware"),
            Self::Kernel => write!(f, "Kernel"),
            Self::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Consultation outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ConsultationOutcome {
    #[default]
    Pending,
    Resolved,
    Escalated,
    Deferred,
    Failed,
}

impl std::fmt::Display for ConsultationOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Resolved => write!(f, "Resolved"),
            Self::Escalated => write!(f, "Escalated"),
            Self::Deferred => write!(f, "Deferred"),
            Self::Failed => write!(f, "Failed"),
        }
    }
}

/// Seniority level consulted
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum SeniorityConsulted {
    #[default]
    Junior,
    Senior,
    Both,
}

impl std::fmt::Display for SeniorityConsulted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Junior => write!(f, "Junior"),
            Self::Senior => write!(f, "Senior"),
            Self::Both => write!(f, "Both"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_department_default() {
        let dept = TeamDepartment::default();
        assert_eq!(dept, TeamDepartment::Desktop);
    }

    #[test]
    fn test_outcome_default() {
        let outcome = ConsultationOutcome::default();
        assert_eq!(outcome, ConsultationOutcome::Pending);
    }
}
