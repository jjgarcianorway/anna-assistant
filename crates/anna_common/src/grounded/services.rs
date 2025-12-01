//! Service Module v6.0 - Grounded in systemctl
//!
//! Source of truth: systemctl commands only
//! No invented data. No hallucinations.

use std::process::Command;

/// A systemd service
#[derive(Debug, Clone)]
pub struct Service {
    pub name: String,
    pub state: ServiceState,
    pub enabled: EnabledState,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceState {
    Active,
    Inactive,
    Failed,
    Unknown,
}

impl ServiceState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ServiceState::Active => "active",
            ServiceState::Inactive => "inactive",
            ServiceState::Failed => "failed",
            ServiceState::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnabledState {
    Enabled,
    Disabled,
    Static,
    Masked,
    Unknown,
}

impl EnabledState {
    pub fn as_str(&self) -> &'static str {
        match self {
            EnabledState::Enabled => "enabled",
            EnabledState::Disabled => "disabled",
            EnabledState::Static => "static",
            EnabledState::Masked => "masked",
            EnabledState::Unknown => "unknown",
        }
    }
}

/// Service counts - all from real systemctl queries
#[derive(Debug, Clone, Default)]
pub struct ServiceCounts {
    pub total: usize,
    pub running: usize,
    pub failed: usize,
    pub enabled: usize,
}

impl ServiceCounts {
    /// Get real counts from systemctl
    pub fn query() -> Self {
        Self {
            total: count_service_units(),
            running: count_running_services(),
            failed: count_failed_services(),
            enabled: count_enabled_services(),
        }
    }
}

/// Count total service unit files
/// Source: systemctl list-unit-files --type=service
fn count_service_units() -> usize {
    let output = Command::new("systemctl")
        .args(["list-unit-files", "--type=service", "--no-legend"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter(|line| !line.trim().is_empty())
                .count()
        }
        _ => 0,
    }
}

/// Count running services
/// Source: systemctl list-units --type=service --state=active
fn count_running_services() -> usize {
    let output = Command::new("systemctl")
        .args(["list-units", "--type=service", "--state=active", "--no-legend"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter(|line| !line.trim().is_empty())
                .count()
        }
        _ => 0,
    }
}

/// Count failed services
/// Source: systemctl --failed --type=service
fn count_failed_services() -> usize {
    let output = Command::new("systemctl")
        .args(["--failed", "--type=service", "--no-legend"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter(|line| !line.trim().is_empty())
                .count()
        }
        _ => 0,
    }
}

/// Count enabled services
/// Source: systemctl list-unit-files --type=service --state=enabled
fn count_enabled_services() -> usize {
    let output = Command::new("systemctl")
        .args(["list-unit-files", "--type=service", "--state=enabled", "--no-legend"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter(|line| !line.trim().is_empty())
                .count()
        }
        _ => 0,
    }
}

/// Get service info
/// Sources: systemctl is-active, systemctl is-enabled, systemctl show
pub fn get_service_info(name: &str) -> Option<Service> {
    // Ensure it's a valid service unit name
    let unit_name = if name.ends_with(".service") {
        name.to_string()
    } else {
        format!("{}.service", name)
    };

    // Check if unit exists
    let exists = Command::new("systemctl")
        .args(["cat", &unit_name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !exists {
        return None;
    }

    let state = get_service_state(&unit_name);
    let enabled = get_enabled_state(&unit_name);
    let description = get_service_description(&unit_name);

    Some(Service {
        name: unit_name,
        state,
        enabled,
        description,
    })
}

/// Get service active state
/// Source: systemctl is-active <unit>
fn get_service_state(unit: &str) -> ServiceState {
    let output = Command::new("systemctl")
        .args(["is-active", unit])
        .output();

    match output {
        Ok(out) => {
            let state = String::from_utf8_lossy(&out.stdout).trim().to_string();
            match state.as_str() {
                "active" => ServiceState::Active,
                "inactive" => ServiceState::Inactive,
                "failed" => ServiceState::Failed,
                _ => ServiceState::Unknown,
            }
        }
        _ => ServiceState::Unknown,
    }
}

/// Get service enabled state
/// Source: systemctl is-enabled <unit>
fn get_enabled_state(unit: &str) -> EnabledState {
    let output = Command::new("systemctl")
        .args(["is-enabled", unit])
        .output();

    match output {
        Ok(out) => {
            let state = String::from_utf8_lossy(&out.stdout).trim().to_string();
            match state.as_str() {
                "enabled" => EnabledState::Enabled,
                "disabled" => EnabledState::Disabled,
                "static" => EnabledState::Static,
                "masked" => EnabledState::Masked,
                _ => EnabledState::Unknown,
            }
        }
        _ => EnabledState::Unknown,
    }
}

/// Get service description
/// Source: systemctl show -p Description <unit>
fn get_service_description(unit: &str) -> String {
    let output = Command::new("systemctl")
        .args(["show", "-p", "Description", "--value", unit])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let desc = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if desc.is_empty() {
                String::new()
            } else {
                format!("{} (source: systemctl show)", desc)
            }
        }
        _ => String::new(),
    }
}

/// List failed services
/// Source: systemctl --failed --type=service
pub fn list_failed_services() -> Vec<String> {
    let output = Command::new("systemctl")
        .args(["--failed", "--type=service", "--no-legend", "--plain"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter_map(|line| {
                    line.split_whitespace().next().map(|s| s.to_string())
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

/// List all service unit files
/// Source: systemctl list-unit-files --type=service
pub fn list_service_units() -> Vec<(String, String)> {
    let output = Command::new("systemctl")
        .args(["list-unit-files", "--type=service", "--no-legend"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        Some((parts[0].to_string(), parts[1].to_string()))
                    } else {
                        None
                    }
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_counts() {
        let counts = ServiceCounts::query();
        // Any systemd system should have services
        assert!(counts.total > 0);
    }

    #[test]
    fn test_service_state() {
        // These should exist on any systemd system
        let info = get_service_info("systemd-journald");
        assert!(info.is_some());
        if let Some(svc) = info {
            assert_eq!(svc.state, ServiceState::Active);
        }
    }
}
