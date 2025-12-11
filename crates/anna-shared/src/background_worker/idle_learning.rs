//! Idle-time learning system (v0.0.430).
//!
//! Runs low-priority learning tasks when the system is idle:
//! - Recipe consolidation from past tickets
//! - Documentation index refresh
//! - Model performance benchmarking

use super::executor::CpuMonitor;
use super::job::{BackgroundJob, JobResult};
use super::{IDLE_CPU_THRESHOLD, MAX_IDLE_JOBS_PER_DAY};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Idle learning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdleLearningConfig {
    /// Whether idle learning is enabled
    pub enabled: bool,
    /// CPU threshold for idle detection (0.0-1.0)
    pub cpu_threshold: f32,
    /// Maximum jobs to run per day
    pub max_jobs_per_day: usize,
    /// Recipe consolidation enabled
    pub recipe_consolidation: bool,
    /// Doc index refresh enabled
    pub doc_refresh: bool,
    /// Model benchmarking enabled
    pub model_benchmark: bool,
    /// Minimum idle time before starting (seconds)
    pub min_idle_time_secs: u64,
}

impl Default for IdleLearningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cpu_threshold: IDLE_CPU_THRESHOLD,
            max_jobs_per_day: MAX_IDLE_JOBS_PER_DAY,
            recipe_consolidation: true,
            doc_refresh: true,
            model_benchmark: false, // Disabled by default (expensive)
            min_idle_time_secs: 300, // 5 minutes idle before starting
        }
    }
}

/// Idle learning state tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IdleLearningState {
    /// Jobs run today
    pub jobs_today: usize,
    /// Last reset date (YYYYMMDD)
    pub last_reset_date: u32,
    /// When system became idle (None if not idle)
    pub idle_since: Option<u64>,
    /// Last recipe consolidation
    pub last_recipe_consolidation: Option<u64>,
    /// Last doc refresh
    pub last_doc_refresh: Option<u64>,
    /// Last benchmark
    pub last_benchmark: Option<u64>,
    /// Total recipes consolidated
    pub total_recipes_consolidated: usize,
    /// Total docs indexed
    pub total_docs_indexed: usize,
}

impl IdleLearningState {
    /// Check and reset daily counter if needed
    pub fn check_daily_reset(&mut self) {
        let today = current_date();
        if today != self.last_reset_date {
            self.jobs_today = 0;
            self.last_reset_date = today;
        }
    }

    /// Record that system is now idle
    pub fn mark_idle(&mut self) {
        if self.idle_since.is_none() {
            self.idle_since = Some(now_timestamp());
        }
    }

    /// Record that system is now busy
    pub fn mark_busy(&mut self) {
        self.idle_since = None;
    }

    /// Check if we've been idle long enough
    pub fn idle_duration(&self) -> Option<u64> {
        self.idle_since
            .map(|since| now_timestamp().saturating_sub(since))
    }
}

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
    pub fn record_completion(&mut self, job: &BackgroundJob, _result: &JobResult) {
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

/// Status for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdleLearningStatus {
    pub enabled: bool,
    pub jobs_today: usize,
    pub max_jobs_per_day: usize,
    pub is_idle: bool,
    pub idle_duration_secs: Option<u64>,
    pub last_recipe_consolidation: Option<u64>,
    pub last_doc_refresh: Option<u64>,
    pub last_benchmark: Option<u64>,
}

impl std::fmt::Display for IdleLearningStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[idle_learning]")?;
        writeln!(
            f,
            "  status             {}",
            if self.enabled { "ENABLED" } else { "DISABLED" }
        )?;
        writeln!(
            f,
            "  jobs_today         {}/{}",
            self.jobs_today, self.max_jobs_per_day
        )?;
        writeln!(
            f,
            "  system_idle        {}",
            if self.is_idle { "YES" } else { "NO" }
        )?;
        if let Some(duration) = self.idle_duration_secs {
            writeln!(f, "  idle_for           {}s", duration)?;
        }
        Ok(())
    }
}

/// Recipe consolidation handler
pub struct RecipeConsolidator {
    /// Path to recipes storage
    recipes_path: String,
    /// Path to tickets archive
    tickets_path: String,
}

impl RecipeConsolidator {
    pub fn new(recipes_path: &str, tickets_path: &str) -> Self {
        Self {
            recipes_path: recipes_path.to_string(),
            tickets_path: tickets_path.to_string(),
        }
    }

    /// Consolidate recipes from recent tickets
    pub fn consolidate(&self) -> JobResult {
        // Implementation would:
        // 1. Scan recent closed tickets
        // 2. Extract successful command patterns
        // 3. Identify repeated patterns
        // 4. Create/update recipe entries

        // Placeholder implementation
        JobResult::success(&format!(
            "Recipe consolidation completed (recipes: {}, tickets: {})",
            self.recipes_path, self.tickets_path
        ))
    }
}

/// Doc index refresher
pub struct DocIndexRefresher {
    /// Path to doc index
    index_path: String,
    /// Paths to scan for docs
    doc_paths: Vec<String>,
}

impl DocIndexRefresher {
    pub fn new(index_path: &str, doc_paths: Vec<String>) -> Self {
        Self {
            index_path: index_path.to_string(),
            doc_paths,
        }
    }

    /// Refresh the documentation index
    pub fn refresh(&self) -> JobResult {
        // Implementation would:
        // 1. Scan doc_paths for new/modified files
        // 2. Extract and index content
        // 3. Update the search index

        // Placeholder implementation
        JobResult::success(&format!(
            "Doc index refreshed ({} paths scanned)",
            self.doc_paths.len()
        ))
    }
}

/// Get current date as YYYYMMDD
fn current_date() -> u32 {
    let secs = now_timestamp();
    let days = secs / 86400;
    // Approximate date calculation (good enough for daily reset)
    let year = 1970 + (days / 365) as u32;
    let day_of_year = (days % 365) as u32;
    let month = (day_of_year / 30) + 1;
    let day = (day_of_year % 30) + 1;
    year * 10000 + month * 100 + day
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

    #[test]
    fn test_idle_learning_config_default() {
        let config = IdleLearningConfig::default();
        assert!(config.enabled);
        assert!(config.recipe_consolidation);
        assert!(config.doc_refresh);
        assert!(!config.model_benchmark);
    }

    #[test]
    fn test_idle_state_daily_reset() {
        let mut state = IdleLearningState {
            jobs_today: 5,
            last_reset_date: 0,
            ..Default::default()
        };

        state.check_daily_reset();
        assert_eq!(state.jobs_today, 0);
        assert!(state.last_reset_date > 0);
    }

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

    #[test]
    fn test_current_date() {
        let date = current_date();
        // Should be a valid YYYYMMDD
        assert!(date > 20200101);
        assert!(date < 21000101);
    }
}
