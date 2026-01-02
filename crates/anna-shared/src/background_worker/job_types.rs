//! Background job type definitions (v0.0.430).

use serde::{Deserialize, Serialize};

/// Kind of background job
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JobKind {
    /// Long-running ticket analysis
    LongTicketAnalysis { ticket_id: String },
    /// Refresh documentation index
    DocIndexRefresh,
    /// Benchmark LLM models for routing decisions
    ModelBenchmark,
    /// Periodic probe check (e.g., weekly trim, disk health)
    PeriodicProbe { probe_name: String },
    /// User reminder (e.g., "Monday 9am storage report")
    UserReminder { reminder_id: String },
    /// Monitor check (e.g., disk threshold)
    MonitorCheck { monitor_id: String },
    /// Recipe consolidation from past tickets
    RecipeConsolidation,
    /// Send pending notification
    SendNotification { notification_id: String },
}

impl JobKind {
    /// Get human-readable description
    pub fn description(&self) -> String {
        match self {
            Self::LongTicketAnalysis { ticket_id } => {
                format!("Long analysis for ticket {}", ticket_id)
            }
            Self::DocIndexRefresh => "Refresh documentation index".to_string(),
            Self::ModelBenchmark => "Benchmark LLM models".to_string(),
            Self::PeriodicProbe { probe_name } => format!("Periodic probe: {}", probe_name),
            Self::UserReminder { reminder_id } => format!("Reminder: {}", reminder_id),
            Self::MonitorCheck { monitor_id } => format!("Monitor check: {}", monitor_id),
            Self::RecipeConsolidation => "Consolidate learned recipes".to_string(),
            Self::SendNotification { notification_id } => {
                format!("Send notification {}", notification_id)
            }
        }
    }

    /// Check if this job requires idle time
    pub fn requires_idle(&self) -> bool {
        matches!(
            self,
            Self::DocIndexRefresh | Self::ModelBenchmark | Self::RecipeConsolidation
        )
    }
}

/// Job priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobPriority {
    /// Only run during idle time
    Low = 0,
    /// Normal scheduled tasks
    Normal = 1,
    /// Urgent follow-up for critical alerts
    High = 2,
}

impl Default for JobPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Job execution status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum JobStatus {
    /// Waiting to be executed
    Pending,
    /// Currently running
    Running,
    /// Successfully completed
    Completed {
        completed_at: u64,
        result_summary: Option<String>,
    },
    /// Failed with error
    Failed { reason: String, failed_at: u64 },
    /// Cancelled by user or system
    Cancelled { cancelled_at: u64 },
}

impl JobStatus {
    /// Check if job is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed { .. } | Self::Failed { .. } | Self::Cancelled { .. }
        )
    }

    /// Check if job is runnable
    pub fn is_runnable(&self) -> bool {
        matches!(self, Self::Pending)
    }

    /// Get short display string
    pub fn display(&self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Running => "RUNNING",
            Self::Completed { .. } => "COMPLETED",
            Self::Failed { .. } => "FAILED",
            Self::Cancelled { .. } => "CANCELLED",
        }
    }
}

impl Default for JobStatus {
    fn default() -> Self {
        Self::Pending
    }
}
