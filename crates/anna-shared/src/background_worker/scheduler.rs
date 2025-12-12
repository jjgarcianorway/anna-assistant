//! Job scheduler (v0.0.430).
//!
//! Manages background job queue with priority and idle-time awareness.

use super::job::{BackgroundJob, JobKind, JobPriority, JobStatus};
use super::storage::JobStorage;
use super::IDLE_CPU_THRESHOLD;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Job scheduler for background work
#[derive(Debug)]
pub struct JobScheduler {
    /// In-memory job queue
    jobs: HashMap<String, BackgroundJob>,
    /// Persistent storage
    storage: JobStorage,
    /// Scheduler configuration
    config: SchedulerConfig,
    /// Runtime statistics
    stats: SchedulerStats,
}

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

impl JobScheduler {
    /// Create a new scheduler
    pub fn new(storage_path: &str) -> Self {
        let storage = JobStorage::new(storage_path);
        let jobs = storage.load().unwrap_or_default();

        Self {
            jobs,
            storage,
            config: SchedulerConfig::default(),
            stats: SchedulerStats::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(storage_path: &str, config: SchedulerConfig) -> Self {
        let mut scheduler = Self::new(storage_path);
        scheduler.config = config;
        scheduler
    }

    /// Enqueue a new job
    pub fn enqueue(&mut self, job: BackgroundJob) -> String {
        let id = job.id.clone();
        self.jobs.insert(id.clone(), job);
        let _ = self.save();
        id
    }

    /// Get a job by ID
    pub fn get(&self, job_id: &str) -> Option<&BackgroundJob> {
        self.jobs.get(job_id)
    }

    /// Get mutable job by ID
    pub fn get_mut(&mut self, job_id: &str) -> Option<&mut BackgroundJob> {
        self.jobs.get_mut(job_id)
    }

    /// Cancel a job
    pub fn cancel(&mut self, job_id: &str) -> bool {
        if let Some(job) = self.jobs.get_mut(job_id) {
            if !job.status.is_terminal() {
                job.mark_cancelled();
                let _ = self.save();
                return true;
            }
        }
        false
    }

    /// Get jobs ready to run
    pub fn get_due_jobs(&self, now: u64, cpu_load: f32) -> Vec<&BackgroundJob> {
        let mut due: Vec<&BackgroundJob> = self
            .jobs
            .values()
            .filter(|job| {
                if !job.is_due(now) {
                    return false;
                }

                // Check if low-priority job can run based on CPU load
                if job.kind.requires_idle() || job.priority == JobPriority::Low {
                    return cpu_load < self.config.idle_cpu_threshold;
                }

                true
            })
            .collect();

        // Sort by priority (high first) then by scheduled time
        due.sort_by(|a, b| match b.priority.cmp(&a.priority) {
            std::cmp::Ordering::Equal => a.scheduled_for.cmp(&b.scheduled_for),
            other => other,
        });

        // Limit concurrent jobs
        let running_count = self.count_running();
        let available_slots = self
            .config
            .max_concurrent_jobs
            .saturating_sub(running_count);

        due.into_iter().take(available_slots).collect()
    }

    /// Get all jobs matching a filter
    pub fn get_jobs(&self, filter: JobFilter) -> Vec<&BackgroundJob> {
        self.jobs
            .values()
            .filter(|job| filter.matches(job))
            .collect()
    }

    /// Count jobs by status
    pub fn count_by_status(&self) -> HashMap<&'static str, usize> {
        let mut counts = HashMap::new();
        for job in self.jobs.values() {
            let status = job.status.display();
            *counts.entry(status).or_insert(0) += 1;
        }
        counts
    }

    /// Count running jobs
    pub fn count_running(&self) -> usize {
        self.jobs
            .values()
            .filter(|j| matches!(j.status, JobStatus::Running))
            .count()
    }

    /// Count pending jobs
    pub fn count_pending(&self) -> usize {
        self.jobs
            .values()
            .filter(|j| matches!(j.status, JobStatus::Pending))
            .count()
    }

    /// Mark job as running
    pub fn mark_running(&mut self, job_id: &str) -> bool {
        if let Some(job) = self.jobs.get_mut(job_id) {
            job.mark_running();
            let _ = self.save();
            return true;
        }
        false
    }

    /// Mark job as completed
    pub fn mark_completed(&mut self, job_id: &str, summary: Option<String>) -> bool {
        if let Some(job) = self.jobs.get_mut(job_id) {
            job.mark_completed(summary);
            self.stats.total_completed += 1;
            self.stats.completed_24h += 1;
            self.stats.last_completion = Some(now_timestamp());

            if job.kind.requires_idle() {
                self.stats.idle_jobs_today += 1;
            }

            let _ = self.save();
            return true;
        }
        false
    }

    /// Mark job as failed
    pub fn mark_failed(&mut self, job_id: &str, reason: &str) -> bool {
        if let Some(job) = self.jobs.get_mut(job_id) {
            let was_terminal = job.status.is_terminal();
            job.mark_failed(reason);

            if job.status.is_terminal() && !was_terminal {
                self.stats.total_failed += 1;
                self.stats.failed_24h += 1;
            }

            let _ = self.save();
            return true;
        }
        false
    }

    /// Check if we can run more idle jobs today
    pub fn can_run_idle_job(&self) -> bool {
        self.stats.idle_jobs_today < self.config.max_idle_jobs_per_day
    }

    /// Reset daily counters (call at midnight)
    pub fn reset_daily_counters(&mut self) {
        self.stats.idle_jobs_today = 0;
        self.stats.completed_24h = 0;
        self.stats.failed_24h = 0;
    }

    /// Cleanup old completed/failed jobs (keep last N days)
    pub fn cleanup_old_jobs(&mut self, max_age_days: u64) {
        let cutoff = now_timestamp().saturating_sub(max_age_days * 24 * 60 * 60);

        self.jobs.retain(|_, job| {
            match &job.status {
                JobStatus::Completed { completed_at, .. } => *completed_at > cutoff,
                JobStatus::Failed { failed_at, .. } => *failed_at > cutoff,
                JobStatus::Cancelled { cancelled_at } => *cancelled_at > cutoff,
                _ => true, // Keep pending/running jobs
            }
        });

        let _ = self.save();
    }

    /// Get scheduler statistics
    pub fn stats(&self) -> &SchedulerStats {
        &self.stats
    }

    /// Get scheduler configuration
    pub fn config(&self) -> &SchedulerConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: SchedulerConfig) {
        self.config = config;
    }

    /// Save jobs to disk
    pub fn save(&self) -> Result<(), std::io::Error> {
        self.storage.save(&self.jobs)
    }

    /// Get summary for status display
    pub fn status_summary(&self) -> SchedulerStatusSummary {
        let counts = self.count_by_status();

        SchedulerStatusSummary {
            pending: *counts.get("PENDING").unwrap_or(&0),
            running: *counts.get("RUNNING").unwrap_or(&0),
            completed_24h: self.stats.completed_24h,
            failed_24h: self.stats.failed_24h,
            idle_jobs_today: self.stats.idle_jobs_today,
            max_idle_jobs: self.config.max_idle_jobs_per_day,
            enabled: self.config.enabled,
        }
    }
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

/// Get current unix timestamp
fn now_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn unique_test_path() -> String {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("/tmp/anna_scheduler_test_{}_{}", std::process::id(), id)
    }

    fn cleanup(path: &str) {
        let _ = std::fs::remove_dir_all(path);
    }

    #[test]
    fn test_enqueue_job() {
        let path = unique_test_path();
        let mut scheduler = JobScheduler::new(&path);
        let job = BackgroundJob::doc_refresh();
        let id = scheduler.enqueue(job);

        assert!(scheduler.get(&id).is_some());
        assert_eq!(scheduler.count_pending(), 1);
        cleanup(&path);
    }

    #[test]
    fn test_get_due_jobs() {
        let path = unique_test_path();
        let mut scheduler = JobScheduler::new(&path);

        // Add a due job
        let job = BackgroundJob::long_ticket("TKT-1");
        scheduler.enqueue(job);

        // Add a future job
        let future_job = BackgroundJob::long_ticket("TKT-2").delayed(3600);
        scheduler.enqueue(future_job);

        let now = now_timestamp();
        let due = scheduler.get_due_jobs(now, 0.1);

        assert_eq!(due.len(), 1);
        cleanup(&path);
    }

    #[test]
    fn test_idle_jobs_require_low_cpu() {
        let path = unique_test_path();
        let mut scheduler = JobScheduler::new(&path);

        let job = BackgroundJob::doc_refresh(); // Low priority, requires idle
        scheduler.enqueue(job);

        let now = now_timestamp();

        // High CPU - should not return idle jobs
        let due = scheduler.get_due_jobs(now, 0.8);
        assert!(due.is_empty());

        // Low CPU - should return idle jobs
        let due = scheduler.get_due_jobs(now, 0.1);
        assert_eq!(due.len(), 1);
        cleanup(&path);
    }

    #[test]
    fn test_job_lifecycle() {
        let path = unique_test_path();
        let mut scheduler = JobScheduler::new(&path);

        let job = BackgroundJob::long_ticket("TKT-1");
        let id = scheduler.enqueue(job);

        scheduler.mark_running(&id);
        assert_eq!(scheduler.count_running(), 1);

        scheduler.mark_completed(&id, Some("Done".to_string()));
        assert_eq!(scheduler.count_running(), 0);
        assert_eq!(scheduler.stats.total_completed, 1);
        cleanup(&path);
    }

    #[test]
    fn test_cancel_job() {
        let path = unique_test_path();
        let mut scheduler = JobScheduler::new(&path);

        let job = BackgroundJob::doc_refresh();
        let id = scheduler.enqueue(job);

        assert!(scheduler.cancel(&id));

        let job = scheduler.get(&id).unwrap();
        assert!(matches!(job.status, JobStatus::Cancelled { .. }));
        cleanup(&path);
    }

    #[test]
    fn test_job_filter() {
        let path = unique_test_path();
        let mut scheduler = JobScheduler::new(&path);

        scheduler.enqueue(BackgroundJob::doc_refresh());
        scheduler.enqueue(BackgroundJob::long_ticket("TKT-1"));

        let pending = scheduler.get_jobs(JobFilter::pending());
        assert_eq!(pending.len(), 2);

        let running = scheduler.get_jobs(JobFilter::running());
        assert!(running.is_empty());
        cleanup(&path);
    }
}
