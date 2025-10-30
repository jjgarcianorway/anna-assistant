use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pass,
    Warn,
    Fail,
}

pub async fn run_diagnostics() -> DiagnosticResults {
    let mut checks = Vec::new();

    // Required checks for Sprint 1
    checks.push(check_daemon_active());
    checks.push(check_socket_ready());
    checks.push(check_polkit_policies());
    checks.push(check_paths_writable());
    checks.push(check_autocomplete_installed());

    // Legacy checks
    checks.push(check_config_dir());
    checks.push(check_permissions());
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
