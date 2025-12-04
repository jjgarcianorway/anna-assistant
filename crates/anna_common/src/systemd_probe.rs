//! Systemd Service Probing v0.0.51
//!
//! Probe systemd service state for evidence collection.

use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::generate_request_id;
use crate::service_state::{ActiveState, EnabledState};
use crate::systemd_action::normalize_service_name;

/// Max status output lines to capture
pub const MAX_STATUS_LINES: usize = 30;

// =============================================================================
// Probe Result
// =============================================================================

/// Result from probing a systemd service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceProbe {
    pub evidence_id: String,
    pub service: String,
    pub exists: bool,
    pub unit_file_path: Option<String>,
    pub active_state: ActiveState,
    pub enabled_state: EnabledState,
    pub description: Option<String>,
    pub main_pid: Option<u32>,
    pub last_failure: Option<String>,
    pub status_output: Option<String>,
    pub timestamp: u64,
}

/// Probe a systemd service for its current state
pub fn probe_service(service: &str) -> ServiceProbe {
    let evidence_id = format!("SVC{}", generate_request_id().chars().take(8).collect::<String>());
    let service = normalize_service_name(service);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Check if service exists
    let exists_output = Command::new("systemctl")
        .args(["list-unit-files", "--type=service", &service])
        .output();

    let exists = exists_output
        .as_ref()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains(&service))
        .unwrap_or(false);

    if !exists {
        // Also check if it's a loaded unit (might not be in unit-files)
        let show_output = Command::new("systemctl")
            .args(["show", &service, "--property=LoadState"])
            .output();

        let loaded = show_output
            .as_ref()
            .map(|o| {
                let s = String::from_utf8_lossy(&o.stdout);
                s.contains("LoadState=loaded")
            })
            .unwrap_or(false);

        if !loaded {
            return ServiceProbe {
                evidence_id,
                service,
                exists: false,
                unit_file_path: None,
                active_state: ActiveState::Unknown,
                enabled_state: EnabledState::Unknown,
                description: None,
                main_pid: None,
                last_failure: None,
                status_output: None,
                timestamp,
            };
        }
    }

    // Get detailed properties
    let (active_state, enabled_state, description, main_pid, unit_file_path, last_failure) =
        get_service_properties(&service);

    // Get bounded status output
    let status_output = get_service_status(&service);

    ServiceProbe {
        evidence_id,
        service,
        exists: true,
        unit_file_path,
        active_state,
        enabled_state,
        description,
        main_pid,
        last_failure,
        status_output,
        timestamp,
    }
}

/// Get service properties via systemctl show
fn get_service_properties(
    service: &str,
) -> (ActiveState, EnabledState, Option<String>, Option<u32>, Option<String>, Option<String>) {
    let props_output = Command::new("systemctl")
        .args([
            "show", service,
            "--property=ActiveState,SubState,UnitFileState,MainPID,Description,FragmentPath,Result",
        ])
        .output();

    let mut active_state = ActiveState::Unknown;
    let mut enabled_state = EnabledState::Unknown;
    let mut description = None;
    let mut main_pid = None;
    let mut unit_file_path = None;
    let mut last_failure = None;

    if let Ok(output) = props_output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Some((key, value)) = line.split_once('=') {
                match key {
                    "ActiveState" => active_state = ActiveState::parse(value),
                    "UnitFileState" => enabled_state = EnabledState::parse(value),
                    "MainPID" => main_pid = value.parse().ok().filter(|&p| p > 0),
                    "Description" => {
                        if !value.is_empty() {
                            description = Some(value.to_string());
                        }
                    }
                    "FragmentPath" => {
                        if !value.is_empty() {
                            unit_file_path = Some(value.to_string());
                        }
                    }
                    "Result" => {
                        if value != "success" && !value.is_empty() {
                            last_failure = Some(value.to_string());
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    (active_state, enabled_state, description, main_pid, unit_file_path, last_failure)
}

/// Get bounded service status output
fn get_service_status(service: &str) -> Option<String> {
    Command::new("systemctl")
        .args(["status", service, "--no-pager", "-n", &MAX_STATUS_LINES.to_string()])
        .output()
        .ok()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            // Limit to reasonable size
            stdout.chars().take(2000).collect::<String>()
        })
}
