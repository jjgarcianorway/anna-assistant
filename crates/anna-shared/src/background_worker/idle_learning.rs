//! Idle-time learning system (v0.0.430).
//!
//! Runs low-priority learning tasks when the system is idle:
//! - Recipe consolidation from past tickets
//! - Documentation index refresh
//! - Model performance benchmarking

use super::executor::CpuMonitor;
use super::job::BackgroundJob;

pub use super::idle_learning_handlers::*;
pub use super::idle_learning_types::*;

use super::idle_learning_types::{now_timestamp, IdleLearningConfig, IdleLearningState};

/// Idle learning manager
pub struct IdleLearningManager {
    config: IdleLearningConfig,
    state: IdleLearningState,
}

impl IdleLearningManager {
    /// Create new manager
    pub fn new(config: IdleLearningConfig) -> Self {
        Self {
            config,
            state: IdleLearningState::default(),
        }
    }

    /// Create with existing state
    pub fn with_state(config: IdleLearningConfig, state: IdleLearningState) -> Self {
        Self { config, state }
    }

    /// Check if idle learning should run
    pub fn should_run(&mut self) -> bool {
        if !self.config.enabled {
            return false;
        }

        self.state.check_daily_reset();

        if self.state.jobs_today >= self.config.max_jobs_per_day {
            return false;
        }

        // Check CPU load
        let is_idle = CpuMonitor::is_idle(self.config.cpu_threshold);

        if is_idle {
            self.state.mark_idle();
        } else {
            self.state.mark_busy();
            return false;
        }

        // Check minimum idle time
        let idle_duration = self.state.idle_duration().unwrap_or(0);
        idle_duration >= self.config.min_idle_time_secs
    }

    /// Get next job to run (if any)
    pub fn get_next_job(&self) -> Option<BackgroundJob> {
        let now = now_timestamp();

        // Prioritize: recipe consolidation > doc refresh > benchmark
        if self.config.recipe_consolidation && self.should_run_recipe_consolidation(now) {
            return Some(BackgroundJob::recipe_consolidation());
        }

        if self.config.doc_refresh && self.should_run_doc_refresh(now) {
            return Some(BackgroundJob::doc_refresh());
        }

        if self.config.model_benchmark && self.should_run_benchmark(now) {
            return Some(BackgroundJob::model_benchmark());
        }

        None
    }

    /// Check if recipe consolidation should run
    fn should_run_recipe_consolidation(&self, now: u64) -> bool {
        // Run every 24 hours
        match self.state.last_recipe_consolidation {
            Some(last) => now.saturating_sub(last) >= 86400,
            None => true,
        }
    }

    /// Check if doc refresh should run
    fn should_run_doc_refresh(&self, now: u64) -> bool {
        // Run every 12 hours
        match self.state.last_doc_refresh {
            Some(last) => now.saturating_sub(last) >= 43200,
            None => true,
        }
    }

    /// Check if benchmark should run
    fn should_run_benchmark(&self, now: u64) -> bool {
        // Run every 7 days
        match self.state.last_benchmark {
            Some(last) => now.saturating_sub(last) >= 604800,
            None => true,
        }
    }

    /// Record job completion
    pub fn record_completion(&mut self, job: &BackgroundJob, _result: &super::job::JobResult) {
        self.state.jobs_today += 1;
        let now = now_timestamp();

        match &job.kind {
            super::job::JobKind::RecipeConsolidation => {
                self.state.last_recipe_consolidation = Some(now);
                self.state.total_recipes_consolidated += 1;
            }
            super::job::JobKind::DocIndexRefresh => {
                self.state.last_doc_refresh = Some(now);
                self.state.total_docs_indexed += 1;
            }
            super::job::JobKind::ModelBenchmark => {
                self.state.last_benchmark = Some(now);
            }
            _ => {}
        }
    }

    /// Get current state
    pub fn state(&self) -> &IdleLearningState {
        &self.state
    }

    /// Get config
    pub fn config(&self) -> &IdleLearningConfig {
        &self.config
    }

    /// Get status summary
    pub fn status(&mut self) -> IdleLearningStatus {
        self.state.check_daily_reset();

        IdleLearningStatus {
            enabled: self.config.enabled,
            jobs_today: self.state.jobs_today,
            max_jobs_per_day: self.config.max_jobs_per_day,
            is_idle: self.state.idle_since.is_some(),
            idle_duration_secs: self.state.idle_duration(),
            last_recipe_consolidation: self.state.last_recipe_consolidation,
            last_doc_refresh: self.state.last_doc_refresh,
            last_benchmark: self.state.last_benchmark,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::background_worker::idle_learning_types::current_date;

    #[test]
    fn test_idle_manager_max_jobs() {
        let config = IdleLearningConfig {
            max_jobs_per_day: 2,
            ..Default::default()
        };
        let mut state = IdleLearningState::default();
        state.jobs_today = 2;
        state.last_reset_date = current_date();

        let mut manager = IdleLearningManager::with_state(config, state);
        assert!(!manager.should_run());
    }
}
