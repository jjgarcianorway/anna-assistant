//! Idle-only benchmark scheduler (v0.0.75).
//!
//! Ensures model benchmarks only run when system is idle and can be interrupted.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Notify;
use tracing::{debug, info, warn};

/// Minimum idle time before starting benchmark (seconds)
const MIN_IDLE_SECS: u64 = 30;

/// Maximum benchmark duration before forced interrupt (seconds)
const MAX_BENCHMARK_SECS: u64 = 60;

/// Cooldown between benchmark attempts (seconds)
const BENCHMARK_COOLDOWN_SECS: u64 = 300; // 5 minutes

/// Benchmark scheduler state
pub struct BenchmarkScheduler {
    /// Whether a benchmark is currently running
    running: Arc<AtomicBool>,
    /// Request for interruption
    interrupt_requested: Arc<AtomicBool>,
    /// Notify for wake-up
    notify: Arc<Notify>,
    /// Last request timestamp (Unix seconds)
    last_request_time: Arc<AtomicU64>,
    /// Last benchmark timestamp (Unix seconds)
    last_benchmark_time: Arc<AtomicU64>,
    /// Whether scheduler is enabled
    enabled: Arc<AtomicBool>,
}

impl BenchmarkScheduler {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            interrupt_requested: Arc::new(AtomicBool::new(false)),
            notify: Arc::new(Notify::new()),
            last_request_time: Arc::new(AtomicU64::new(current_timestamp())),
            last_benchmark_time: Arc::new(AtomicU64::new(0)),
            enabled: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Record that a user request occurred (resets idle timer)
    pub fn record_request(&self) {
        self.last_request_time
            .store(current_timestamp(), Ordering::SeqCst);

        // If benchmark is running, request interrupt
        if self.running.load(Ordering::SeqCst) {
            debug!("User request during benchmark - requesting interrupt");
            self.interrupt_requested.store(true, Ordering::SeqCst);
            self.notify.notify_one();
        }
    }

    /// Check if system is idle enough for benchmarking
    pub fn is_idle(&self) -> bool {
        let now = current_timestamp();
        let last_request = self.last_request_time.load(Ordering::SeqCst);
        now.saturating_sub(last_request) >= MIN_IDLE_SECS
    }

    /// Check if enough time has passed since last benchmark
    pub fn cooldown_elapsed(&self) -> bool {
        let now = current_timestamp();
        let last_benchmark = self.last_benchmark_time.load(Ordering::SeqCst);
        now.saturating_sub(last_benchmark) >= BENCHMARK_COOLDOWN_SECS
    }

    /// Check if benchmark should run
    pub fn should_run(&self) -> bool {
        self.enabled.load(Ordering::SeqCst) && self.is_idle() && self.cooldown_elapsed()
    }

    /// Try to acquire benchmark lock
    pub fn try_start(&self) -> Option<BenchmarkGuard> {
        if !self.should_run() {
            return None;
        }

        // Try to acquire running lock
        if self
            .running
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            self.interrupt_requested.store(false, Ordering::SeqCst);
            info!("Starting benchmark (system idle)");
            Some(BenchmarkGuard {
                scheduler: self.clone_refs(),
                start_time: Instant::now(),
            })
        } else {
            None
        }
    }

    /// Check if interrupt was requested
    pub fn is_interrupted(&self) -> bool {
        self.interrupt_requested.load(Ordering::SeqCst)
    }

    /// Wait for interrupt or timeout
    pub async fn wait_interruptible(&self, duration: Duration) -> bool {
        let timeout = tokio::time::timeout(duration, self.notify.notified()).await;
        timeout.is_err() // true = completed without interrupt
    }

    /// Enable/disable scheduler
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::SeqCst);
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    fn clone_refs(&self) -> SchedulerRefs {
        SchedulerRefs {
            running: self.running.clone(),
            interrupt_requested: self.interrupt_requested.clone(),
            last_benchmark_time: self.last_benchmark_time.clone(),
        }
    }
}

impl Default for BenchmarkScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for BenchmarkScheduler {
    fn clone(&self) -> Self {
        Self {
            running: self.running.clone(),
            interrupt_requested: self.interrupt_requested.clone(),
            notify: self.notify.clone(),
            last_request_time: self.last_request_time.clone(),
            last_benchmark_time: self.last_benchmark_time.clone(),
            enabled: self.enabled.clone(),
        }
    }
}

/// Internal refs for guard
struct SchedulerRefs {
    running: Arc<AtomicBool>,
    interrupt_requested: Arc<AtomicBool>,
    last_benchmark_time: Arc<AtomicU64>,
}

/// Guard that releases benchmark lock on drop
pub struct BenchmarkGuard {
    scheduler: SchedulerRefs,
    start_time: Instant,
}

impl BenchmarkGuard {
    /// Check if should abort due to interrupt or timeout
    pub fn should_abort(&self) -> bool {
        if self.scheduler.interrupt_requested.load(Ordering::SeqCst) {
            debug!("Benchmark interrupted by user request");
            return true;
        }

        if self.start_time.elapsed() > Duration::from_secs(MAX_BENCHMARK_SECS) {
            warn!("Benchmark exceeded max duration - aborting");
            return true;
        }

        false
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Complete benchmark successfully
    pub fn complete(self) {
        info!(
            "Benchmark completed in {:.1}s",
            self.start_time.elapsed().as_secs_f32()
        );
        // Drop will release lock and record time
    }
}

impl Drop for BenchmarkGuard {
    fn drop(&mut self) {
        self.scheduler.running.store(false, Ordering::SeqCst);
        self.scheduler
            .last_benchmark_time
            .store(current_timestamp(), Ordering::SeqCst);
    }
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Benchmark task that runs only when idle
pub struct BenchmarkTask<F>
where
    F: Fn() -> bool + Send + 'static,
{
    scheduler: BenchmarkScheduler,
    task: F,
}

impl<F> BenchmarkTask<F>
where
    F: Fn() -> bool + Send + 'static,
{
    pub fn new(scheduler: BenchmarkScheduler, task: F) -> Self {
        Self { scheduler, task }
    }

    /// Try to run the task if conditions allow
    pub fn try_run(&self) -> Option<bool> {
        let guard = self.scheduler.try_start()?;

        // Run task with periodic interrupt checks
        let result = (self.task)();

        if guard.should_abort() {
            return None; // Interrupted
        }

        guard.complete();
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_initial_state() {
        let scheduler = BenchmarkScheduler::new();
        assert!(!scheduler.is_idle()); // Just created, recent "request"
        assert!(scheduler.cooldown_elapsed()); // No previous benchmark
        assert!(scheduler.is_enabled());
    }

    #[test]
    fn test_record_request_resets_idle() {
        let scheduler = BenchmarkScheduler::new();

        // Manually set old timestamp
        scheduler.last_request_time.store(0, Ordering::SeqCst);
        assert!(scheduler.is_idle());

        // Record new request
        scheduler.record_request();
        assert!(!scheduler.is_idle());
    }

    #[test]
    fn test_try_start_prevents_concurrent() {
        let scheduler = BenchmarkScheduler::new();

        // Set up idle conditions
        scheduler.last_request_time.store(0, Ordering::SeqCst);

        let guard1 = scheduler.try_start();
        assert!(guard1.is_some());

        // Second attempt should fail
        let guard2 = scheduler.try_start();
        assert!(guard2.is_none());

        // After drop, should be able to start again
        drop(guard1);

        // But need to wait for cooldown
        assert!(!scheduler.cooldown_elapsed());
    }

    #[test]
    fn test_interrupt_on_request() {
        let scheduler = BenchmarkScheduler::new();
        scheduler.last_request_time.store(0, Ordering::SeqCst);

        let guard = scheduler.try_start().unwrap();
        assert!(!guard.should_abort());

        // Simulate user request during benchmark
        scheduler.record_request();
        assert!(guard.should_abort());
    }

    #[test]
    fn test_enable_disable() {
        let scheduler = BenchmarkScheduler::new();
        assert!(scheduler.is_enabled());

        scheduler.set_enabled(false);
        assert!(!scheduler.is_enabled());
        assert!(!scheduler.should_run());
    }
}
