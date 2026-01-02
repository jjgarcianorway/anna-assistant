//! Types for senior strategic thinking.

use crate::teams::Team;
use serde::{Deserialize, Serialize};

/// A strategic insight from a senior specialist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategicInsight {
    /// Unique insight ID
    pub id: String,
    /// Team the insight is from
    pub team: Team,
    /// Senior specialist who generated it
    pub specialist: String,
    /// Category of insight
    pub category: InsightCategory,
    /// Title/summary
    pub title: String,
    /// Detailed analysis
    pub analysis: String,
    /// Recommended actions
    pub recommendations: Vec<String>,
    /// Priority level
    pub priority: InsightPriority,
    /// When generated (unix timestamp)
    pub generated_at: u64,
    /// Based on how many tickets
    pub ticket_count: usize,
    /// Time period analyzed (days)
    pub period_days: u32,
}

/// Categories of strategic insights
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InsightCategory {
    /// Recurring pattern detected
    RecurringPattern,
    /// Performance degradation trend
    PerformanceTrend,
    /// Security concern
    SecurityConcern,
    /// Maintenance suggestion
    MaintenanceSuggestion,
    /// Configuration opportunity
    ConfigurationOpportunity,
    /// Capacity planning
    CapacityPlanning,
}

impl InsightCategory {
    pub fn display(&self) -> &'static str {
        match self {
            Self::RecurringPattern => "Recurring Pattern",
            Self::PerformanceTrend => "Performance Trend",
            Self::SecurityConcern => "Security Concern",
            Self::MaintenanceSuggestion => "Maintenance Suggestion",
            Self::ConfigurationOpportunity => "Configuration Opportunity",
            Self::CapacityPlanning => "Capacity Planning",
        }
    }
}

/// Priority of insights
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InsightPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl InsightPriority {
    pub fn display(&self) -> &'static str {
        match self {
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
            Self::Critical => "CRITICAL",
        }
    }
}

/// Strategic analysis session state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StrategicSession {
    /// Session ID
    pub session_id: String,
    /// Started at (unix timestamp)
    pub started_at: u64,
    /// Completed at (if finished)
    pub completed_at: Option<u64>,
    /// Insights generated
    pub insights: Vec<StrategicInsight>,
    /// Tickets analyzed
    pub tickets_analyzed: usize,
    /// Time range analyzed (days)
    pub days_analyzed: u32,
    /// Can be resumed if interrupted
    pub resumable: bool,
    /// Progress checkpoint (for resuming)
    pub checkpoint: Option<AnalysisCheckpoint>,
}

/// Checkpoint for resuming interrupted analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisCheckpoint {
    /// Last ticket ID processed
    pub last_ticket_id: String,
    /// Current team being analyzed
    pub current_team: Team,
    /// Partial insights collected
    pub partial_insights: Vec<StrategicInsight>,
}

impl StrategicSession {
    /// Create new session
    pub fn new() -> Self {
        Self {
            session_id: super::utils::generate_session_id(),
            started_at: super::utils::now_timestamp(),
            completed_at: None,
            insights: vec![],
            tickets_analyzed: 0,
            days_analyzed: 7, // Default to 7 days
            resumable: true,
            checkpoint: None,
        }
    }

    /// Mark as completed
    pub fn complete(&mut self) {
        self.completed_at = Some(super::utils::now_timestamp());
        self.checkpoint = None;
    }

    /// Create checkpoint for resuming
    pub fn create_checkpoint(&mut self, last_ticket: &str, team: Team) {
        self.checkpoint = Some(AnalysisCheckpoint {
            last_ticket_id: last_ticket.to_string(),
            current_team: team,
            partial_insights: self.insights.clone(),
        });
    }

    /// Resume from checkpoint
    pub fn resume(&mut self) -> Option<&AnalysisCheckpoint> {
        self.checkpoint.as_ref()
    }

    /// Add insight
    pub fn add_insight(&mut self, insight: StrategicInsight) {
        self.insights.push(insight);
    }

    /// Is session complete?
    pub fn is_complete(&self) -> bool {
        self.completed_at.is_some()
    }

    /// Get duration in seconds
    pub fn duration_secs(&self) -> u64 {
        let end = self.completed_at.unwrap_or_else(super::utils::now_timestamp);
        end.saturating_sub(self.started_at)
    }
}

/// Summary of a ticket for analysis
#[derive(Debug, Clone)]
pub struct TicketSummary {
    pub ticket_id: String,
    pub team: Team,
    pub query_type: String,
    pub resolution_success: bool,
    pub created_at: u64,
    pub resolved_at: Option<u64>,
}

/// A detected pattern
#[derive(Debug, Clone)]
pub struct DetectedPattern {
    pub team: Team,
    pub pattern_type: String,
    pub occurrence_count: usize,
    pub first_seen: u64,
    pub last_seen: u64,
}
