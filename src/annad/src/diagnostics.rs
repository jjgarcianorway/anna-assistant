use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosticResults {
    pub checks: Vec<DiagnosticCheck>,
    pub overall_status: Status,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosticCheck {
    pub name: String,
    pub status: Status,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pass,
    Warn,
    Fail,
}

pub async fn run_diagnostics() -> DiagnosticResults {
    let mut checks = Vec::new();

    // Check 1: Config directory exists
    checks.push(check_config_dir());

    // Check 2: Socket is running
    checks.push(check_socket());

    // Check 3: Permissions
    checks.push(check_permissions());

    // Check 4: System dependencies
    checks.push(check_dependencies());

    // Determine overall status
    let overall_status = if checks.iter().any(|c| c.status == Status::Fail) {
        Status::Fail
    } else if checks.iter().any(|c| c.status == Status::Warn) {
        Status::Warn
    } else {
        Status::Pass
    };

    DiagnosticResults {
        checks,
        overall_status,
    }
}

fn check_config_dir() -> DiagnosticCheck {
    let config_dir = Path::new("/etc/anna");
    if config_dir.exists() && config_dir.is_dir() {
        DiagnosticCheck {
            name: "Config Directory".to_string(),
            status: Status::Pass,
            message: "/etc/anna exists and is readable".to_string(),
        }
    } else {
        DiagnosticCheck {
            name: "Config Directory".to_string(),
            status: Status::Fail,
            message: "/etc/anna does not exist or is not a directory".to_string(),
        }
    }
}

fn check_socket() -> DiagnosticCheck {
    let socket_path = Path::new("/run/anna.sock");
    if socket_path.exists() {
        DiagnosticCheck {
            name: "Unix Socket".to_string(),
            status: Status::Pass,
            message: "/run/anna.sock is available".to_string(),
        }
    } else {
        DiagnosticCheck {
            name: "Unix Socket".to_string(),
            status: Status::Warn,
            message: "/run/anna.sock not found (daemon may be starting)".to_string(),
        }
    }
}

fn check_permissions() -> DiagnosticCheck {
    if nix::unistd::Uid::effective().is_root() {
        DiagnosticCheck {
            name: "Daemon Permissions".to_string(),
            status: Status::Pass,
            message: "Running as root".to_string(),
        }
    } else {
        DiagnosticCheck {
            name: "Daemon Permissions".to_string(),
            status: Status::Fail,
            message: "Not running as root".to_string(),
        }
    }
}

fn check_dependencies() -> DiagnosticCheck {
    // Check for essential system tools
    let tools = ["bash", "systemctl"];
    let mut missing = Vec::new();

    for tool in &tools {
        if which::which(tool).is_err() {
            missing.push(*tool);
        }
    }

    if missing.is_empty() {
        DiagnosticCheck {
            name: "System Dependencies".to_string(),
            status: Status::Pass,
            message: "All required tools available".to_string(),
        }
    } else {
        DiagnosticCheck {
            name: "System Dependencies".to_string(),
            status: Status::Warn,
            message: format!("Missing tools: {}", missing.join(", ")),
        }
    }
}
