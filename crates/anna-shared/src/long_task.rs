//! Long-running task detection and handling (v0.0.455).
//!
//! Detects when tasks take too long and offers to:
//! - Continue in background
//! - Send email when complete
//! - Resume interrupted analysis
//!
//! v0.0.455: Initial implementation per VISION.md Phase 34.

use crate::background_worker::job::{BackgroundJob, JobKind, JobPriority, PendingNotification, NotificationPriority};
use crate::email::EmailConfig;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Default threshold for considering a task "long-running" (in seconds)
pub const DEFAULT_LONG_TASK_THRESHOLD_SECS: u64 = 120; // 2 minutes

/// Configuration for long-running task handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongTaskConfig {
    /// Threshold in seconds before task is considered long-running
    pub threshold_secs: u64,
    /// Whether to prompt for email when threshold exceeded
    pub prompt_for_email: bool,
    /// Whether to offer background continuation
    pub allow_background: bool,
    /// Maximum background tasks
    pub max_background_tasks: u32,
}

impl Default for LongTaskConfig {
    fn default() -> Self {
        Self {
            threshold_secs: DEFAULT_LONG_TASK_THRESHOLD_SECS,
            prompt_for_email: true,
            allow_background: true,
            max_background_tasks: 3,
        }
    }
}

/// Status of a long-running task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LongTaskStatus {
    /// Task running normally (under threshold)
    Running {
        started_at: u64,
        elapsed_secs: u64,
    },
    /// Task exceeded threshold, awaiting user decision
    ThresholdExceeded {
        started_at: u64,
        elapsed_secs: u64,
        offered_options: bool,
    },
    /// User chose to continue in background
    MovedToBackground {
        job_id: String,
        email: Option<String>,
    },
    /// Task completed
    Completed {
        duration_secs: u64,
    },
    /// Task cancelled by user
    Cancelled,
}

/// Options presented to user when task takes too long
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongTaskOptions {
    /// Continue waiting in foreground
    pub wait: bool,
    /// Move to background and notify when done
    pub background: bool,
    /// Cancel the task
    pub cancel: bool,
}

impl Default for LongTaskOptions {
    fn default() -> Self {
        Self {
            wait: true,
            background: true,
            cancel: true,
        }
    }
}

/// Tracker for a potentially long-running task
#[derive(Debug)]
pub struct LongTaskTracker {
    /// Task identifier (ticket ID, query hash, etc.)
    pub task_id: String,
    /// When the task started
    started: Instant,
    /// Configuration
    config: LongTaskConfig,
    /// Current status
    status: LongTaskStatus,
    /// User's email (if known)
    user_email: Option<String>,
    /// Whether threshold prompt has been shown
    threshold_prompted: bool,
}

impl LongTaskTracker {
    /// Create a new tracker
    pub fn new(task_id: &str) -> Self {
        let now_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            task_id: task_id.to_string(),
            started: Instant::now(),
            config: LongTaskConfig::default(),
            status: LongTaskStatus::Running {
                started_at: now_ts,
                elapsed_secs: 0,
            },
            user_email: EmailConfig::load().user_email,
            threshold_prompted: false,
        }
    }

    /// Create with custom config
    pub fn with_config(task_id: &str, config: LongTaskConfig) -> Self {
        let mut tracker = Self::new(task_id);
        tracker.config = config;
        tracker
    }

    /// Check elapsed time
    pub fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }

    /// Check if threshold exceeded
    pub fn is_threshold_exceeded(&self) -> bool {
        self.elapsed().as_secs() >= self.config.threshold_secs
    }

    /// Check if we should prompt user
    pub fn should_prompt(&self) -> bool {
        self.is_threshold_exceeded() && !self.threshold_prompted
    }

    /// Mark that prompt was shown
    pub fn mark_prompted(&mut self) {
        self.threshold_prompted = true;
        let elapsed = self.elapsed().as_secs();
        let now_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.status = LongTaskStatus::ThresholdExceeded {
            started_at: now_ts - elapsed,
            elapsed_secs: elapsed,
            offered_options: true,
        };
    }

    /// Set user email
    pub fn set_email(&mut self, email: &str) {
        self.user_email = Some(email.to_string());
        // Save to config for reuse
        let _ = store_email(email);
    }

    /// Get user email
    pub fn email(&self) -> Option<&str> {
        self.user_email.as_deref()
    }

    /// Move task to background
    pub fn move_to_background(&mut self) -> BackgroundJob {
        let job = BackgroundJob::new(
            JobKind::LongTicketAnalysis {
                ticket_id: self.task_id.clone(),
            },
            JobPriority::Normal,
        )
        .with_metadata("email", self.user_email.as_deref().unwrap_or(""))
        .with_metadata("elapsed_secs", &self.elapsed().as_secs().to_string());

        self.status = LongTaskStatus::MovedToBackground {
            job_id: job.id.clone(),
            email: self.user_email.clone(),
        };

        job
    }

    /// Mark task completed
    pub fn mark_completed(&mut self) {
        self.status = LongTaskStatus::Completed {
            duration_secs: self.elapsed().as_secs(),
        };
    }

    /// Mark task cancelled
    pub fn mark_cancelled(&mut self) {
        self.status = LongTaskStatus::Cancelled;
    }

    /// Get current status
    pub fn status(&self) -> &LongTaskStatus {
        &self.status
    }

    /// Build notification for completed background task
    pub fn completion_notification(&self, result_summary: &str) -> PendingNotification {
        let duration = match &self.status {
            LongTaskStatus::Completed { duration_secs } => *duration_secs,
            _ => self.elapsed().as_secs(),
        };

        let body = format!(
            r#"Your long-running analysis has completed.

Task: {}
Duration: {} seconds

Result:
{}

--
Anna Service Desk
"#,
            self.task_id, duration, result_summary
        );

        PendingNotification::new(
            &format!("[Anna] Task {} completed", self.task_id),
            &body,
            NotificationPriority::Normal,
        )
    }
}

/// Check if a stored email is available
pub fn has_stored_email() -> bool {
    EmailConfig::load().user_email.is_some()
}

/// Get stored email
pub fn get_stored_email() -> Option<String> {
    EmailConfig::load().user_email
}

/// Store email for reuse
pub fn store_email(email: &str) -> Result<(), String> {
    let mut config = EmailConfig::load();
    config.user_email = Some(email.to_string());
    config.save().map_err(|e| e.to_string())
}

/// Format the long-task prompt message
pub fn format_long_task_prompt(elapsed_secs: u64, has_email: bool) -> String {
    let mins = elapsed_secs / 60;
    let secs = elapsed_secs % 60;

    let mut prompt = format!(
        "This task has been running for {}m {}s.\n",
        mins, secs
    );

    prompt.push_str("\nOptions:\n");
    prompt.push_str("  [1] Keep waiting (continue in foreground)\n");
    prompt.push_str("  [2] Move to background");

    if has_email {
        prompt.push_str(" (I'll email you when done)\n");
    } else {
        prompt.push_str("\n      (Provide email for notification)\n");
    }

    prompt.push_str("  [3] Cancel task\n");

    prompt
}

/// Parse user's choice for long-task prompt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LongTaskChoice {
    KeepWaiting,
    MoveToBackground,
    Cancel,
}

impl LongTaskChoice {
    pub fn from_input(input: &str) -> Option<Self> {
        let trimmed = input.trim().to_lowercase();
        match trimmed.as_str() {
            "1" | "wait" | "continue" | "keep" => Some(Self::KeepWaiting),
            "2" | "background" | "bg" | "back" => Some(Self::MoveToBackground),
            "3" | "cancel" | "stop" | "abort" => Some(Self::Cancel),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_creation() {
        let tracker = LongTaskTracker::new("TEST-001");
        assert_eq!(tracker.task_id, "TEST-001");
        assert!(!tracker.is_threshold_exceeded());
    }

    #[test]
    fn test_config_defaults() {
        let config = LongTaskConfig::default();
        assert_eq!(config.threshold_secs, DEFAULT_LONG_TASK_THRESHOLD_SECS);
        assert!(config.prompt_for_email);
        assert!(config.allow_background);
    }

    #[test]
    fn test_choice_parsing() {
        assert_eq!(LongTaskChoice::from_input("1"), Some(LongTaskChoice::KeepWaiting));
        assert_eq!(LongTaskChoice::from_input("2"), Some(LongTaskChoice::MoveToBackground));
        assert_eq!(LongTaskChoice::from_input("3"), Some(LongTaskChoice::Cancel));
        assert_eq!(LongTaskChoice::from_input("wait"), Some(LongTaskChoice::KeepWaiting));
        assert_eq!(LongTaskChoice::from_input("bg"), Some(LongTaskChoice::MoveToBackground));
        assert_eq!(LongTaskChoice::from_input("cancel"), Some(LongTaskChoice::Cancel));
        assert_eq!(LongTaskChoice::from_input("invalid"), None);
    }

    #[test]
    fn test_prompt_formatting() {
        let prompt = format_long_task_prompt(125, true);
        assert!(prompt.contains("2m 5s"));
        assert!(prompt.contains("email you when done"));
    }

    #[test]
    fn test_prompt_without_email() {
        let prompt = format_long_task_prompt(180, false);
        assert!(prompt.contains("3m 0s"));
        assert!(prompt.contains("Provide email"));
    }
}
