use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;
use tracing::{info, warn};

use crate::telemetry;

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
    pub fix_hint: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutoFixResult {
    pub check_name: String,
    pub attempted: bool,
    pub success: bool,
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

    // Core system checks (organized by category)
    checks.push(check_daemon_active());
    checks.push(check_socket_ready());
    checks.push(check_socket_permissions());
    checks.push(check_anna_group());
    checks.push(check_group_membership());

    // Path checks
    checks.push(check_config_dir());
    checks.push(check_state_dir());
    checks.push(check_runtime_dir());
    checks.push(check_paths_writable());

    // Permission checks
    checks.push(check_permissions());
    checks.push(check_config_permissions());
    checks.push(check_state_permissions());

    // Dependency checks
    checks.push(check_dependencies());
    checks.push(check_polkit_policies());
    checks.push(check_autocomplete_installed());

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

/// Check if daemon is active via systemd
fn check_daemon_active() -> DiagnosticCheck {
    let output = Command::new("systemctl")
        .args(&["is-active", "annad.service"])
        .output();

    match output {
        Ok(result) if result.status.success() => {
            DiagnosticCheck {
                name: "daemon_active".to_string(),
                status: Status::Pass,
                message: "Daemon service is active".to_string(),
                fix_hint: None,
            }
        }
        _ => DiagnosticCheck {
            name: "daemon_active".to_string(),
            status: Status::Fail,
            message: "Daemon service is not active".to_string(),
            fix_hint: Some("sudo systemctl start annad.service".to_string()),
        },
    }
}

/// Check if socket is ready
fn check_socket_ready() -> DiagnosticCheck {
    let socket_path = Path::new("/run/anna/annad.sock");
    if socket_path.exists() {
        // Check if it's actually a socket
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            if let Ok(metadata) = std::fs::metadata(socket_path) {
                if metadata.file_type().is_socket() {
                    return DiagnosticCheck {
                        name: "socket_ready".to_string(),
                        status: Status::Pass,
                        message: "/run/anna/annad.sock is available".to_string(),
                        fix_hint: None,
                    };
                }
            }
        }
    }

    DiagnosticCheck {
        name: "socket_ready".to_string(),
        status: Status::Fail,
        message: "/run/anna/annad.sock not found".to_string(),
        fix_hint: Some("sudo systemctl restart annad.service".to_string()),
    }
}

/// Check if polkit policies are present
fn check_polkit_policies() -> DiagnosticCheck {
    let policy_path = Path::new("/usr/share/polkit-1/actions/com.anna.policy");
    if policy_path.exists() {
        DiagnosticCheck {
            name: "polkit_policies_present".to_string(),
            status: Status::Pass,
            message: "Polkit policies installed".to_string(),
            fix_hint: None,
        }
    } else {
        DiagnosticCheck {
            name: "polkit_policies_present".to_string(),
            status: Status::Fail,
            message: "Polkit policies missing".to_string(),
            fix_hint: Some("Run: sudo cp polkit/com.anna.policy /usr/share/polkit-1/actions/".to_string()),
        }
    }
}

/// Check if required paths are writable
fn check_paths_writable() -> DiagnosticCheck {
    let paths = vec![
        ("/etc/anna", true),  // System path (needs root)
        ("$HOME/.config/anna", false),  // User path
        ("$HOME/.local/share/anna", false),  // User data path
    ];

    let mut missing = Vec::new();
    let mut failed_write = Vec::new();

    for (path_template, needs_root) in paths {
        let path_str = if path_template.contains("$HOME") {
            if let Ok(home) = std::env::var("HOME") {
                path_template.replace("$HOME", &home)
            } else {
                continue; // Skip if no HOME
            }
        } else {
            path_template.to_string()
        };

        let path = Path::new(&path_str);

        if !path.exists() {
            // Try to create
            if std::fs::create_dir_all(&path).is_err() {
                missing.push(path_str.clone());
            }
        } else if !needs_root {
            // Test writability for user paths
            let test_file = path.join(".write_test");
            if std::fs::write(&test_file, "test").is_err() {
                failed_write.push(path_str);
            } else {
                let _ = std::fs::remove_file(&test_file);
            }
        }
    }

    if missing.is_empty() && failed_write.is_empty() {
        DiagnosticCheck {
            name: "paths_writable".to_string(),
            status: Status::Pass,
            message: "All required paths accessible".to_string(),
            fix_hint: None,
        }
    } else if !failed_write.is_empty() {
        DiagnosticCheck {
            name: "paths_writable".to_string(),
            status: Status::Fail,
            message: format!("Cannot write to: {}", failed_write.join(", ")),
            fix_hint: Some("Check file permissions".to_string()),
        }
    } else {
        DiagnosticCheck {
            name: "paths_writable".to_string(),
            status: Status::Warn,
            message: format!("Created missing directories: {}", missing.join(", ")),
            fix_hint: None,
        }
    }
}

/// Check if bash completion is installed
fn check_autocomplete_installed() -> DiagnosticCheck {
    let completion_paths = vec![
        "/usr/share/bash-completion/completions/annactl",
        "/etc/bash_completion.d/annactl",
    ];

    for path in completion_paths {
        if Path::new(path).exists() {
            return DiagnosticCheck {
                name: "autocomplete_installed".to_string(),
                status: Status::Pass,
                message: "Bash completion installed".to_string(),
                fix_hint: None,
            };
        }
    }

    DiagnosticCheck {
        name: "autocomplete_installed".to_string(),
        status: Status::Warn,
        message: "Bash completion not found".to_string(),
        fix_hint: Some("Run: sudo cp completion/annactl.bash /usr/share/bash-completion/completions/annactl".to_string()),
    }
}

// Legacy checks below

fn check_config_dir() -> DiagnosticCheck {
    let config_dir = Path::new("/etc/anna");
    if config_dir.exists() && config_dir.is_dir() {
        DiagnosticCheck {
            name: "config_directory".to_string(),
            status: Status::Pass,
            message: "/etc/anna exists and is readable".to_string(),
            fix_hint: None,
        }
    } else {
        DiagnosticCheck {
            name: "config_directory".to_string(),
            status: Status::Fail,
            message: "/etc/anna does not exist or is not a directory".to_string(),
            fix_hint: Some("sudo mkdir -p /etc/anna".to_string()),
        }
    }
}

fn check_permissions() -> DiagnosticCheck {
    if nix::unistd::Uid::effective().is_root() {
        DiagnosticCheck {
            name: "daemon_permissions".to_string(),
            status: Status::Pass,
            message: "Running as root".to_string(),
            fix_hint: None,
        }
    } else {
        DiagnosticCheck {
            name: "daemon_permissions".to_string(),
            status: Status::Fail,
            message: "Not running as root".to_string(),
            fix_hint: Some("Start daemon via: sudo systemctl start annad".to_string()),
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
            name: "system_dependencies".to_string(),
            status: Status::Pass,
            message: "All required tools available".to_string(),
            fix_hint: None,
        }
    } else {
        DiagnosticCheck {
            name: "system_dependencies".to_string(),
            status: Status::Warn,
            message: format!("Missing tools: {}", missing.join(", ")),
            fix_hint: Some("Install missing tools via package manager".to_string()),
        }
    }
}

/// Check if anna group exists
fn check_anna_group() -> DiagnosticCheck {
    use nix::unistd::Group;
    match Group::from_name("anna") {
        Ok(Some(_)) => DiagnosticCheck {
            name: "anna_group".to_string(),
            status: Status::Pass,
            message: "Anna group exists".to_string(),
            fix_hint: None,
        },
        _ => DiagnosticCheck {
            name: "anna_group".to_string(),
            status: Status::Fail,
            message: "Anna group not found".to_string(),
            fix_hint: Some("sudo groupadd anna".to_string()),
        },
    }
}

/// Check if current user is in anna group
fn check_group_membership() -> DiagnosticCheck {
    // Get current user's groups
    let output = Command::new("groups").output();

    match output {
        Ok(result) if result.status.success() => {
            let groups_str = String::from_utf8_lossy(&result.stdout);
            if groups_str.contains("anna") {
                DiagnosticCheck {
                    name: "group_membership".to_string(),
                    status: Status::Pass,
                    message: "User is in anna group".to_string(),
                    fix_hint: None,
                }
            } else {
                DiagnosticCheck {
                    name: "group_membership".to_string(),
                    status: Status::Warn,
                    message: "User not in anna group".to_string(),
                    fix_hint: Some("sudo usermod -aG anna $USER && newgrp anna".to_string()),
                }
            }
        }
        _ => DiagnosticCheck {
            name: "group_membership".to_string(),
            status: Status::Warn,
            message: "Could not check group membership".to_string(),
            fix_hint: None,
        },
    }
}

/// Check socket permissions
fn check_socket_permissions() -> DiagnosticCheck {
    let socket_path = Path::new("/run/anna/annad.sock");
    if !socket_path.exists() {
        return DiagnosticCheck {
            name: "socket_permissions".to_string(),
            status: Status::Fail,
            message: "Socket does not exist".to_string(),
            fix_hint: Some("sudo systemctl restart annad".to_string()),
        };
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(socket_path) {
            let mode = metadata.permissions().mode();
            let perms = mode & 0o777;
            if perms == 0o660 || perms == 0o666 {
                return DiagnosticCheck {
                    name: "socket_permissions".to_string(),
                    status: Status::Pass,
                    message: format!("Socket permissions correct ({:o})", perms),
                    fix_hint: None,
                };
            } else {
                return DiagnosticCheck {
                    name: "socket_permissions".to_string(),
                    status: Status::Warn,
                    message: format!("Socket permissions incorrect ({:o}, expected 660)", perms),
                    fix_hint: Some("sudo systemctl restart annad".to_string()),
                };
            }
        }
    }

    DiagnosticCheck {
        name: "socket_permissions".to_string(),
        status: Status::Warn,
        message: "Could not check socket permissions".to_string(),
        fix_hint: None,
    }
}

/// Check state directory
fn check_state_dir() -> DiagnosticCheck {
    let state_dir = Path::new("/var/lib/anna");
    if state_dir.exists() && state_dir.is_dir() {
        DiagnosticCheck {
            name: "state_directory".to_string(),
            status: Status::Pass,
            message: "/var/lib/anna exists".to_string(),
            fix_hint: None,
        }
    } else {
        DiagnosticCheck {
            name: "state_directory".to_string(),
            status: Status::Fail,
            message: "/var/lib/anna missing".to_string(),
            fix_hint: Some("sudo mkdir -p /var/lib/anna && sudo chown root:anna /var/lib/anna && sudo chmod 0750 /var/lib/anna".to_string()),
        }
    }
}

/// Check runtime directory
fn check_runtime_dir() -> DiagnosticCheck {
    let runtime_dir = Path::new("/run/anna");
    if runtime_dir.exists() && runtime_dir.is_dir() {
        DiagnosticCheck {
            name: "runtime_directory".to_string(),
            status: Status::Pass,
            message: "/run/anna exists".to_string(),
            fix_hint: None,
        }
    } else {
        DiagnosticCheck {
            name: "runtime_directory".to_string(),
            status: Status::Fail,
            message: "/run/anna missing".to_string(),
            fix_hint: Some("sudo mkdir -p /run/anna && sudo chown root:anna /run/anna && sudo chmod 0770 /run/anna".to_string()),
        }
    }
}

/// Check config directory permissions
fn check_config_permissions() -> DiagnosticCheck {
    let config_dir = Path::new("/etc/anna");
    if !config_dir.exists() {
        return DiagnosticCheck {
            name: "config_permissions".to_string(),
            status: Status::Fail,
            message: "Config directory does not exist".to_string(),
            fix_hint: Some("sudo mkdir -p /etc/anna && sudo chown root:anna /etc/anna && sudo chmod 0750 /etc/anna".to_string()),
        };
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(config_dir) {
            let mode = metadata.permissions().mode();
            let perms = mode & 0o777;
            if perms == 0o750 {
                return DiagnosticCheck {
                    name: "config_permissions".to_string(),
                    status: Status::Pass,
                    message: "Config directory permissions correct (0750)".to_string(),
                    fix_hint: None,
                };
            } else {
                return DiagnosticCheck {
                    name: "config_permissions".to_string(),
                    status: Status::Warn,
                    message: format!("Config permissions incorrect ({:o}, expected 0750)", perms),
                    fix_hint: Some("sudo chmod 0750 /etc/anna && sudo chown root:anna /etc/anna".to_string()),
                };
            }
        }
    }

    DiagnosticCheck {
        name: "config_permissions".to_string(),
        status: Status::Warn,
        message: "Could not check config permissions".to_string(),
        fix_hint: None,
    }
}

/// Check state directory permissions
fn check_state_permissions() -> DiagnosticCheck {
    let state_dir = Path::new("/var/lib/anna");
    if !state_dir.exists() {
        return DiagnosticCheck {
            name: "state_permissions".to_string(),
            status: Status::Fail,
            message: "State directory does not exist".to_string(),
            fix_hint: Some("sudo mkdir -p /var/lib/anna && sudo chown root:anna /var/lib/anna && sudo chmod 0750 /var/lib/anna".to_string()),
        };
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(state_dir) {
            let mode = metadata.permissions().mode();
            let perms = mode & 0o777;
            if perms == 0o750 {
                return DiagnosticCheck {
                    name: "state_permissions".to_string(),
                    status: Status::Pass,
                    message: "State directory permissions correct (0750)".to_string(),
                    fix_hint: None,
                };
            } else {
                return DiagnosticCheck {
                    name: "state_permissions".to_string(),
                    status: Status::Warn,
                    message: format!("State permissions incorrect ({:o}, expected 0750)", perms),
                    fix_hint: Some("sudo chmod -R 0750 /var/lib/anna && sudo chown -R root:anna /var/lib/anna".to_string()),
                };
            }
        }
    }

    DiagnosticCheck {
        name: "state_permissions".to_string(),
        status: Status::Warn,
        message: "Could not check state permissions".to_string(),
        fix_hint: None,
    }
}

/// Run auto-fix for failed diagnostic checks
/// Only safe, low-risk fixes are attempted
pub async fn run_autofix() -> Vec<AutoFixResult> {
    let mut results = Vec::new();

    // Run diagnostics first to see what needs fixing
    let diag_results = run_diagnostics().await;

    for check in diag_results.checks {
        if check.status != Status::Fail {
            continue; // Only fix failures
        }

        let fix_result = match check.name.as_str() {
            "socket_ready" => autofix_socket_directory(),
            "paths_writable" => autofix_paths(),
            "config_directory" => autofix_config_directory(),
            "polkit_policies_present" => autofix_polkit_notice(),
            _ => AutoFixResult {
                check_name: check.name.clone(),
                attempted: false,
                success: false,
                message: "No auto-fix available for this check".to_string(),
            },
        };

        // Log auto-fix attempt
        let _ = telemetry::log_event(telemetry::Event::RpcCall {
            name: format!("autofix.{}", check.name),
            status: if fix_result.success {
                "success"
            } else {
                "failed"
            }
            .to_string(),
        });

        results.push(fix_result);
    }

    results
}

/// Auto-fix: Recreate socket directory
fn autofix_socket_directory() -> AutoFixResult {
    let socket_dir = Path::new("/run/anna");

    if !socket_dir.exists() {
        match fs::create_dir_all(socket_dir) {
            Ok(_) => {
                info!("Auto-fix: Created socket directory /run/anna");
                AutoFixResult {
                    check_name: "socket_ready".to_string(),
                    attempted: true,
                    success: true,
                    message: "Created /run/anna directory".to_string(),
                }
            }
            Err(e) => {
                warn!("Auto-fix failed to create socket directory: {}", e);
                AutoFixResult {
                    check_name: "socket_ready".to_string(),
                    attempted: true,
                    success: false,
                    message: format!("Failed to create directory: {}", e),
                }
            }
        }
    } else {
        AutoFixResult {
            check_name: "socket_ready".to_string(),
            attempted: false,
            success: false,
            message: "Directory exists but socket missing (daemon may need restart)".to_string(),
        }
    }
}

/// Auto-fix: Create required paths
fn autofix_paths() -> AutoFixResult {
    let paths = vec![
        Path::new("/etc/anna"),
        Path::new("/var/lib/anna"),
        Path::new("/var/lib/anna/events"),
        Path::new("/var/lib/anna/state"),
    ];

    let mut created = Vec::new();
    let mut failed = Vec::new();

    for path in paths {
        if !path.exists() {
            match fs::create_dir_all(path) {
                Ok(_) => created.push(path.display().to_string()),
                Err(e) => {
                    failed.push(format!("{}: {}", path.display(), e));
                }
            }
        }
    }

    if !failed.is_empty() {
        AutoFixResult {
            check_name: "paths_writable".to_string(),
            attempted: true,
            success: false,
            message: format!("Failed to create: {}", failed.join(", ")),
        }
    } else if !created.is_empty() {
        AutoFixResult {
            check_name: "paths_writable".to_string(),
            attempted: true,
            success: true,
            message: format!("Created paths: {}", created.join(", ")),
        }
    } else {
        AutoFixResult {
            check_name: "paths_writable".to_string(),
            attempted: false,
            success: false,
            message: "All paths exist".to_string(),
        }
    }
}

/// Auto-fix: Create config directory
fn autofix_config_directory() -> AutoFixResult {
    let config_dir = Path::new("/etc/anna");

    if !config_dir.exists() {
        match fs::create_dir_all(config_dir) {
            Ok(_) => AutoFixResult {
                check_name: "config_directory".to_string(),
                attempted: true,
                success: true,
                message: "Created /etc/anna directory".to_string(),
            },
            Err(e) => AutoFixResult {
                check_name: "config_directory".to_string(),
                attempted: true,
                success: false,
                message: format!("Failed to create directory: {}", e),
            },
        }
    } else {
        AutoFixResult {
            check_name: "config_directory".to_string(),
            attempted: false,
            success: false,
            message: "Directory already exists".to_string(),
        }
    }
}

/// Auto-fix: Polkit policy (cannot auto-install, just provide notice)
fn autofix_polkit_notice() -> AutoFixResult {
    AutoFixResult {
        check_name: "polkit_policies_present".to_string(),
        attempted: false,
        success: false,
        message: "Cannot auto-install polkit policy. Run installer or: sudo cp polkit/com.anna.policy /usr/share/polkit-1/actions/".to_string(),
    }
}
