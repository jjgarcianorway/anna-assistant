//! Job execution result types (v0.0.430).

use super::job::BackgroundJob;
use super::notification::PendingNotification;

/// Job result from execution
#[derive(Debug, Clone)]
pub struct JobResult {
    /// Whether execution succeeded
    pub success: bool,
    /// Summary of what was done
    pub summary: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Any follow-up jobs to schedule
    pub follow_up_jobs: Vec<BackgroundJob>,
    /// Any notifications to send
    pub notifications: Vec<PendingNotification>,
}

impl JobResult {
    /// Create a successful result
    pub fn success(summary: &str) -> Self {
        Self {
            success: true,
            summary: Some(summary.to_string()),
            error: None,
            duration_ms: 0,
            follow_up_jobs: vec![],
            notifications: vec![],
        }
    }

    /// Create a failed result
    pub fn failure(error: &str) -> Self {
        Self {
            success: false,
            summary: None,
            error: Some(error.to_string()),
            duration_ms: 0,
            follow_up_jobs: vec![],
            notifications: vec![],
        }
    }

    /// Set duration
    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }

    /// Add follow-up job
    pub fn with_follow_up(mut self, job: BackgroundJob) -> Self {
        self.follow_up_jobs.push(job);
        self
    }

    /// Add notification
    pub fn with_notification(mut self, notification: PendingNotification) -> Self {
        self.notifications.push(notification);
        self
    }
}
