// v0.0.712: Settings Report Types (Phase 288)
// Report type and frequency enums

use serde::{Deserialize, Serialize};

/// Report type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ReportType {
    /// Status report
    #[default]
    Status,
    /// Change report
    Change,
    /// Audit report
    Audit,
    /// Compliance report
    Compliance,
}

impl std::fmt::Display for ReportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Status => write!(f, "status"),
            Self::Change => write!(f, "change"),
            Self::Audit => write!(f, "audit"),
            Self::Compliance => write!(f, "compliance"),
        }
    }
}

/// Report frequency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ReportFrequency {
    /// On-demand
    #[default]
    OnDemand,
    /// Daily
    Daily,
    /// Weekly
    Weekly,
    /// Monthly
    Monthly,
}

impl std::fmt::Display for ReportFrequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OnDemand => write!(f, "on-demand"),
            Self::Daily => write!(f, "daily"),
            Self::Weekly => write!(f, "weekly"),
            Self::Monthly => write!(f, "monthly"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_type_display() {
        assert_eq!(format!("{}", ReportType::Status), "status");
        assert_eq!(format!("{}", ReportType::Audit), "audit");
    }

    #[test]
    fn test_frequency_display() {
        assert_eq!(format!("{}", ReportFrequency::OnDemand), "on-demand");
        assert_eq!(format!("{}", ReportFrequency::Weekly), "weekly");
    }
}
