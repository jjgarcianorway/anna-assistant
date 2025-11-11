//! Action Executor - Safely executes system commands with rollback support

use anna_common::{Action, Advice, AuditEntry, Config, RollbackToken};
use anyhow::{Context, Result};
use chrono::Utc;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{error, info, warn};

use crate::snapshotter::{Snapshotter, Snapshot};

/// Validates command for suspicious patterns before execution
/// SECURITY: Commands come from daemon's trusted whitelist, but this provides defense-in-depth
fn validate_command(command: &str) -> Result<()> {
    // Check for extremely dangerous patterns
    // FIXED (Beta.107): Previous regexes were broken (e.g., "curl.*|.*sh" matched ANY string with "sh")
    let dangerous_patterns = [
        r"^rm\s+-rf\s+/$",           // Exact: rm -rf / (recursive delete of root)
        r"^rm\s+-rf\s+/\s",          // rm -rf / with more args
        r"mkfs\.",                   // Filesystem creation (destructive)
        r"dd\s+if=/dev",             // Direct disk device read
        r"dd\s+of=/dev",             // Direct disk device write
        r"curl.*\|\s*(ba)?sh",       // Download and pipe to shell
        r"wget.*\|\s*(ba)?sh",       // Download and pipe to shell
        r":\(\)\s*\{\s*:\|:&\s*\};:", // Fork bomb pattern
    ];

    for pattern in &dangerous_patterns {
        if regex::Regex::new(pattern).unwrap().is_match(command) {
            warn!("SECURITY: Blocked dangerous command pattern: {}", pattern);
            return Err(anyhow::anyhow!(
                "Command contains dangerous pattern and was blocked for safety: {}", pattern
            ));
        }
    }

    // Log if command uses shell features (for audit)
    if command.contains("&&") || command.contains("||") || command.contains("|")
        || command.contains("$()") || command.contains("`") || command.contains(">")
        || command.contains("<") {
        info!("SECURITY: Executing command with shell features: {}", command);
    }

    Ok(())
}

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
        // Generate rollback command even for dry run (Beta.89)
        let (rollback_command, can_rollback, rollback_unavailable_reason) =
            anna_common::rollback::generate_rollback_command(command);

        let action = Action {
            id: action_id,
            advice_id: advice.id.clone(),
            command: command.clone(),
            executed_at: Utc::now(),
            success: true,
            output: format!("[DRY RUN] Would execute: {}", command),
            error: None,
            rollback_command,
            can_rollback,
            rollback_unavailable_reason,
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

    // Generate rollback command (Beta.89)
    let (rollback_command, can_rollback, rollback_unavailable_reason) =
        anna_common::rollback::generate_rollback_command(command);

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
                rollback_command: rollback_command.clone(),
                can_rollback,
                rollback_unavailable_reason: rollback_unavailable_reason.clone(),
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
                rollback_command,
                can_rollback,
                rollback_unavailable_reason,
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
pub async fn execute_command(command: &str) -> Result<String> {
    if command.trim().is_empty() {
        return Err(anyhow::anyhow!("Empty command"));
    }

    // SECURITY: Validate command before execution
    validate_command(command)?;

    info!("Executing shell command: {}", command);

    // Execute through shell to support complex syntax like $(...), &&, |, etc.
    // Beta.107: Added better error reporting for permission issues
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context(format!("Failed to execute command: {}", command))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        error!("Command failed with exit code {:?}: {}", output.status.code(), stderr);

        // Check for common errors
        if stderr.contains("Permission denied") || stderr.contains("EACCES") {
            return Err(anyhow::anyhow!(
                "Permission denied. Daemon is running as UID={}, command: {}\nError: {}",
                nix::unistd::getuid(),
                command,
                stderr
            ));
        }

        Err(anyhow::anyhow!("Command failed: {}\nStdout: {}", stderr, stdout))
    }
}

/// Streaming chunk for real-time command output
#[derive(Debug, Clone)]
pub enum ExecutionChunk {
    Stdout(String),
    Stderr(String),
    Status(String),
}

/// Execute a command with streaming output via channel
pub async fn execute_command_streaming_channel(
    command: &str,
) -> Result<(tokio::sync::mpsc::Receiver<ExecutionChunk>, tokio::task::JoinHandle<Result<bool>>)> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::sync::mpsc;

    if command.trim().is_empty() {
        return Err(anyhow::anyhow!("Empty command"));
    }

    // SECURITY: Validate command before execution
    validate_command(command)?;

    info!("Executing shell command with streaming: {}", command);

    let (tx, rx) = mpsc::channel(100); // Buffer up to 100 chunks

    // Send initial status
    let _ = tx.send(ExecutionChunk::Status(format!("Executing: {}", command))).await;

    let command = command.to_string();

    // Spawn task to execute command and stream output
    let handle = tokio::spawn(async move {
        // Spawn process with pipes
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(&command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn command")?;

        let stdout = child.stdout.take().context("Failed to capture stdout")?;
        let stderr = child.stderr.take().context("Failed to capture stderr")?;

        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        // Read stdout and stderr concurrently
        let mut stdout_done = false;
        let mut stderr_done = false;

        while !stdout_done || !stderr_done {
            tokio::select! {
                result = stdout_reader.next_line(), if !stdout_done => {
                    match result {
                        Ok(Some(line)) => {
                            let _ = tx.send(ExecutionChunk::Stdout(line)).await;
                        }
                        Ok(None) => stdout_done = true,
                        Err(e) => {
                            error!("Error reading stdout: {}", e);
                            stdout_done = true;
                        }
                    }
                }
                result = stderr_reader.next_line(), if !stderr_done => {
                    match result {
                        Ok(Some(line)) => {
                            let _ = tx.send(ExecutionChunk::Stderr(line)).await;
                        }
                        Ok(None) => stderr_done = true,
                        Err(e) => {
                            error!("Error reading stderr: {}", e);
                            stderr_done = true;
                        }
                    }
                }
            }
        }

        // Wait for process to complete
        let status = child.wait().await.context("Failed to wait for command")?;

        Ok(status.success())
    });

    Ok((rx, handle))
}

/// Execute a command with streaming output (callback version - deprecated, use channel version)
#[allow(dead_code)]
pub async fn execute_command_streaming<F>(
    command: &str,
    mut output_callback: F,
) -> Result<bool>
where
    F: FnMut(ExecutionChunk),
{
    use tokio::io::{AsyncBufReadExt, BufReader};

    if command.trim().is_empty() {
        return Err(anyhow::anyhow!("Empty command"));
    }

    // SECURITY: Validate command before execution
    validate_command(command)?;

    info!("Executing shell command with streaming: {}", command);
    output_callback(ExecutionChunk::Status(format!("Executing: {}", command)));

    // Spawn process with pipes
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn command")?;

    let stdout = child.stdout.take().context("Failed to capture stdout")?;
    let stderr = child.stderr.take().context("Failed to capture stderr")?;

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    // Read stdout and stderr concurrently
    let mut stdout_done = false;
    let mut stderr_done = false;

    while !stdout_done || !stderr_done {
        tokio::select! {
            result = stdout_reader.next_line(), if !stdout_done => {
                match result {
                    Ok(Some(line)) => {
                        output_callback(ExecutionChunk::Stdout(line));
                    }
                    Ok(None) => stdout_done = true,
                    Err(e) => {
                        error!("Error reading stdout: {}", e);
                        stdout_done = true;
                    }
                }
            }
            result = stderr_reader.next_line(), if !stderr_done => {
                match result {
                    Ok(Some(line)) => {
                        output_callback(ExecutionChunk::Stderr(line));
                    }
                    Ok(None) => stderr_done = true,
                    Err(e) => {
                        error!("Error reading stderr: {}", e);
                        stderr_done = true;
                    }
                }
            }
        }
    }

    // Wait for process to complete
    let status = child.wait().await.context("Failed to wait for command")?;

    Ok(status.success())
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
        action_type: "apply_action".to_string(),
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
    use anna_common::{Priority, RiskLevel};

    #[tokio::test]
    async fn test_dry_run() {
        let advice = Advice {
            id: "test-1".to_string(),
            title: "Test".to_string(),
            reason: "Test".to_string(),
            action: "Test action".to_string(),
            command: Some("echo hello".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            wiki_refs: vec![],
            category: "test".to_string(),
            alternatives: vec![],
            depends_on: vec![],
            related_to: vec![],
            bundle: None,
            satisfies: vec![],
            popularity: 50,
            requires: vec![],
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
            priority: Priority::Optional,
            wiki_refs: vec![],
            category: "test".to_string(),
            alternatives: vec![],
            depends_on: vec![],
            related_to: vec![],
            bundle: None,
            satisfies: vec![],
            popularity: 50,
            requires: vec![],
        };

        let action = execute_action(&advice, false).await.unwrap();
        assert!(action.success);
        assert!(action.output.contains("hello"));
    }
}
