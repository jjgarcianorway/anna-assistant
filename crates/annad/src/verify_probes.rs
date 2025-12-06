//! Verification probes for clarification answers.
//!
//! Safe, read-only probes to verify user clarifications against system reality.
//! No destructive commands - only existence checks.

use anna_shared::facts::{FactKey, FactsStore};
use anna_shared::intake::{VerifyPlan, VerificationResult};
use anna_shared::rpc::ProbeResult;
use std::process::Command;
use tracing::{info, warn};

/// Run a verification probe and return the result
pub fn run_verify_probe(plan: &VerifyPlan, user_answer: &str) -> VerificationResult {
    match plan {
        VerifyPlan::None => VerificationResult::success(user_answer.to_string(), "no_verification"),

        VerifyPlan::BinaryExists { binary } => {
            let target = if binary == "PLACEHOLDER" { user_answer } else { binary.as_str() };
            verify_binary_exists(target)
        }

        VerifyPlan::UnitExists { unit } => {
            let target = if unit == "PLACEHOLDER" { user_answer } else { unit.as_str() };
            verify_unit_exists(target)
        }

        VerifyPlan::MountExists { mount } => {
            let target = if mount == "PLACEHOLDER" { user_answer } else { mount.as_str() };
            verify_mount_exists(target)
        }

        VerifyPlan::InterfaceExists { iface } => {
            let target = if iface == "PLACEHOLDER" { user_answer } else { iface.as_str() };
            verify_interface_exists(target)
        }

        VerifyPlan::FileExists { path } => {
            let target = if path == "PLACEHOLDER" { user_answer } else { path.as_str() };
            verify_file_exists(target)
        }

        VerifyPlan::DirectoryExists { path } => {
            let target = if path == "PLACEHOLDER" { user_answer } else { path.as_str() };
            verify_directory_exists(target)
        }

        VerifyPlan::FromEvidence { key } => {
            // This is handled by checking existing probe evidence
            VerificationResult::success(user_answer.to_string(), &format!("evidence:{}", key))
        }
    }
}

/// Verify a binary exists using `command -v`
fn verify_binary_exists(binary: &str) -> VerificationResult {
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
            // Try to find alternatives
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

    // Editor alternatives
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
fn binary_exists(binary: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {}", binary))
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Verify a systemd unit exists
fn verify_unit_exists(unit: &str) -> VerificationResult {
    info!("Verifying unit exists: {}", unit);

    if !is_safe_name(unit) {
        return VerificationResult::failed("Invalid unit name", "verify_unit");
    }

    // Normalize unit name
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
fn verify_mount_exists(mount: &str) -> VerificationResult {
    info!("Verifying mount exists: {}", mount);

    let output = Command::new("df").arg("-h").output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mounts: Vec<String> = stdout
                .lines()
                .skip(1) // Skip header
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
fn verify_interface_exists(iface: &str) -> VerificationResult {
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

            // Handle "wifi" and "ethernet" as interface types
            let found = match iface.to_lowercase().as_str() {
                "wifi" | "wlan" => interfaces.iter().any(|i| i.starts_with("wlan") || i.starts_with("wlp")),
                "ethernet" | "eth" => interfaces.iter().any(|i| i.starts_with("eth") || i.starts_with("enp")),
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
fn verify_file_exists(path: &str) -> VerificationResult {
    info!("Verifying file exists: {}", path);

    // Basic path validation
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
fn verify_directory_exists(path: &str) -> VerificationResult {
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
fn is_safe_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() < 256
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '/')
}

/// Verify clarification answer and update facts store if successful
pub fn verify_and_store(
    plan: &VerifyPlan,
    user_answer: &str,
    fact_key: Option<&FactKey>,
    facts: &mut FactsStore,
) -> VerificationResult {
    let result = run_verify_probe(plan, user_answer);

    if result.verified {
        if let (Some(key), Some(ref value)) = (fact_key, &result.value) {
            // Store the verified fact
            facts.set_verified(key.clone(), value.clone(), result.source.clone());
            info!("Stored verified fact: {:?} = {}", key, value);

            // For binaries, also store the binary availability fact
            if let FactKey::PreferredEditor = key {
                let binary_key = FactKey::BinaryAvailable(user_answer.to_string());
                facts.set_verified(binary_key, value.clone(), result.source.clone());
            }
        }
    }

    result
}

/// Verify from existing probe evidence
pub fn verify_from_evidence(
    plan: &VerifyPlan,
    user_answer: &str,
    probe_results: &[ProbeResult],
) -> VerificationResult {
    match plan {
        VerifyPlan::FromEvidence { key } => {
            // Search probe results for matching evidence
            for probe in probe_results {
                if probe.exit_code == 0 {
                    // Check if answer matches evidence
                    let stdout_lower = probe.stdout.to_lowercase();
                    let answer_lower = user_answer.to_lowercase();

                    match key.as_str() {
                        "network_interfaces" => {
                            // For wifi/ethernet check
                            let has_wifi = stdout_lower.contains("wlan") || stdout_lower.contains("wlp");
                            let has_eth = stdout_lower.contains("eth") || stdout_lower.contains("enp");

                            match answer_lower.as_str() {
                                "wifi" if has_wifi => {
                                    return VerificationResult::success(
                                        user_answer.to_string(),
                                        "evidence:ip_link",
                                    );
                                }
                                "ethernet" if has_eth => {
                                    return VerificationResult::success(
                                        user_answer.to_string(),
                                        "evidence:ip_link",
                                    );
                                }
                                "both" if has_wifi && has_eth => {
                                    return VerificationResult::success(
                                        user_answer.to_string(),
                                        "evidence:ip_link",
                                    );
                                }
                                _ => {}
                            }
                        }
                        _ => {
                            // Generic contains check
                            if stdout_lower.contains(&answer_lower) {
                                return VerificationResult::success(
                                    user_answer.to_string(),
                                    &format!("evidence:{}", key),
                                );
                            }
                        }
                    }
                }
            }

            VerificationResult::failed(
                &format!("Could not verify {} from evidence", user_answer),
                "evidence_check",
            )
        }
        _ => VerificationResult::failed("Not an evidence verification plan", "invalid_plan"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_safe_name() {
        assert!(is_safe_name("vim"));
        assert!(is_safe_name("nginx.service"));
        assert!(is_safe_name("my-app"));
        assert!(is_safe_name("app_name"));
        assert!(!is_safe_name(""));
        assert!(!is_safe_name("cmd; rm -rf"));
        assert!(!is_safe_name("$(malicious)"));
    }

    #[test]
    fn test_verify_plan_none() {
        let result = run_verify_probe(&VerifyPlan::None, "anything");
        assert!(result.verified);
        assert_eq!(result.value, Some("anything".to_string()));
    }

    #[test]
    fn test_verification_result_success() {
        let result = VerificationResult::success("/usr/bin/vim".to_string(), "test");
        assert!(result.verified);
        assert!(result.alternatives.is_empty());
    }

    #[test]
    fn test_verification_result_failed_with_alternatives() {
        let result = VerificationResult::failed_with_alternatives(
            "not found",
            vec!["alt1".to_string(), "alt2".to_string()],
            "test",
        );
        assert!(!result.verified);
        assert_eq!(result.alternatives.len(), 2);
    }
}
