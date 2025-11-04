//! Action Executor - Safely executes system commands with rollback support

use anna_common::{Action, Advice, AuditEntry, Config, RollbackToken};
use anyhow::{Context, Result};
use chrono::Utc;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{error, info, warn};

use crate::snapshotter::{Snapshotter, Snapshot};

/// Execute an action based on advice with snapshot support
pub async fn execute_action_with_snapshot(
    advice: &Advice,
    dry_run: bool,
    config: Option<&Config>,
) -> Result<(Action, Option<Snapshot>)> {
    let action_id = format!("action_{}", uuid::Uuid::new_v4());

    info!("Executing action: {} (dry_run={})", advice.id, dry_run);

    let command = match &advice.command {
        Some(cmd) => cmd,
        None => {
            return Err(anyhow::anyhow!("No command specified for advice {}", advice.id));
        }
    };

    if dry_run {
        let action = Action {
            id: action_id,
            advice_id: advice.id.clone(),
            command: command.clone(),
            executed_at: Utc::now(),
            success: true,
            output: format!("[DRY RUN] Would execute: {}", command),
            error: None,
        };
        return Ok((action, None));
    }

    // Create snapshot if configured and risk level requires it
    let mut snapshot = None;
    if let Some(cfg) = config {
        let snapshotter = Snapshotter::new(cfg.clone());
        if snapshotter.should_snapshot_for_risk(advice.risk) {
            info!("Creating snapshot before executing {} (risk: {:?})", advice.id, advice.risk);
            match snapshotter.create_snapshot(&format!("Before: {}", advice.title)).await {
                Ok(snap) => {
                    info!("Snapshot created: {}", snap.id);
                    snapshot = Some(snap);
                }
                Err(e) => {
                    warn!("Failed to create snapshot: {}. Proceeding anyway.", e);
                }
            }
        }
    }

    // Execute the command
    let result = execute_command(command).await;

    let action = match result {
        Ok(output) => {
            info!("Action {} completed successfully", action_id);
            Action {
                id: action_id,
                advice_id: advice.id.clone(),
                command: command.clone(),
                executed_at: Utc::now(),
                success: true,
                output,
                error: None,
            }
        }
        Err(e) => {
            error!("Action {} failed: {}", action_id, e);
            Action {
                id: action_id,
                advice_id: advice.id.clone(),
                command: command.clone(),
                executed_at: Utc::now(),
                success: false,
                output: String::new(),
                error: Some(e.to_string()),
            }
        }
    };

    Ok((action, snapshot))
}

/// Execute an action based on advice (backward compatibility)
pub async fn execute_action(advice: &Advice, dry_run: bool) -> Result<Action> {
    let (action, _snapshot) = execute_action_with_snapshot(advice, dry_run, None).await?;
    Ok(action)
}

/// Execute a shell command safely
async fn execute_command(command: &str) -> Result<String> {
    // Parse command into program and args
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Err(anyhow::anyhow!("Empty command"));
    }

    let program = parts[0];
    let args = &parts[1..];

    info!("Executing: {} {:?}", program, args);

    let output = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to execute command")?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(anyhow::anyhow!("Command failed: {}", stderr))
    }
}

/// Create a rollback token for an action with snapshot info
#[allow(dead_code)]
pub fn create_rollback_token(
    action: &Action,
    rollback_cmd: Option<String>,
    snapshot: Option<&Snapshot>,
) -> RollbackToken {
    RollbackToken {
        action_id: action.id.clone(),
        advice_id: action.advice_id.clone(),
        executed_at: action.executed_at,
        command: action.command.clone(),
        rollback_command: rollback_cmd,
        snapshot_before: snapshot.map(|s| s.id.clone()),
    }
}

/// Create an audit log entry for an action
pub fn create_audit_entry(action: &Action, actor: &str) -> AuditEntry {
    AuditEntry {
        timestamp: Utc::now(),
        actor: actor.to_string(),
        action_type: "execute_action".to_string(),
        details: format!(
            "Executed command '{}' from advice {}",
            action.command, action.advice_id
        ),
        success: action.success,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_common::RiskLevel;

    #[tokio::test]
    async fn test_dry_run() {
        let advice = Advice {
            id: "test-1".to_string(),
            title: "Test".to_string(),
            reason: "Test".to_string(),
            action: "Test action".to_string(),
            command: Some("echo hello".to_string()),
            risk: RiskLevel::Low,
            wiki_refs: vec![],
        };

        let action = execute_action(&advice, true).await.unwrap();
        assert!(action.success);
        assert!(action.output.contains("DRY RUN"));
    }

    #[tokio::test]
    async fn test_execute_echo() {
        let advice = Advice {
            id: "test-2".to_string(),
            title: "Test".to_string(),
            reason: "Test".to_string(),
            action: "Test action".to_string(),
            command: Some("echo hello".to_string()),
            risk: RiskLevel::Low,
            wiki_refs: vec![],
        };

        let action = execute_action(&advice, false).await.unwrap();
        assert!(action.success);
        assert!(action.output.contains("hello"));
    }
}
