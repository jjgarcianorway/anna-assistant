//! Scheduler types and configurations.

use super::super::job::{BackgroundJob, JobKind, JobPriority};
use super::super::IDLE_CPU_THRESHOLD;
use serde::{Deserialize, Serialize};

/// Scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// CPU load threshold for running low-priority jobs (0.0-1.0)
    pub idle_cpu_threshold: f32,
    /// Maximum concurrent jobs
    pub max_concurrent_jobs: usize,
    /// Maximum jobs to run per day for idle learning
    pub max_idle_jobs_per_day: usize,
    /// Whether scheduler is enabled
    pub enabled: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            idle_cpu_threshold: IDLE_CPU_THRESHOLD,
            max_concurrent_jobs: 2,
            max_idle_jobs_per_day: 10,
            enabled: true,
        }
    }
}

/// Scheduler runtime statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchedulerStats {
    /// Jobs completed in last 24 hours
    pub completed_24h: usize,
    /// Jobs failed in last 24 hours
    pub failed_24h: usize,
    /// Idle jobs run today
    pub idle_jobs_today: usize,
    /// Total jobs ever completed
    pub total_completed: usize,
    /// Total jobs ever failed
    pub total_failed: usize,
    /// Average job duration (ms)
    pub avg_duration_ms: u64,
    /// Last job completion time
    pub last_completion: Option<u64>,
}

/// Filter for querying jobs
#[derive(Debug, Clone, Default)]
pub struct JobFilter {
    /// Filter by status
    pub status: Option<Vec<&'static str>>,
    /// Filter by kind
    pub kind: Option<JobKind>,
    /// Filter by priority
    pub priority: Option<JobPriority>,
    /// Created after timestamp
    pub created_after: Option<u64>,
}

impl JobFilter {
    /// Create empty filter (matches all)
    pub fn all() -> Self {
        Self::default()
    }

    /// Filter by status
    pub fn with_status(mut self, statuses: &[&'static str]) -> Self {
        self.status = Some(statuses.to_vec());
        self
    }

    /// Filter pending jobs
    pub fn pending() -> Self {
        Self::default().with_status(&["PENDING"])
    }

    /// Filter running jobs
    pub fn running() -> Self {
        Self::default().with_status(&["RUNNING"])
    }

    /// Filter completed jobs
    pub fn completed() -> Self {
        Self::default().with_status(&["COMPLETED"])
    }

    /// Filter failed jobs
    pub fn failed() -> Self {
        Self::default().with_status(&["FAILED"])
    }

    /// Check if job matches filter
    pub fn matches(&self, job: &BackgroundJob) -> bool {
        if let Some(ref statuses) = self.status {
            if !statuses.contains(&job.status.display()) {
                return false;
            }
        }

        if let Some(ref kind) = self.kind {
            if job.kind != *kind {
                return false;
            }
        }

        if let Some(priority) = self.priority {
            if job.priority != priority {
                return false;
            }
        }

        if let Some(after) = self.created_after {
            if job.created_at < after {
                return false;
            }
        }

        true
    }
}

/// Summary for status display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStatusSummary {
    pub pending: usize,
    pub running: usize,
    pub completed_24h: usize,
    pub failed_24h: usize,
    pub idle_jobs_today: usize,
    pub max_idle_jobs: usize,
    pub enabled: bool,
}

impl std::fmt::Display for SchedulerStatusSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[background_jobs]")?;
        writeln!(f, "  pending              {}", self.pending)?;
        writeln!(f, "  running              {}", self.running)?;
        writeln!(f, "  completed_last_24h   {}", self.completed_24h)?;
        writeln!(f, "  failed_last_24h      {}", self.failed_24h)?;
        writeln!(
            f,
            "  idle_jobs_today      {}/{}",
            self.idle_jobs_today, self.max_idle_jobs
        )?;
        writeln!(
            f,
            "  scheduler            {}",
            if self.enabled { "ENABLED" } else { "DISABLED" }
        )
    }
}
