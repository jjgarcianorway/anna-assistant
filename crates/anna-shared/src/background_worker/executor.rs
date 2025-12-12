//! Job executor (v0.0.430).
//!
//! Executes background jobs based on their kind.

use super::job::{BackgroundJob, JobKind, JobResult, NotificationPriority, PendingNotification};
use std::time::Instant;

/// Job executor trait
pub trait JobExecutor: Send + Sync {
    /// Execute a job and return the result
    fn execute(&self, job: &BackgroundJob) -> JobResult;
}

/// Default executor that dispatches to specialized handlers
pub struct DefaultExecutor {
    /// Handler for long ticket analysis
    pub long_ticket_handler: Option<Box<dyn LongTicketHandler>>,
    /// Handler for doc index refresh
    pub doc_refresh_handler: Option<Box<dyn DocRefreshHandler>>,
    /// Handler for model benchmarking
    pub benchmark_handler: Option<Box<dyn BenchmarkHandler>>,
    /// Handler for periodic probes
    pub probe_handler: Option<Box<dyn ProbeHandler>>,
    /// Handler for user reminders
    pub reminder_handler: Option<Box<dyn ReminderHandler>>,
    /// Handler for monitor checks
    pub monitor_handler: Option<Box<dyn MonitorHandler>>,
    /// Handler for recipe consolidation
    pub recipe_handler: Option<Box<dyn RecipeHandler>>,
}

impl Default for DefaultExecutor {
    fn default() -> Self {
        Self {
            long_ticket_handler: None,
            doc_refresh_handler: None,
            benchmark_handler: None,
            probe_handler: None,
            reminder_handler: None,
            monitor_handler: None,
            recipe_handler: None,
        }
    }
}

impl JobExecutor for DefaultExecutor {
    fn execute(&self, job: &BackgroundJob) -> JobResult {
        let start = Instant::now();

        let result = match &job.kind {
            JobKind::LongTicketAnalysis { ticket_id } => self.execute_long_ticket(ticket_id),
            JobKind::DocIndexRefresh => self.execute_doc_refresh(),
            JobKind::ModelBenchmark => self.execute_benchmark(),
            JobKind::PeriodicProbe { probe_name } => self.execute_probe(probe_name),
            JobKind::UserReminder { reminder_id } => self.execute_reminder(reminder_id),
            JobKind::MonitorCheck { monitor_id } => self.execute_monitor(monitor_id),
            JobKind::RecipeConsolidation => self.execute_recipe_consolidation(),
            JobKind::SendNotification { notification_id } => {
                self.execute_notification(notification_id)
            }
        };

        result.with_duration(start.elapsed().as_millis() as u64)
    }
}

impl DefaultExecutor {
    /// Create new executor
    pub fn new() -> Self {
        Self::default()
    }

    fn execute_long_ticket(&self, ticket_id: &str) -> JobResult {
        if let Some(ref handler) = self.long_ticket_handler {
            handler.analyze(ticket_id)
        } else {
            JobResult::failure("Long ticket handler not configured")
        }
    }

    fn execute_doc_refresh(&self) -> JobResult {
        if let Some(ref handler) = self.doc_refresh_handler {
            handler.refresh()
        } else {
            JobResult::success("Doc refresh handler not configured - skipped")
        }
    }

    fn execute_benchmark(&self) -> JobResult {
        if let Some(ref handler) = self.benchmark_handler {
            handler.benchmark()
        } else {
            JobResult::success("Benchmark handler not configured - skipped")
        }
    }

    fn execute_probe(&self, probe_name: &str) -> JobResult {
        if let Some(ref handler) = self.probe_handler {
            handler.run_probe(probe_name)
        } else {
            JobResult::failure(&format!("Probe handler not configured for {}", probe_name))
        }
    }

    fn execute_reminder(&self, reminder_id: &str) -> JobResult {
        if let Some(ref handler) = self.reminder_handler {
            handler.trigger_reminder(reminder_id)
        } else {
            // Create a notification for the reminder
            let notification = PendingNotification::new(
                &format!("Reminder: {}", reminder_id),
                "Your scheduled reminder has triggered.",
                NotificationPriority::Normal,
            );
            JobResult::success("Reminder triggered").with_notification(notification)
        }
    }

    fn execute_monitor(&self, monitor_id: &str) -> JobResult {
        if let Some(ref handler) = self.monitor_handler {
            handler.check_monitor(monitor_id)
        } else {
            JobResult::failure(&format!(
                "Monitor handler not configured for {}",
                monitor_id
            ))
        }
    }

    fn execute_recipe_consolidation(&self) -> JobResult {
        if let Some(ref handler) = self.recipe_handler {
            handler.consolidate()
        } else {
            JobResult::success("Recipe consolidation not configured - skipped")
        }
    }

    fn execute_notification(&self, notification_id: &str) -> JobResult {
        // Notifications are handled by the notification system directly
        JobResult::success(&format!("Notification {} processed", notification_id))
    }
}

/// Handler for long ticket analysis
pub trait LongTicketHandler: Send + Sync {
    fn analyze(&self, ticket_id: &str) -> JobResult;
}

/// Handler for doc index refresh
pub trait DocRefreshHandler: Send + Sync {
    fn refresh(&self) -> JobResult;
}

/// Handler for model benchmarking
pub trait BenchmarkHandler: Send + Sync {
    fn benchmark(&self) -> JobResult;
}

/// Handler for periodic probes
pub trait ProbeHandler: Send + Sync {
    fn run_probe(&self, probe_name: &str) -> JobResult;
}

/// Handler for user reminders
pub trait ReminderHandler: Send + Sync {
    fn trigger_reminder(&self, reminder_id: &str) -> JobResult;
}

/// Handler for monitor checks
pub trait MonitorHandler: Send + Sync {
    fn check_monitor(&self, monitor_id: &str) -> JobResult;
}

/// Handler for recipe consolidation
pub trait RecipeHandler: Send + Sync {
    fn consolidate(&self) -> JobResult;
}

/// CPU load detection for idle-time scheduling
pub struct CpuMonitor;

impl CpuMonitor {
    /// Get current CPU load (0.0-1.0)
    pub fn get_load() -> f32 {
        // Read from /proc/loadavg on Linux
        if let Ok(content) = std::fs::read_to_string("/proc/loadavg") {
            if let Some(load_str) = content.split_whitespace().next() {
                if let Ok(load) = load_str.parse::<f32>() {
                    // Normalize by number of CPUs
                    let num_cpus = num_cpus();
                    return (load / num_cpus as f32).min(1.0);
                }
            }
        }
        // Default to busy if can't read
        0.5
    }

    /// Check if system is idle enough for low-priority work
    pub fn is_idle(threshold: f32) -> bool {
        Self::get_load() < threshold
    }
}

/// Get number of CPUs
fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_executor() {
        let executor = DefaultExecutor::new();
        let job = BackgroundJob::doc_refresh();
        let result = executor.execute(&job);
        // Should succeed with "not configured" message
        assert!(result.success);
    }

    #[test]
    fn test_cpu_monitor() {
        let load = CpuMonitor::get_load();
        assert!(load >= 0.0 && load <= 1.0);
    }

    #[test]
    fn test_num_cpus() {
        assert!(num_cpus() >= 1);
    }
}
