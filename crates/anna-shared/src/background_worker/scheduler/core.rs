//! Core scheduler implementation.

use super::super::job::{BackgroundJob, JobPriority, JobStatus};
use super::super::storage::JobStorage;
use super::helpers::now_timestamp;
use super::types::{JobFilter, SchedulerConfig, SchedulerStats, SchedulerStatusSummary};
use std::collections::HashMap;

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
