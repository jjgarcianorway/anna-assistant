//! Systemd service state detection for annad.

use std::process::Command;

/// Systemd service state
#[derive(Debug, PartialEq)]
pub enum ServiceState {
    Active,
    Inactive,
    Failed,
    NotFound,
    Unknown,
}

/// Query the systemd service state for annad
pub fn get_service_state() -> ServiceState {
    let output = Command::new("systemctl")
        .args(["is-active", "annad"])
        .output();

    match output {
        Ok(o) => {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            match s.as_str() {
                "active" | "activating" => ServiceState::Active,
                "inactive" | "deactivating" => ServiceState::Inactive,
                "failed" => ServiceState::Failed,
                "not-found" | "" => {
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    if stderr.contains("not-found") || stderr.contains("not found") {
                        ServiceState::NotFound
                    } else {
                        ServiceState::Unknown
                    }
                }
                _ => ServiceState::Unknown,
            }
        }
        Err(_) => ServiceState::Unknown,
    }
}
