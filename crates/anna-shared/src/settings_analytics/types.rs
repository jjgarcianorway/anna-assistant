// v0.0.577: Settings Analytics - Types (Phase 153)

use serde::{Deserialize, Serialize};

/// Analytics time period
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalyticsPeriod {
    /// Last 24 hours
    Day,
    /// Last 7 days
    Week,
    /// Last 30 days
    Month,
    /// All time
    AllTime,
}

impl std::fmt::Display for AnalyticsPeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Day => write!(f, "Last 24 Hours"),
            Self::Week => write!(f, "Last 7 Days"),
            Self::Month => write!(f, "Last 30 Days"),
            Self::AllTime => write!(f, "All Time"),
        }
    }
}

/// Analytics metric type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    /// Change count
    ChangeCount,
    /// Access count
    AccessCount,
    /// Revert count
    RevertCount,
    /// Export count
    ExportCount,
    /// Import count
    ImportCount,
}

impl std::fmt::Display for MetricType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChangeCount => write!(f, "Changes"),
            Self::AccessCount => write!(f, "Accesses"),
            Self::RevertCount => write!(f, "Reverts"),
            Self::ExportCount => write!(f, "Exports"),
            Self::ImportCount => write!(f, "Imports"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_period_display() {
        assert_eq!(format!("{}", AnalyticsPeriod::Day), "Last 24 Hours");
        assert_eq!(format!("{}", AnalyticsPeriod::Week), "Last 7 Days");
    }

    #[test]
    fn test_metric_type_display() {
        assert_eq!(format!("{}", MetricType::ChangeCount), "Changes");
        assert_eq!(format!("{}", MetricType::ExportCount), "Exports");
    }
}
