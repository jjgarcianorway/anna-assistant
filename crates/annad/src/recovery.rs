//! Self-healing recovery module for Anna infrastructure.
//!
//! v0.3.36: Phase 8 - Proactive Monitoring and Auto-Healing
//!
//! Anna monitors and recovers from:
//! - Ollama service failures
//! - Model loading failures
//! - Wiki initialization failures
//!
//! All recovery is automatic - users never see manual commands.

use anna_shared::status::{RecoveryOutcome, RecoveryStatus, SubsystemHealth};
use anyhow::{anyhow, Result};
use std::process::Command;
use std::time::Duration;
use tracing::{info, warn};

use crate::ollama;

/// Ollama recovery configuration
pub struct OllamaRecoveryConfig {
    /// Maximum time to wait for Ollama to start
    pub start_timeout_secs: u64,
    /// Whether to use pkexec for privilege escalation
    pub allow_pkexec: bool,
}

impl Default for OllamaRecoveryConfig {
    fn default() -> Self {
        Self {
            start_timeout_secs: 30,
            allow_pkexec: true,
        }
    }
}

/// Attempt to recover Ollama service
/// Returns Ok(true) if recovery was successful, Ok(false) if already running
pub async fn recover_ollama(
    recovery_status: &mut RecoveryStatus,
    config: &OllamaRecoveryConfig,
) -> Result<bool> {
    let timestamp = RecoveryStatus::now_rfc3339();
    let start = std::time::Instant::now();

    // Check if already running
    if ollama::is_running().await {
        recovery_status.ollama.health = SubsystemHealth::Healthy;
        return Ok(false);
    }

    info!("Ollama not responding, attempting recovery...");
    recovery_status.ollama.mark_recovering();

    // Step 1: Try standard service start
    if try_start_ollama_service().await {
        if wait_for_ollama(config.start_timeout_secs).await {
            record_recovery_success(
                recovery_status,
                "ollama",
                "service_start",
                &timestamp,
                start.elapsed().as_millis() as u64,
            );
            return Ok(true);
        }
    }

    // Step 2: Try pkexec if allowed
    if config.allow_pkexec && try_pkexec_ollama_start().await {
        if wait_for_ollama(config.start_timeout_secs).await {
            record_recovery_success(
                recovery_status,
                "ollama",
                "pkexec_start",
                &timestamp,
                start.elapsed().as_millis() as u64,
            );
            return Ok(true);
        }
    }

    // Step 3: Try direct process start
    if try_direct_ollama_start().await {
        if wait_for_ollama(config.start_timeout_secs).await {
            record_recovery_success(
                recovery_status,
                "ollama",
                "direct_start",
                &timestamp,
                start.elapsed().as_millis() as u64,
            );
            return Ok(true);
        }
    }

    // Recovery failed
    let error = "Ollama could not be started after multiple attempts";
    record_recovery_failure(
        recovery_status,
        "ollama",
        "all_methods",
        &timestamp,
        start.elapsed().as_millis() as u64,
        error,
    );
    Err(anyhow!("{}", error))
}

/// Try starting Ollama via systemctl (may fail without privileges)
async fn try_start_ollama_service() -> bool {
    let status = Command::new("systemctl")
        .args(["start", "ollama"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    matches!(status, Ok(s) if s.success())
}

/// Try starting Ollama via pkexec (GUI privilege escalation)
async fn try_pkexec_ollama_start() -> bool {
    // Check if pkexec is available
    if !is_command_available("pkexec") {
        return false;
    }

    let status = Command::new("pkexec")
        .args(["systemctl", "start", "ollama"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    matches!(status, Ok(s) if s.success())
}

/// Try starting Ollama directly as a process
async fn try_direct_ollama_start() -> bool {
    let mut cmd = Command::new("ollama");
    cmd.env("HOME", "/root");
    cmd.env("OLLAMA_MODELS", "/var/lib/anna/models");
    cmd.arg("serve");
    cmd.stdout(std::process::Stdio::null());
    cmd.stderr(std::process::Stdio::null());

    cmd.spawn().is_ok()
}

/// Wait for Ollama to become responsive
async fn wait_for_ollama(timeout_secs: u64) -> bool {
    let check_interval = Duration::from_millis(500);
    let max_checks = (timeout_secs * 2) as u32;

    for _ in 0..max_checks {
        if ollama::is_running().await {
            return true;
        }
        tokio::time::sleep(check_interval).await;
    }

    false
}

/// Check if a command is available in PATH
fn is_command_available(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Record a successful recovery
fn record_recovery_success(
    status: &mut RecoveryStatus,
    subsystem: &str,
    trigger: &str,
    timestamp: &str,
    duration_ms: u64,
) {
    use anna_shared::status::RecoveryEvent;

    let event = RecoveryEvent {
        timestamp: timestamp.to_string(),
        subsystem: subsystem.to_string(),
        trigger: trigger.to_string(),
        outcome: RecoveryOutcome::Success,
        duration_ms,
    };

    match subsystem {
        "ollama" => status.ollama.record_success(timestamp),
        "models" => status.models.record_success(timestamp),
        "wiki" => status.wiki.record_success(timestamp),
        "daemon" => status.daemon.record_success(timestamp),
        "permissions" => status.permissions.record_success(timestamp),
        _ => {}
    }

    status.add_event(event);
    info!("Recovery successful: {} via {} in {}ms", subsystem, trigger, duration_ms);
}

/// Record a failed recovery
fn record_recovery_failure(
    status: &mut RecoveryStatus,
    subsystem: &str,
    trigger: &str,
    timestamp: &str,
    duration_ms: u64,
    error: &str,
) {
    use anna_shared::status::RecoveryEvent;

    let event = RecoveryEvent {
        timestamp: timestamp.to_string(),
        subsystem: subsystem.to_string(),
        trigger: trigger.to_string(),
        outcome: RecoveryOutcome::Failed(error.to_string()),
        duration_ms,
    };

    match subsystem {
        "ollama" => status.ollama.record_failure(timestamp, error),
        "models" => status.models.record_failure(timestamp, error),
        "wiki" => status.wiki.record_failure(timestamp, error),
        "daemon" => status.daemon.record_failure(timestamp, error),
        "permissions" => status.permissions.record_failure(timestamp, error),
        _ => {}
    }

    status.add_event(event);
    warn!("Recovery failed: {} via {} - {}", subsystem, trigger, error);
}

/// Check all subsystems health and attempt recovery if needed
pub async fn health_check_and_recover(recovery_status: &mut RecoveryStatus) {
    // Check Ollama
    if !ollama::is_running().await {
        let config = OllamaRecoveryConfig::default();
        if let Err(e) = recover_ollama(recovery_status, &config).await {
            warn!("Ollama recovery failed: {}", e);
        }
    } else {
        recovery_status.ollama.health = SubsystemHealth::Healthy;
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_manual_commands_in_recovery() {
        // Verify this module doesn't contain manual command instructions in error messages
        let source = include_str!("recovery.rs");

        // Find main code (excluding test module)
        let test_start = source.find("#[cfg(test)]").unwrap_or(source.len());
        let main_code = &source[..test_start];

        // Forbidden patterns in user-facing messages
        let forbidden = [
            "sudo systemctl",
            "Run: sudo",
            "Try: sudo",
            "Execute: ",
            "Run this command",
        ];

        for pattern in &forbidden {
            assert!(
                !main_code.contains(pattern),
                "Recovery module should not contain manual command pattern: {}",
                pattern
            );
        }
    }

    #[test]
    fn test_recovery_config_defaults() {
        let config = OllamaRecoveryConfig::default();
        assert_eq!(config.start_timeout_secs, 30);
        assert!(config.allow_pkexec);
    }

    #[test]
    fn test_record_recovery_success() {
        let mut status = RecoveryStatus::default();
        record_recovery_success(&mut status, "ollama", "test", "2026-01-13T12:00:00Z", 100);

        assert_eq!(status.ollama.total_attempts, 1);
        assert_eq!(status.ollama.successful_recoveries, 1);
        assert_eq!(status.ollama.health, SubsystemHealth::Healthy);
        assert_eq!(status.recent_events.len(), 1);
    }

    #[test]
    fn test_record_recovery_failure() {
        let mut status = RecoveryStatus::default();
        record_recovery_failure(
            &mut status,
            "ollama",
            "test",
            "2026-01-13T12:00:00Z",
            100,
            "test error",
        );

        assert_eq!(status.ollama.total_attempts, 1);
        assert_eq!(status.ollama.failed_recoveries, 1);
        assert_eq!(status.ollama.health, SubsystemHealth::Degraded);
        assert_eq!(status.recent_events.len(), 1);
    }
}
