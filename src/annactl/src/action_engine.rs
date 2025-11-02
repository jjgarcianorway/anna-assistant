//! Action Engine for Anna v0.14.0 "Orion III"
//!
//! Safe, reversible autonomous action execution

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Action definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: String,                    // Unique action ID
    pub action_type: String,           // "cache_clean", "log_rotate", "service_restart", etc.
    pub description: String,           // Human-readable description
    pub priority: String,              // "critical", "high", "medium", "low"
    pub command: String,               // Shell command to execute
    pub reversible: bool,              // Can this action be reverted?
    pub revert_command: Option<String>, // Command to undo action
    pub safe_to_autorun: bool,         // Can run without user confirmation?
    pub created_at: u64,               // When action was created
    pub tags: Vec<String>,             // Categories: "cleanup", "optimization", "security"
}

/// Action execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub action_id: String,
    pub executed_at: u64,
    pub actor: String,                 // "auto", "user", "scheduler"
    pub success: bool,
    pub output: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
}

/// Action execution policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionPolicy {
    AutoRun,      // Execute automatically (low/medium priority, safe actions)
    Confirm,      // Require user confirmation (high priority)
    LogOnly,      // Never execute, only log recommendation (critical)
    DryRun,       // Show what would happen without executing
}

impl Action {
    /// Determine execution policy for this action
    pub fn execution_policy(&self) -> ExecutionPolicy {
        match self.priority.as_str() {
            "critical" => ExecutionPolicy::LogOnly,
            "high" => ExecutionPolicy::Confirm,
            "medium" | "low" if self.safe_to_autorun => ExecutionPolicy::AutoRun,
            _ => ExecutionPolicy::Confirm,
        }
    }

    /// Check if action is safe to run automatically
    pub fn is_auto_runnable(&self) -> bool {
        self.safe_to_autorun && matches!(self.priority.as_str(), "medium" | "low")
    }

    /// Get action category emoji
    pub fn emoji(&self) -> &'static str {
        if self.tags.contains(&"cleanup".to_string()) {
            "🧹"
        } else if self.tags.contains(&"optimization".to_string()) {
            "⚡"
        } else if self.tags.contains(&"security".to_string()) {
            "🔒"
        } else if self.tags.contains(&"maintenance".to_string()) {
            "🔧"
        } else {
            "📋"
        }
    }
}

/// Action manager
pub struct ActionManager {
    actions_path: PathBuf,
    results_path: PathBuf,
}

impl ActionManager {
    /// Create new action manager
    pub fn new() -> Result<Self> {
        let state_dir = Self::get_state_dir()?;
        fs::create_dir_all(&state_dir)?;

        let actions_path = state_dir.join("actions.jsonl");
        let results_path = state_dir.join("action_results.jsonl");

        Ok(Self {
            actions_path,
            results_path,
        })
    }

    /// Get state directory
    fn get_state_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME not set")?;
        Ok(PathBuf::from(home).join(".local/state/anna"))
    }

    /// Register a new action
    pub fn register(&self, action: Action) -> Result<()> {
        let json = serde_json::to_string(&action)?;
        let mut content = String::new();

        // Load existing actions
        if self.actions_path.exists() {
            content = fs::read_to_string(&self.actions_path)?;
        }

        // Append new action
        content.push_str(&json);
        content.push('\n');

        fs::write(&self.actions_path, content)?;

        Ok(())
    }

    /// Load all registered actions
    pub fn load_actions(&self) -> Result<Vec<Action>> {
        if !self.actions_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.actions_path)?;
        let mut actions = Vec::new();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<Action>(line) {
                Ok(action) => actions.push(action),
                Err(e) => {
                    eprintln!("Warning: Failed to parse action: {}", e);
                    continue;
                }
            }
        }

        Ok(actions)
    }

    /// Get auto-runnable actions
    pub fn get_auto_runnable(&self) -> Result<Vec<Action>> {
        let all_actions = self.load_actions()?;
        Ok(all_actions.into_iter().filter(|a| a.is_auto_runnable()).collect())
    }

    /// Execute an action (dry-run mode)
    pub fn execute_dry_run(&self, action: &Action) -> Result<String> {
        Ok(format!(
            "Would execute: {}\nCommand: {}\nReversible: {}",
            action.description,
            action.command,
            if action.reversible { "yes" } else { "no" }
        ))
    }

    /// Execute an action (real execution)
    pub fn execute(&self, action: &Action, actor: &str) -> Result<ActionResult> {
        use std::process::Command;
        use std::time::Instant;

        let start = Instant::now();

        let output = Command::new("sh")
            .arg("-c")
            .arg(&action.command)
            .output()
            .context("Failed to execute command")?;

        let duration = start.elapsed();

        let result = ActionResult {
            action_id: action.id.clone(),
            executed_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            actor: actor.to_string(),
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string()
                + &String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
            duration_ms: duration.as_millis() as u64,
        };

        // Log result
        self.record_result(&result)?;

        Ok(result)
    }

    /// Revert an action
    pub fn revert(&self, action: &Action, actor: &str) -> Result<ActionResult> {
        if !action.reversible {
            anyhow::bail!("Action {} is not reversible", action.id);
        }

        let revert_cmd = action
            .revert_command
            .as_ref()
            .context("No revert command defined")?;

        let start = std::time::Instant::now();

        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(revert_cmd)
            .output()
            .context("Failed to execute revert command")?;

        let duration = start.elapsed();

        let result = ActionResult {
            action_id: format!("{}_revert", action.id),
            executed_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            actor: actor.to_string(),
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string()
                + &String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
            duration_ms: duration.as_millis() as u64,
        };

        self.record_result(&result)?;

        Ok(result)
    }

    /// Record action result
    fn record_result(&self, result: &ActionResult) -> Result<()> {
        let json = serde_json::to_string(result)?;
        let mut content = String::new();

        if self.results_path.exists() {
            content = fs::read_to_string(&self.results_path)?;
        }

        content.push_str(&json);
        content.push('\n');

        fs::write(&self.results_path, content)?;

        Ok(())
    }

    /// Load action results
    pub fn load_results(&self) -> Result<Vec<ActionResult>> {
        if !self.results_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.results_path)?;
        let mut results = Vec::new();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<ActionResult>(line) {
                Ok(result) => results.push(result),
                Err(e) => {
                    eprintln!("Warning: Failed to parse result: {}", e);
                    continue;
                }
            }
        }

        Ok(results)
    }

    /// Find action by ID
    pub fn find_action(&self, id: &str) -> Result<Option<Action>> {
        let actions = self.load_actions()?;
        Ok(actions.into_iter().find(|a| a.id == id))
    }

    /// Clear all actions (for testing)
    #[allow(dead_code)]
    pub fn clear(&self) -> Result<()> {
        if self.actions_path.exists() {
            fs::remove_file(&self.actions_path)?;
        }
        if self.results_path.exists() {
            fs::remove_file(&self.results_path)?;
        }
        Ok(())
    }
}

/// Create safe built-in actions
pub fn create_safe_actions() -> Vec<Action> {
    vec![
        Action {
            id: "cleanup_pacman_cache".to_string(),
            action_type: "cache_clean".to_string(),
            description: "Clean pacman package cache (keep last 3 versions)".to_string(),
            priority: "low".to_string(),
            command: "sudo pacman -Sc --noconfirm".to_string(),
            reversible: false,
            revert_command: None,
            safe_to_autorun: true,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tags: vec!["cleanup".to_string(), "disk".to_string()],
        },
        Action {
            id: "rotate_journal_logs".to_string(),
            action_type: "log_rotate".to_string(),
            description: "Rotate system logs older than 30 days".to_string(),
            priority: "low".to_string(),
            command: "sudo journalctl --vacuum-time=30d".to_string(),
            reversible: false,
            revert_command: None,
            safe_to_autorun: true,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tags: vec!["cleanup".to_string(), "logs".to_string()],
        },
        Action {
            id: "cleanup_user_cache".to_string(),
            action_type: "cache_clean".to_string(),
            description: "Clean user cache directory".to_string(),
            priority: "low".to_string(),
            command: "rm -rf ~/.cache/thumbnails/*".to_string(),
            reversible: false,
            revert_command: None,
            safe_to_autorun: true,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tags: vec!["cleanup".to_string(), "user".to_string()],
        },
        Action {
            id: "check_failed_services".to_string(),
            action_type: "diagnostic".to_string(),
            description: "List failed systemd services".to_string(),
            priority: "medium".to_string(),
            command: "systemctl --failed --no-pager".to_string(),
            reversible: false,
            revert_command: None,
            safe_to_autorun: true,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tags: vec!["diagnostic".to_string(), "services".to_string()],
        },
        Action {
            id: "update_package_database".to_string(),
            action_type: "maintenance".to_string(),
            description: "Update package database (no upgrades)".to_string(),
            priority: "medium".to_string(),
            command: "sudo pacman -Sy".to_string(),
            reversible: false,
            revert_command: None,
            safe_to_autorun: false, // Database updates shouldn't auto-run
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tags: vec!["maintenance".to_string(), "updates".to_string()],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_execution_policy() {
        let critical = Action {
            id: "test".to_string(),
            action_type: "test".to_string(),
            description: "test".to_string(),
            priority: "critical".to_string(),
            command: "echo test".to_string(),
            reversible: false,
            revert_command: None,
            safe_to_autorun: true,
            created_at: 0,
            tags: vec![],
        };
        assert_eq!(critical.execution_policy(), ExecutionPolicy::LogOnly);

        let high = Action { priority: "high".to_string(), ..critical.clone() };
        assert_eq!(high.execution_policy(), ExecutionPolicy::Confirm);

        let low_safe = Action {
            priority: "low".to_string(),
            safe_to_autorun: true,
            ..critical.clone()
        };
        assert_eq!(low_safe.execution_policy(), ExecutionPolicy::AutoRun);

        let low_unsafe = Action {
            priority: "low".to_string(),
            safe_to_autorun: false,
            ..critical
        };
        assert_eq!(low_unsafe.execution_policy(), ExecutionPolicy::Confirm);
    }

    #[test]
    fn test_is_auto_runnable() {
        let action = Action {
            id: "test".to_string(),
            action_type: "test".to_string(),
            description: "test".to_string(),
            priority: "low".to_string(),
            command: "echo test".to_string(),
            reversible: false,
            revert_command: None,
            safe_to_autorun: true,
            created_at: 0,
            tags: vec![],
        };
        assert!(action.is_auto_runnable());

        let action_unsafe = Action { safe_to_autorun: false, ..action.clone() };
        assert!(!action_unsafe.is_auto_runnable());

        let action_high = Action { priority: "high".to_string(), ..action };
        assert!(!action_high.is_auto_runnable());
    }

    #[test]
    fn test_action_emoji() {
        let action = Action {
            id: "test".to_string(),
            action_type: "test".to_string(),
            description: "test".to_string(),
            priority: "low".to_string(),
            command: "echo test".to_string(),
            reversible: false,
            revert_command: None,
            safe_to_autorun: true,
            created_at: 0,
            tags: vec!["cleanup".to_string()],
        };
        assert_eq!(action.emoji(), "🧹");

        let action_security = Action {
            tags: vec!["security".to_string()],
            ..action.clone()
        };
        assert_eq!(action_security.emoji(), "🔒");
    }

    #[test]
    fn test_action_serialization() {
        let action = Action {
            id: "test_action".to_string(),
            action_type: "cache_clean".to_string(),
            description: "Test action".to_string(),
            priority: "low".to_string(),
            command: "echo test".to_string(),
            reversible: true,
            revert_command: Some("echo revert".to_string()),
            safe_to_autorun: true,
            created_at: 1699000000,
            tags: vec!["cleanup".to_string()],
        };

        let json = serde_json::to_string(&action).unwrap();
        let parsed: Action = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, "test_action");
        assert_eq!(parsed.priority, "low");
        assert!(parsed.reversible);
    }

    #[test]
    fn test_action_result_serialization() {
        let result = ActionResult {
            action_id: "test_action".to_string(),
            executed_at: 1699000000,
            actor: "auto".to_string(),
            success: true,
            output: "Success".to_string(),
            exit_code: Some(0),
            duration_ms: 150,
        };

        let json = serde_json::to_string(&result).unwrap();
        let parsed: ActionResult = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.action_id, "test_action");
        assert!(parsed.success);
        assert_eq!(parsed.exit_code, Some(0));
    }

    #[test]
    fn test_safe_actions_creation() {
        let actions = create_safe_actions();

        assert!(!actions.is_empty());

        // All should have valid priorities
        for action in &actions {
            assert!(matches!(
                action.priority.as_str(),
                "low" | "medium" | "high" | "critical"
            ));
        }

        // At least one cleanup action
        assert!(actions.iter().any(|a| a.tags.contains(&"cleanup".to_string())));
    }
}
