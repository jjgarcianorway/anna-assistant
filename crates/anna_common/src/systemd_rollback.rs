//! Systemd Service Rollback v0.0.51
//!
//! Rollback systemd service actions to previous state.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::service_state::{ActiveState, EnabledState};
use crate::systemd_probe::{probe_service, ServiceProbe};

/// Rollback directory base
pub const ROLLBACK_BASE: &str = "/var/lib/anna/rollback";

// =============================================================================
// Rollback Result
// =============================================================================

/// Result from rolling back a service action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRollbackResult {
    pub success: bool,
    pub case_id: String,
    pub service: String,
    pub restored_active: ActiveState,
    pub restored_enabled: EnabledState,
    pub original_active: ActiveState,
    pub original_enabled: EnabledState,
    pub message: String,
    pub error: Option<String>,
}

/// Execute rollback for a service action case
pub fn rollback_service_action(case_id: &str) -> Result<ServiceRollbackResult, String> {
    let rollback_dir = PathBuf::from(ROLLBACK_BASE).join(case_id);

    if !rollback_dir.exists() {
        return Err(format!("No rollback data found for case: {}", case_id));
    }

    // Read rollback metadata
    let metadata_path = rollback_dir.join("service_rollback.json");
    let metadata_content = fs::read_to_string(&metadata_path)
        .map_err(|e| format!("Cannot read rollback metadata: {}", e))?;

    let metadata: serde_json::Value = serde_json::from_str(&metadata_content)
        .map_err(|e| format!("Cannot parse rollback metadata: {}", e))?;

    let service = metadata["service"]
        .as_str()
        .ok_or("Missing service in rollback metadata")?;
    let original_active = metadata["pre_state"]["active_state"]
        .as_str()
        .ok_or("Missing original active state")?;
    let original_enabled = metadata["pre_state"]["enabled_state"]
        .as_str()
        .ok_or("Missing original enabled state")?;

    let original_active_state = ActiveState::parse(original_active);
    let original_enabled_state = EnabledState::parse(original_enabled);

    // Probe current state
    let current = probe_service(service);

    // Execute restore operations
    execute_restore_operations(
        service,
        &current,
        &original_active_state,
        &original_enabled_state,
    )?;

    // Verify restored state
    let restored = probe_service(service);

    let message = if current.active_state == original_active_state
        && current.enabled_state == original_enabled_state
    {
        "No changes needed - service already in original state".to_string()
    } else {
        format!(
            "Restored {} to original state (active={}, enabled={})",
            service,
            restored.active_state.as_str(),
            restored.enabled_state.as_str()
        )
    };

    Ok(ServiceRollbackResult {
        success: true,
        case_id: case_id.to_string(),
        service: service.to_string(),
        restored_active: restored.active_state,
        restored_enabled: restored.enabled_state,
        original_active: original_active_state,
        original_enabled: original_enabled_state,
        message,
        error: None,
    })
}

fn execute_restore_operations(
    service: &str,
    current: &ServiceProbe,
    original_active: &ActiveState,
    original_enabled: &EnabledState,
) -> Result<(), String> {
    // Restore active state if different
    if current.active_state != *original_active {
        let op = match original_active {
            ActiveState::Active => "start",
            ActiveState::Inactive => "stop",
            _ => return Ok(()), // Can't restore other states
        };

        let output = Command::new("systemctl")
            .args([op, service])
            .output()
            .map_err(|e| format!("Failed to execute systemctl {}: {}", op, e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "Rollback failed at systemctl {}: {}",
                op,
                stderr.trim()
            ));
        }
    }

    // Restore enabled state if different
    if current.enabled_state != *original_enabled {
        let op = match original_enabled {
            EnabledState::Enabled => "enable",
            EnabledState::Disabled => "disable",
            _ => return Ok(()), // Can't restore other states
        };

        let output = Command::new("systemctl")
            .args([op, service])
            .output()
            .map_err(|e| format!("Failed to execute systemctl {}: {}", op, e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "Rollback failed at systemctl {}: {}",
                op,
                stderr.trim()
            ));
        }
    }

    Ok(())
}

/// Generate a case ID for service actions
pub fn generate_service_case_id() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("svc_{:x}", ts)
}
