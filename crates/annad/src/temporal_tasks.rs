//! Temporal task support - background operations with timers.
//! Enables Anna to "capture data for X minutes", "monitor for 1 hour", etc.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tokio::time::sleep;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// A background task with a time limit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalTask {
    pub id: String,
    pub description: String,
    pub start_command: String,
    pub stop_command: Option<String>,
    pub duration_secs: u64,
    pub output_path: PathBuf,
    pub started_at: SystemTime,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Global registry of running temporal tasks.
lazy_static::lazy_static! {
    static ref TASK_REGISTRY: Arc<Mutex<HashMap<String, TemporalTask>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

/// Start a background task that runs for a specific duration.
pub async fn start_temporal_task(
    description: String,
    start_command: String,
    stop_command: Option<String>,
    duration_secs: u64,
) -> Result<TemporalTask> {
    let task_id = Uuid::new_v4().to_string();
    let output_path = PathBuf::from(format!("/tmp/anna_task_{}.log", task_id));

    info!(
        "Starting temporal task '{}' for {}s",
        description, duration_secs
    );

    // Execute start command
    let start_cmd_with_output = format!("{} > {} 2>&1 &", start_command, output_path.display());
    crate::core_loop::execute_command(&format!("sh -c '{}'", start_cmd_with_output))?;

    let task = TemporalTask {
        id: task_id.clone(),
        description: description.clone(),
        start_command,
        stop_command,
        duration_secs,
        output_path: output_path.clone(),
        started_at: SystemTime::now(),
        status: TaskStatus::Running,
    };

    // Register task
    {
        let mut registry = TASK_REGISTRY.lock().unwrap();
        registry.insert(task_id.clone(), task.clone());
    }

    // Spawn background task to stop after duration
    let task_clone = task.clone();
    tokio::spawn(async move {
        sleep(Duration::from_secs(task_clone.duration_secs)).await;
        if let Err(e) = complete_temporal_task(&task_clone.id).await {
            warn!("Failed to complete temporal task {}: {}", task_clone.id, e);
        }
    });

    info!("Temporal task {} started, will complete in {}s", task_id, duration_secs);
    Ok(task)
}

/// Complete a temporal task (called automatically after duration).
async fn complete_temporal_task(task_id: &str) -> Result<()> {
    let task = {
        let mut registry = TASK_REGISTRY.lock().unwrap();
        match registry.get_mut(task_id) {
            Some(t) => {
                if t.status != TaskStatus::Running {
                    return Ok(()); // Already completed
                }
                t.status = TaskStatus::Completed;
                t.clone()
            }
            None => return Err(anyhow!("Task {} not found", task_id)),
        }
    };

    info!("Completing temporal task {}: {}", task_id, task.description);

    // Execute stop command if provided
    if let Some(stop_cmd) = &task.stop_command {
        debug!("Executing stop command: {}", stop_cmd);
        if let Err(e) = crate::core_loop::execute_command(stop_cmd) {
            warn!("Stop command failed: {}", e);
        }
    }

    info!("Temporal task {} completed", task_id);
    Ok(())
}

/// Get status of a temporal task.
pub fn get_task_status(task_id: &str) -> Option<TemporalTask> {
    let registry = TASK_REGISTRY.lock().unwrap();
    registry.get(task_id).cloned()
}

/// Get all active temporal tasks.
pub fn get_active_tasks() -> Vec<TemporalTask> {
    let registry = TASK_REGISTRY.lock().unwrap();
    registry
        .values()
        .filter(|t| t.status == TaskStatus::Running)
        .cloned()
        .collect()
}

/// Read output from a temporal task.
pub fn read_task_output(task_id: &str) -> Result<String> {
    let task = get_task_status(task_id)
        .ok_or_else(|| anyhow!("Task {} not found", task_id))?;

    std::fs::read_to_string(&task.output_path)
        .map_err(|e| anyhow!("Failed to read task output: {}", e))
}

/// Cancel a running temporal task.
pub async fn cancel_task(task_id: &str) -> Result<()> {
    let task = {
        let mut registry = TASK_REGISTRY.lock().unwrap();
        match registry.get_mut(task_id) {
            Some(t) => {
                if t.status != TaskStatus::Running {
                    return Err(anyhow!("Task {} is not running", task_id));
                }
                t.status = TaskStatus::Cancelled;
                t.clone()
            }
            None => return Err(anyhow!("Task {} not found", task_id)),
        }
    };

    info!("Cancelling temporal task {}", task_id);

    // Execute stop command if provided
    if let Some(stop_cmd) = &task.stop_command {
        crate::core_loop::execute_command(stop_cmd)?;
    }

    Ok(())
}

/// Analyze a question to detect temporal requirements.
/// Returns (duration_seconds, should_use_temporal)
pub fn detect_temporal_requirement(question: &str) -> Option<u64> {
    let q = question.to_lowercase();

    // Check for time expressions
    let patterns = [
        (r"(\d+)\s*minutes?", 60),
        (r"(\d+)\s*hours?", 3600),
        (r"(\d+)\s*seconds?", 1),
        (r"(\d+)\s*mins?", 60),
        (r"(\d+)\s*hrs?", 3600),
        (r"(\d+)\s*secs?", 1),
    ];

    for (pattern, multiplier) in patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(captures) = re.captures(&q) {
                if let Some(num_match) = captures.get(1) {
                    if let Ok(num) = num_match.as_str().parse::<u64>() {
                        let duration = num * multiplier;
                        if duration > 0 && duration <= 3600 * 24 {
                            // Max 24 hours
                            return Some(duration);
                        }
                    }
                }
            }
        }
    }

    // Check for temporal keywords without specific duration
    if q.contains("monitor") || q.contains("capture") || q.contains("watch") || q.contains("track")
    {
        // Default to 5 minutes if no duration specified
        if !q.contains("minute") && !q.contains("hour") && !q.contains("second") {
            return Some(300); // 5 minutes default
        }
    }

    None
}

/// Check if a question requires background monitoring.
pub fn requires_background_monitoring(question: &str) -> bool {
    let q = question.to_lowercase();

    let monitoring_keywords = [
        "capture",
        "monitor",
        "watch",
        "track",
        "record",
        "log",
        "observe",
        "scan for",
        "look for",
    ];

    monitoring_keywords.iter().any(|kw| q.contains(kw))
        && (q.contains("for ") || q.contains("minute") || q.contains("hour"))
}
