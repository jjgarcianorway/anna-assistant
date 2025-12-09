//! Verification runners (v0.0.198).

use std::process::Command;

use super::types::{ServiceExpectedState, VerificationStep, VerifyExpectation, VerifyResult};

/// Run a verification step
pub fn run_verification(step: &VerificationStep) -> VerifyResult {
    match &step.expectation {
        VerifyExpectation::CommandExists { name } => verify_command_exists(&step.id, name),
        VerifyExpectation::ExitCode { command, expected } => {
            verify_exit_code(&step.id, command, *expected)
        }
        VerifyExpectation::FileExists { path } => verify_file_exists(&step.id, path),
        VerifyExpectation::FileContainsLine { path, pattern } => {
            verify_file_contains(&step.id, path, pattern)
        }
        VerifyExpectation::PackageInstalled { package } => {
            verify_package_installed(&step.id, package)
        }
        VerifyExpectation::ServiceState { service, expected } => {
            verify_service_state(&step.id, service, *expected)
        }
        VerifyExpectation::OutputContains { command, pattern } => {
            verify_output_contains(&step.id, command, pattern)
        }
    }
}

fn verify_command_exists(step_id: &str, name: &str) -> VerifyResult {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {}", name))
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            VerifyResult::pass(step_id, path)
        }
        Ok(_) => VerifyResult::fail(step_id, "not found", format!("{} is not installed", name)),
        Err(e) => VerifyResult::fail(step_id, "", format!("Failed to check: {}", e)),
    }
}

fn verify_exit_code(step_id: &str, command: &str, expected: i32) -> VerifyResult {
    let output = Command::new("sh").arg("-c").arg(command).output();

    match output {
        Ok(out) => {
            let code = out.status.code().unwrap_or(-1);
            if code == expected {
                VerifyResult::pass(step_id, format!("exit code {}", code))
            } else {
                VerifyResult::fail(
                    step_id,
                    format!("exit code {}", code),
                    format!("Expected exit code {}, got {}", expected, code),
                )
            }
        }
        Err(e) => VerifyResult::fail(step_id, "", format!("Failed to run: {}", e)),
    }
}

fn verify_file_exists(step_id: &str, path: &str) -> VerifyResult {
    // Expand ~ to home directory
    let expanded = expand_path(path);
    if std::path::Path::new(&expanded).exists() {
        VerifyResult::pass(step_id, expanded)
    } else {
        VerifyResult::fail(
            step_id,
            "not found",
            format!("File {} does not exist", path),
        )
    }
}

fn verify_file_contains(step_id: &str, path: &str, pattern: &str) -> VerifyResult {
    let expanded = expand_path(path);
    match std::fs::read_to_string(&expanded) {
        Ok(content) => {
            if content.contains(pattern) {
                VerifyResult::pass(step_id, "pattern found")
            } else {
                VerifyResult::fail(
                    step_id,
                    "pattern not found",
                    format!("File does not contain: {}", pattern),
                )
            }
        }
        Err(e) => VerifyResult::fail(step_id, "", format!("Cannot read file: {}", e)),
    }
}

fn verify_package_installed(step_id: &str, package: &str) -> VerifyResult {
    // Try pacman first (Arch)
    let output = Command::new("pacman").arg("-Q").arg(package).output();

    match output {
        Ok(out) if out.status.success() => {
            let info = String::from_utf8_lossy(&out.stdout).trim().to_string();
            VerifyResult::pass(step_id, info)
        }
        Ok(_) => VerifyResult::fail(
            step_id,
            "not installed",
            format!("Package {} is not installed", package),
        ),
        Err(_) => {
            // Try dpkg (Debian/Ubuntu)
            let dpkg = Command::new("dpkg").arg("-s").arg(package).output();
            match dpkg {
                Ok(out) if out.status.success() => VerifyResult::pass(step_id, "installed (dpkg)"),
                _ => VerifyResult::fail(step_id, "unknown", "Cannot determine package status"),
            }
        }
    }
}

fn verify_service_state(
    step_id: &str,
    service: &str,
    expected: ServiceExpectedState,
) -> VerifyResult {
    let check_cmd = match expected {
        ServiceExpectedState::Active | ServiceExpectedState::Inactive => "is-active",
        ServiceExpectedState::Enabled | ServiceExpectedState::Disabled => "is-enabled",
    };

    let output = Command::new("systemctl")
        .arg(check_cmd)
        .arg(service)
        .output();

    match output {
        Ok(out) => {
            let state = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let matches = match expected {
                ServiceExpectedState::Active => state == "active",
                ServiceExpectedState::Inactive => state == "inactive",
                ServiceExpectedState::Enabled => state == "enabled",
                ServiceExpectedState::Disabled => state == "disabled",
            };
            if matches {
                VerifyResult::pass(step_id, state)
            } else {
                VerifyResult::fail(
                    step_id,
                    &state,
                    format!("Expected {}, got {}", expected, state),
                )
            }
        }
        Err(e) => VerifyResult::fail(step_id, "", format!("Failed to check: {}", e)),
    }
}

fn verify_output_contains(step_id: &str, command: &str, pattern: &str) -> VerifyResult {
    let output = Command::new("sh").arg("-c").arg(command).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if stdout.contains(pattern) {
                VerifyResult::pass(step_id, "pattern found in output")
            } else {
                VerifyResult::fail(
                    step_id,
                    "pattern not found",
                    format!("Output does not contain: {}", pattern),
                )
            }
        }
        Err(e) => VerifyResult::fail(step_id, "", format!("Failed to run: {}", e)),
    }
}

/// Expand ~ to home directory
pub fn expand_path(path: &str) -> String {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{}{}", home, &path[1..]);
        }
    }
    path.to_string()
}
