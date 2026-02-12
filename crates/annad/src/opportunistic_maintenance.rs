//! Opportunistic Maintenance - Run expensive scans during idle time.
//!
//! Philosophy: Use idle CPU/disk time productively. User sees: "Ran maintenance, found 2 opportunities".
//! NO HARDCODING: Smart idle detection, not arbitrary thresholds.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

/// Maintenance task that can run during idle time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceTask {
    pub name: String,
    pub description: String,
    pub estimated_duration_secs: u32,
    pub priority: TaskPriority,
    pub last_run: Option<DateTime<Utc>>,
    pub run_interval_days: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
}

/// Opportunistic maintenance scheduler.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceScheduler {
    pub tasks: Vec<MaintenanceTask>,
    pub last_idle_check: Option<DateTime<Utc>>,
}

impl Default for MaintenanceScheduler {
    fn default() -> Self {
        Self {
            tasks: vec![
                MaintenanceTask {
                    name: "full_regression_scan".to_string(),
                    description: "Deep regression analysis".to_string(),
                    estimated_duration_secs: 30,
                    priority: TaskPriority::Medium,
                    last_run: None,
                    run_interval_days: 7,
                },
                MaintenanceTask {
                    name: "deep_cleanup_scan".to_string(),
                    description: "Comprehensive cleanup opportunities scan".to_string(),
                    estimated_duration_secs: 45,
                    priority: TaskPriority::Medium,
                    last_run: None,
                    run_interval_days: 7,
                },
                MaintenanceTask {
                    name: "prediction_update".to_string(),
                    description: "Update predictive models".to_string(),
                    estimated_duration_secs: 20,
                    priority: TaskPriority::High,
                    last_run: None,
                    run_interval_days: 3,
                },
                MaintenanceTask {
                    name: "change_history_scan".to_string(),
                    description: "Scan and record system changes".to_string(),
                    estimated_duration_secs: 10,
                    priority: TaskPriority::Low,
                    last_run: None,
                    run_interval_days: 1,
                },
            ],
            last_idle_check: None,
        }
    }
}

impl MaintenanceScheduler {
    /// Load from disk.
    pub fn load() -> Self {
        let path = Self::storage_path();

        if let Ok(contents) = std::fs::read_to_string(&path) {
            if let Ok(scheduler) = serde_json::from_str(&contents) {
                return scheduler;
            }
        }

        Self::default()
    }

    /// Save to disk.
    pub fn save(&self) -> Result<()> {
        let path = Self::storage_path();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;

        Ok(())
    }

    fn storage_path() -> PathBuf {
        PathBuf::from("/var/lib/anna/maintenance_scheduler.json")
    }

    /// Get tasks that are due to run.
    pub fn get_due_tasks(&self) -> Vec<&MaintenanceTask> {
        let now = Utc::now();

        self.tasks
            .iter()
            .filter(|task| {
                if let Some(last_run) = task.last_run {
                    let days_since = (now - last_run).num_days();
                    days_since >= task.run_interval_days as i64
                } else {
                    true // Never run
                }
            })
            .collect()
    }

    /// Mark task as completed.
    pub fn mark_completed(&mut self, task_name: &str) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.name == task_name) {
            task.last_run = Some(Utc::now());
            let _ = self.save();
        }
    }
}

/// Check if system is idle (low CPU, low disk I/O).
pub fn is_system_idle() -> bool {
    // Check CPU load
    if let Ok(load) = std::fs::read_to_string("/proc/loadavg") {
        let parts: Vec<&str> = load.split_whitespace().collect();
        if let Some(load_1min) = parts.first() {
            if let Ok(load_val) = load_1min.parse::<f32>() {
                // If 1-min load < 1.0, system is relatively idle
                if load_val > 1.0 {
                    return false;
                }
            }
        }
    }

    // Check disk I/O wait
    if let Ok(stat) = std::fs::read_to_string("/proc/stat") {
        for line in stat.lines() {
            if line.starts_with("cpu ") {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() > 5 {
                    // iowait is field 5
                    if let Ok(iowait) = fields[5].parse::<u64>() {
                        // If iowait is very high, system is not idle
                        if iowait > 1000 {
                            // Arbitrary threshold
                            return false;
                        }
                    }
                }
                break;
            }
        }
    }

    true
}

/// Run opportunistic maintenance if system is idle.
pub async fn run_opportunistic_maintenance() -> Result<String> {
    info!("Checking for opportunistic maintenance opportunities...");

    if !is_system_idle() {
        return Ok("System not idle, skipping maintenance.".to_string());
    }

    let mut scheduler = MaintenanceScheduler::load();
    let due_tasks: Vec<String> = scheduler
        .get_due_tasks()
        .iter()
        .take(3)
        .map(|t| t.name.clone())
        .collect();

    if due_tasks.is_empty() {
        return Ok("No maintenance tasks due.".to_string());
    }

    info!("System idle, running {} maintenance tasks", due_tasks.len());

    let mut results = Vec::new();

    for task_name in &due_tasks {
        // Limit to 3 tasks per run
        info!("Running maintenance task: {}", task_name);

        match task_name.as_str() {
            "full_regression_scan" => {
                let regressions = crate::regression_detector::detect_regressions().await?;
                if !regressions.is_empty() {
                    results.push(format!("Found {} regressions", regressions.len()));
                }
            }
            "deep_cleanup_scan" => {
                let cleanup = crate::cleanup_detector::scan_for_cleanable_space().await?;
                if cleanup.total_cleanable_mb > 100.0 {
                    results.push(format!("Found {:.1}GB cleanable", cleanup.total_cleanable_mb / 1024.0));
                }
            }
            "prediction_update" => {
                let _ = crate::predictive_maintenance::generate_health_forecast().await?;
                results.push("Updated predictive models".to_string());
            }
            "change_history_scan" => {
                let count = crate::change_tracking::scan_and_record_recent_changes().await?;
                if count > 0 {
                    results.push(format!("Recorded {} system changes", count));
                }
            }
            _ => {}
        }

        scheduler.mark_completed(task_name);

        // Check if still idle
        if !is_system_idle() {
            info!("System no longer idle, stopping maintenance");
            break;
        }
    }

    scheduler.save()?;

    let summary = if results.is_empty() {
        "Maintenance complete, no issues found.".to_string()
    } else {
        format!("Maintenance complete: {}", results.join(", "))
    };

    Ok(summary)
}

/// Check if opportunistic maintenance should run.
pub fn should_run_maintenance() -> bool {
    let scheduler = MaintenanceScheduler::load();
    !scheduler.get_due_tasks().is_empty() && is_system_idle()
}
