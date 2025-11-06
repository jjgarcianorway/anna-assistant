//! Action History - Store all executed actions for rollback support (Beta.91)
//!
//! Stores full Action objects in a JSONL file, enabling rollback functionality
//! by preserving rollback commands and action metadata.

use anna_common::Action;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::fs::{create_dir_all, OpenOptions};
use tokio::io::AsyncWriteExt;
use tracing::{info, warn};

const ACTION_HISTORY_DIR: &str = "/var/log/anna";
const ACTION_HISTORY_FILE: &str = "action_history.jsonl";

/// Action history manager for rollback support
pub struct ActionHistory {
    log_path: PathBuf,
}

impl ActionHistory {
    /// Create a new action history manager
    pub async fn new() -> Result<Self> {
        let history_dir = Path::new(ACTION_HISTORY_DIR);
        create_dir_all(history_dir)
            .await
            .context("Failed to create action history directory")?;

        let log_path = history_dir.join(ACTION_HISTORY_FILE);

        info!("Action history initialized: {}", log_path.display());

        Ok(Self { log_path })
    }

    /// Save an action to history
    pub async fn save(&self, action: &Action) -> Result<()> {
        let json = serde_json::to_string(action)? + "\n";

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .await
            .context("Failed to open action history")?;

        file.write_all(json.as_bytes())
            .await
            .context("Failed to write action")?;

        file.sync_all()
            .await
            .context("Failed to sync action history")?;

        Ok(())
    }

    /// Read all actions from history
    pub async fn read_all(&self) -> Result<Vec<Action>> {
        if !self.log_path.exists() {
            return Ok(vec![]);
        }

        let content = tokio::fs::read_to_string(&self.log_path)
            .await
            .context("Failed to read action history")?;

        let actions: Vec<Action> = content
            .lines()
            .filter(|line| !line.is_empty())
            .filter_map(|line| {
                match serde_json::from_str(line) {
                    Ok(action) => Some(action),
                    Err(e) => {
                        warn!("Failed to parse action from history: {}", e);
                        None
                    }
                }
            })
            .collect();

        Ok(actions)
    }

    /// Get all rollbackable actions (successful actions with rollback commands)
    pub async fn get_rollbackable_actions(&self) -> Result<Vec<Action>> {
        let actions = self.read_all().await?;

        let rollbackable: Vec<Action> = actions
            .into_iter()
            .filter(|action| action.success && action.can_rollback)
            .collect();

        Ok(rollbackable)
    }

    /// Get a specific action by advice ID (most recent if multiple)
    pub async fn get_by_advice_id(&self, advice_id: &str) -> Result<Option<Action>> {
        let actions = self.read_all().await?;

        // Find most recent action for this advice_id
        let action = actions
            .into_iter()
            .filter(|a| a.advice_id == advice_id && a.success)
            .max_by_key(|a| a.executed_at);

        Ok(action)
    }

    /// Get last N successful actions
    pub async fn get_last_n_actions(&self, n: usize) -> Result<Vec<Action>> {
        let mut actions = self.read_all().await?;

        // Filter successful actions only
        actions.retain(|a| a.success);

        // Sort by execution time (most recent first)
        actions.sort_by(|a, b| b.executed_at.cmp(&a.executed_at));

        // Take first N
        actions.truncate(n);

        Ok(actions)
    }

    /// Get the path to the action history file
    #[allow(dead_code)]
    pub fn path(&self) -> &Path {
        &self.log_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_common::RiskLevel;
    use chrono::Utc;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_action_history() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test_history.jsonl");

        let history = ActionHistory {
            log_path: log_path.clone(),
        };

        let action = Action {
            id: "test-1".to_string(),
            advice_id: "mangohud".to_string(),
            command: "pacman -S --noconfirm mangohud".to_string(),
            executed_at: Utc::now(),
            success: true,
            output: "Package installed".to_string(),
            error: None,
            rollback_command: Some("pacman -Rns --noconfirm mangohud".to_string()),
            can_rollback: true,
            rollback_unavailable_reason: None,
        };

        history.save(&action).await.unwrap();

        let actions = history.read_all().await.unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].advice_id, "mangohud");

        let rollbackable = history.get_rollbackable_actions().await.unwrap();
        assert_eq!(rollbackable.len(), 1);
    }

    #[tokio::test]
    async fn test_get_by_advice_id() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test_history2.jsonl");

        let history = ActionHistory {
            log_path: log_path.clone(),
        };

        // Add two actions with same advice_id
        for i in 0..2 {
            let action = Action {
                id: format!("test-{}", i),
                advice_id: "test-advice".to_string(),
                command: "echo test".to_string(),
                executed_at: Utc::now(),
                success: true,
                output: "ok".to_string(),
                error: None,
                rollback_command: Some("echo rollback".to_string()),
                can_rollback: true,
                rollback_unavailable_reason: None,
            };
            history.save(&action).await.unwrap();
        }

        let action = history.get_by_advice_id("test-advice").await.unwrap();
        assert!(action.is_some());
        // Should get the most recent one (second one added)
        assert_eq!(action.unwrap().id, "test-1");
    }
}
