//! Explainability Layer
//!
//! Every autonomous action taken by Anna must be explainable.
//! This module tracks:
//! - What action was taken
//! - Why it was taken (triggering condition)
//! - What the result was (success/failure)
//! - Timestamp and context

use anyhow::{Context, Result};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};

const ADAPTIVE_LOG: &str = "/var/log/anna/adaptive.log";
const MAX_RECENT_ACTIONS: usize = 100;

/// An action taken by Anna's autonomous system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub timestamp: String,
    pub name: String,
    pub condition: String,
    pub reason: String,
    pub command: Option<String>,
    pub result: ActionResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionResult {
    Success,
    Failed { error: String },
    Skipped { reason: String },
}

impl ActionResult {
    pub fn to_string(&self) -> String {
        match self {
            ActionResult::Success => "SUCCESS".to_string(),
            ActionResult::Failed { error } => format!("FAILED: {}", error),
            ActionResult::Skipped { reason } => format!("SKIPPED: {}", reason),
        }
    }
}

/// Action logger that maintains both file log and in-memory recent actions
pub struct ActionLogger {
    recent: Arc<Mutex<Vec<Action>>>,
    log_path: String,
}

impl ActionLogger {
    pub fn new() -> Self {
        Self {
            recent: Arc::new(Mutex::new(Vec::with_capacity(MAX_RECENT_ACTIONS))),
            log_path: ADAPTIVE_LOG.to_string(),
        }
    }

    /// Log an action to both file and memory
    pub fn log_action(&self, action: Action) -> Result<()> {
        // Write to file
        self.write_to_file(&action)?;

        // Store in memory (keep last N actions)
        let mut recent = self.recent.lock().unwrap();
        if recent.len() >= MAX_RECENT_ACTIONS {
            recent.remove(0);
        }
        recent.push(action);

        Ok(())
    }

    /// Get last N actions from memory
    pub fn get_recent(&self, count: usize) -> Vec<Action> {
        let recent = self.recent.lock().unwrap();
        recent
            .iter()
            .rev()
            .take(count)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Get all recent actions
    pub fn get_all_recent(&self) -> Vec<Action> {
        let recent = self.recent.lock().unwrap();
        recent.clone()
    }

    fn write_to_file(&self, action: &Action) -> Result<()> {
        // Ensure log directory exists
        if let Some(parent) = Path::new(&self.log_path).parent() {
            if !parent.exists() {
                // Try to create with elevated privileges
                let _ = std::process::Command::new("sudo")
                    .args(&["mkdir", "-p", parent.to_str().unwrap()])
                    .output();
            }
        }

        // Format log entry
        let log_entry = format!(
            "[{}] ACTION {} → triggered by {}\nWHY: {}\nCOMMAND: {}\nRESULT: {}\n\n",
            action.timestamp,
            action.name,
            action.condition,
            action.reason,
            action.command.as_ref().unwrap_or(&"<none>".to_string()),
            action.result.to_string()
        );

        // Try direct write first
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            file.write_all(log_entry.as_bytes())?;
            return Ok(());
        }

        // Fallback: write to temp and copy with sudo
        let temp_path = "/tmp/anna_adaptive.log";
        fs::write(temp_path, log_entry.as_bytes())
            .context("Failed to write temporary log")?;

        let _ = std::process::Command::new("sudo")
            .args(&["bash", "-c", &format!("cat {} >> {}", temp_path, self.log_path)])
            .output();

        let _ = fs::remove_file(temp_path);

        Ok(())
    }

    /// Read actions from log file (for annactl explain)
    pub fn read_log_file(&self, count: usize) -> Result<Vec<String>> {
        if !Path::new(&self.log_path).exists() {
            return Ok(vec!["No actions logged yet.".to_string()]);
        }

        let content = fs::read_to_string(&self.log_path)
            .context("Failed to read adaptive log")?;

        // Split into action blocks (separated by blank lines)
        let blocks: Vec<String> = content
            .split("\n\n")
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string())
            .collect();

        // Return last N blocks
        let result = blocks
            .iter()
            .rev()
            .take(count)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        Ok(result)
    }
}

/// Create a new action record
pub fn create_action(
    name: impl Into<String>,
    condition: impl Into<String>,
    reason: impl Into<String>,
    command: Option<String>,
    result: ActionResult,
) -> Action {
    Action {
        timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        name: name.into(),
        condition: condition.into(),
        reason: reason.into(),
        command,
        result,
    }
}

/// Global action logger singleton
static mut LOGGER: Option<Arc<ActionLogger>> = None;

/// Initialize the global logger
pub fn init_logger() {
    unsafe {
        LOGGER = Some(Arc::new(ActionLogger::new()));
    }
}

/// Get the global logger
pub fn get_logger() -> Option<Arc<ActionLogger>> {
    unsafe { LOGGER.clone() }
}

/// Log an action using the global logger
pub fn log_action(action: Action) -> Result<()> {
    if let Some(logger) = get_logger() {
        logger.log_action(action)
    } else {
        anyhow::bail!("Logger not initialized")
    }
}
