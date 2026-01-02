//! Tests for scheduler functionality.

#[cfg(test)]
mod tests {
    use super::super::core::JobScheduler;
    use super::super::helpers::now_timestamp;
    use super::super::types::JobFilter;
    use crate::background_worker::job::{BackgroundJob, JobStatus};
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
        assert_eq!(scheduler.stats().total_completed, 1);
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
