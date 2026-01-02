//! Background job types (v0.0.430).

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

// Re-export types from submodules
pub use super::job_result::JobResult;
pub use super::job_types::{JobKind, JobPriority, JobStatus};
pub use super::notification::{NotificationPriority, PendingNotification};

/// A background job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundJob {
    /// Unique job ID
    pub id: String,
    /// Type of job
    pub kind: JobKind,
    /// Priority level
    pub priority: JobPriority,
    /// When the job was created (unix timestamp)
    pub created_at: u64,
    /// When the job should run (unix timestamp)
    pub scheduled_for: u64,
    /// Last execution time (unix timestamp)
    pub last_run_at: Option<u64>,
    /// Current status
    pub status: JobStatus,
    /// Number of retry attempts
    pub retry_count: u32,
    /// Maximum retries allowed
    pub max_retries: u32,
    /// Optional metadata
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

impl BackgroundJob {
    /// Create a new job
    pub fn new(kind: JobKind, priority: JobPriority) -> Self {
        let now = now_timestamp();
        Self {
            id: generate_job_id(),
            kind,
            priority,
            created_at: now,
            scheduled_for: now,
            last_run_at: None,
            status: JobStatus::Pending,
            retry_count: 0,
            max_retries: 3,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Schedule job for a specific time
    pub fn scheduled_at(mut self, timestamp: u64) -> Self {
        self.scheduled_for = timestamp;
        self
    }

    /// Schedule job with delay from now
    pub fn delayed(mut self, delay_secs: u64) -> Self {
        self.scheduled_for = now_timestamp() + delay_secs;
        self
    }

    /// Set maximum retries
    pub fn with_max_retries(mut self, max: u32) -> Self {
        self.max_retries = max;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Check if job is due to run
    pub fn is_due(&self, now: u64) -> bool {
        self.status.is_runnable() && self.scheduled_for <= now
    }

    /// Check if job can be retried
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    /// Mark job as running
    pub fn mark_running(&mut self) {
        self.status = JobStatus::Running;
        self.last_run_at = Some(now_timestamp());
    }

    /// Mark job as completed
    pub fn mark_completed(&mut self, summary: Option<String>) {
        self.status = JobStatus::Completed {
            completed_at: now_timestamp(),
            result_summary: summary,
        };
    }

    /// Mark job as failed
    pub fn mark_failed(&mut self, reason: &str) {
        self.retry_count += 1;
        if self.can_retry() {
            // Reset to pending for retry with backoff
            self.status = JobStatus::Pending;
            self.scheduled_for = now_timestamp() + (60 * self.retry_count as u64);
        } else {
            self.status = JobStatus::Failed {
                reason: reason.to_string(),
                failed_at: now_timestamp(),
            };
        }
    }

    /// Mark job as cancelled
    pub fn mark_cancelled(&mut self) {
        self.status = JobStatus::Cancelled {
            cancelled_at: now_timestamp(),
        };
    }

    /// Create a long ticket analysis job
    pub fn long_ticket(ticket_id: &str) -> Self {
        Self::new(
            JobKind::LongTicketAnalysis {
                ticket_id: ticket_id.to_string(),
            },
            JobPriority::Normal,
        )
    }

    /// Create a doc index refresh job (low priority)
    pub fn doc_refresh() -> Self {
        Self::new(JobKind::DocIndexRefresh, JobPriority::Low)
    }

    /// Create a model benchmark job (low priority)
    pub fn model_benchmark() -> Self {
        Self::new(JobKind::ModelBenchmark, JobPriority::Low)
    }

    /// Create a periodic probe job
    pub fn periodic_probe(probe_name: &str, priority: JobPriority) -> Self {
        Self::new(
            JobKind::PeriodicProbe {
                probe_name: probe_name.to_string(),
            },
            priority,
        )
    }

    /// Create a user reminder job
    pub fn reminder(reminder_id: &str) -> Self {
        Self::new(
            JobKind::UserReminder {
                reminder_id: reminder_id.to_string(),
            },
            JobPriority::Normal,
        )
    }

    /// Create a monitor check job
    pub fn monitor_check(monitor_id: &str, priority: JobPriority) -> Self {
        Self::new(
            JobKind::MonitorCheck {
                monitor_id: monitor_id.to_string(),
            },
            priority,
        )
    }

    /// Create a recipe consolidation job (low priority)
    pub fn recipe_consolidation() -> Self {
        Self::new(JobKind::RecipeConsolidation, JobPriority::Low)
    }
}

/// Generate a unique job ID
fn generate_job_id() -> String {
    format!(
        "JOB-{}",
        uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
    )
}

/// Get current unix timestamp
pub fn now_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_creation() {
        let job = BackgroundJob::long_ticket("TKT-123");
        assert!(job.id.starts_with("JOB-"));
        assert_eq!(job.priority, JobPriority::Normal);
        assert!(matches!(job.kind, JobKind::LongTicketAnalysis { .. }));
    }

    #[test]
    fn test_job_scheduling() {
        let job = BackgroundJob::doc_refresh().delayed(3600);
        assert!(job.scheduled_for > job.created_at);
    }

    #[test]
    fn test_job_status_transitions() {
        let mut job = BackgroundJob::doc_refresh();
        assert!(job.status.is_runnable());

        job.mark_running();
        assert!(!job.status.is_runnable());
        assert!(matches!(job.status, JobStatus::Running));

        job.mark_completed(Some("Done".to_string()));
        assert!(job.status.is_terminal());
    }

    #[test]
    fn test_job_retry() {
        let mut job = BackgroundJob::doc_refresh().with_max_retries(2);

        // First failure - can still retry (1 < 2)
        job.mark_failed("First failure");
        assert_eq!(job.retry_count, 1);
        assert!(job.status.is_runnable()); // Should be pending for retry

        // Second failure - no more retries (2 >= 2)
        job.mark_failed("Second failure");
        assert_eq!(job.retry_count, 2);
        assert!(job.status.is_terminal()); // Failed permanently
    }

    #[test]
    fn test_job_priority_ordering() {
        assert!(JobPriority::Low < JobPriority::Normal);
        assert!(JobPriority::Normal < JobPriority::High);
    }
}
