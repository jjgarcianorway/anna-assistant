//! Idle-time learning types and configuration (v0.0.430).

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

use super::{IDLE_CPU_THRESHOLD, MAX_IDLE_JOBS_PER_DAY};

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
            model_benchmark: false,  // Disabled by default (expensive)
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

/// Get current date as YYYYMMDD
pub(crate) fn current_date() -> u32 {
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
pub(crate) fn now_timestamp() -> u64 {
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
    fn test_current_date() {
        let date = current_date();
        // Should be a valid YYYYMMDD
        assert!(date > 20200101);
        assert!(date < 21000101);
    }
}
