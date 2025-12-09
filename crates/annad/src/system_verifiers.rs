//! System verification functions for checking existence of system resources.
//!
//! Extracted from verify_probes.rs (v0.0.160) for modularization.
//! Safe, read-only probes - no destructive commands.

use anna_shared::intake::VerificationResult;
use std::process::Command;
use tracing::{info, warn};

/// Verify a binary exists using `command -v`
pub fn verify_binary_exists(binary: &str) -> VerificationResult {
    info!("Verifying binary exists: {}", binary);

    // Sanitize input - only allow alphanumeric and common chars
    if !is_safe_name(binary) {
        return VerificationResult::failed("Invalid binary name", "verify_binary");
    }

    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {}", binary))
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            info!("Binary {} found at: {}", binary, path);
            VerificationResult::success(path, &format!("probe:command -v {}", binary))
        }
        _ => {
            warn!("Binary {} not found", binary);
            let alternatives = find_binary_alternatives(binary);
            if alternatives.is_empty() {
                VerificationResult::failed(
                    &format!("{} not found on this system", binary),
                    "probe:command -v",
                )
            } else {
                VerificationResult::failed_with_alternatives(
                    &format!("{} not found, but alternatives exist", binary),
                    alternatives,
                    "probe:command -v",
                )
            }
        }
    }
}

/// Find alternatives for a missing binary
fn find_binary_alternatives(binary: &str) -> Vec<String> {
    let mut alternatives = Vec::new();

    let editor_map: &[(&str, &[&str])] = &[
        ("vim", &["nvim", "vi", "nano"]),
        ("nvim", &["vim", "vi", "nano"]),
        ("vi", &["vim", "nvim", "nano"]),
        ("nano", &["vim", "vi", "pico"]),
        ("emacs", &["vim", "nano"]),
    ];

    for (key, alts) in editor_map {
        if *key == binary {
            for alt in *alts {
                if binary_exists(alt) {
                    alternatives.push(alt.to_string());
                }
            }
            break;
        }
    }

    alternatives
}

/// Quick check if a binary exists
pub fn binary_exists(binary: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {}", binary))
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Verify a systemd unit exists
pub fn verify_unit_exists(unit: &str) -> VerificationResult {
    info!("Verifying unit exists: {}", unit);

    if !is_safe_name(unit) {
        return VerificationResult::failed("Invalid unit name", "verify_unit");
    }

    let unit_name = if unit.ends_with(".service") {
        unit.to_string()
    } else {
        format!("{}.service", unit)
    };

    let output = Command::new("systemctl")
        .arg("list-unit-files")
        .arg(&unit_name)
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if stdout.contains(&unit_name) {
                info!("Unit {} found", unit_name);
                VerificationResult::success(unit_name, "probe:systemctl list-unit-files")
            } else {
                warn!("Unit {} not found in list", unit_name);
                VerificationResult::failed(
                    &format!("{} not found as a systemd unit", unit),
                    "probe:systemctl list-unit-files",
                )
            }
        }
        _ => VerificationResult::failed(
            &format!("Could not verify unit {}", unit),
            "probe:systemctl",
        ),
    }
}

/// Verify a mount point exists
pub fn verify_mount_exists(mount: &str) -> VerificationResult {
    info!("Verifying mount exists: {}", mount);

    let output = Command::new("df").arg("-h").output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mounts: Vec<String> = stdout
                .lines()
                .skip(1)
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    parts.last().map(|s| s.to_string())
                })
                .collect();

            if mounts.iter().any(|m| m == mount || m.starts_with(mount)) {
                info!("Mount {} found", mount);
                VerificationResult::success(mount.to_string(), "probe:df")
            } else {
                warn!("Mount {} not found", mount);
                VerificationResult::failed_with_alternatives(
                    &format!("{} not found as a mount point", mount),
                    mounts,
                    "probe:df",
                )
            }
        }
        _ => VerificationResult::failed(&format!("Could not verify mount {}", mount), "probe:df"),
    }
}

/// Verify a network interface exists
pub fn verify_interface_exists(iface: &str) -> VerificationResult {
    info!("Verifying interface exists: {}", iface);

    let output = Command::new("ip").arg("link").arg("show").output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let interfaces: Vec<String> = stdout
                .lines()
                .filter(|line| line.contains(": ") && !line.starts_with(' '))
                .filter_map(|line| {
                    line.split(':')
                        .nth(1)
                        .map(|s| s.trim().split('@').next().unwrap_or("").to_string())
                })
                .filter(|s| !s.is_empty())
                .collect();

            let found = match iface.to_lowercase().as_str() {
                "wifi" | "wlan" => interfaces
                    .iter()
                    .any(|i| i.starts_with("wlan") || i.starts_with("wlp")),
                "ethernet" | "eth" => interfaces
                    .iter()
                    .any(|i| i.starts_with("eth") || i.starts_with("enp")),
                _ => interfaces.iter().any(|i| i == iface),
            };

            if found {
                info!("Interface {} found", iface);
                VerificationResult::success(iface.to_string(), "probe:ip link")
            } else {
                warn!("Interface {} not found", iface);
                VerificationResult::failed_with_alternatives(
                    &format!("{} not found as a network interface", iface),
                    interfaces,
                    "probe:ip link",
                )
            }
        }
        _ => VerificationResult::failed(
            &format!("Could not verify interface {}", iface),
            "probe:ip link",
        ),
    }
}

/// Verify a file exists
pub fn verify_file_exists(path: &str) -> VerificationResult {
    info!("Verifying file exists: {}", path);

    if path.contains("..") || path.contains('\0') {
        return VerificationResult::failed("Invalid path", "verify_file");
    }

    let output = Command::new("test").arg("-f").arg(path).status();

    match output {
        Ok(status) if status.success() => {
            info!("File {} exists", path);
            VerificationResult::success(path.to_string(), "probe:test -f")
        }
        _ => {
            warn!("File {} not found", path);
            VerificationResult::failed(&format!("{} does not exist", path), "probe:test -f")
        }
    }
}

/// Verify a directory exists
pub fn verify_directory_exists(path: &str) -> VerificationResult {
    info!("Verifying directory exists: {}", path);

    if path.contains("..") || path.contains('\0') {
        return VerificationResult::failed("Invalid path", "verify_dir");
    }

    let output = Command::new("test").arg("-d").arg(path).status();

    match output {
        Ok(status) if status.success() => {
            info!("Directory {} exists", path);
            VerificationResult::success(path.to_string(), "probe:test -d")
        }
        _ => {
            warn!("Directory {} not found", path);
            VerificationResult::failed(&format!("{} does not exist", path), "probe:test -d")
        }
    }
}

/// Check if a name is safe for use in commands
pub fn is_safe_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() < 256
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '/')
}
