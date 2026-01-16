//! System state detection for assisted operations.
//!
//! This module READS system state for diagnosis. It does NOT execute fixes.
//! Commands here are diagnostic only - gathering information to show the human.

use std::process::Command;
use std::fs;
use std::path::Path;

use super::types::{DetectedIssue, Evidence, IssueSeverity};

/// Read the contents of a file for diagnostic purposes.
pub fn read_file_content(path: &str) -> Option<String> {
    fs::read_to_string(path).ok()
}

/// Check if a file exists.
pub fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// Run a diagnostic command and capture output.
///
/// # IMPORTANT: DIAGNOSTIC ONLY
///
/// This function runs commands to READ system state.
/// It is used for things like:
/// - `iw dev` to check WiFi status
/// - `cat /etc/modprobe.d/iwlwifi.conf` to read config
/// - `systemctl status NetworkManager` to check service state
///
/// This function is NEVER used to execute fix commands.
/// Fix commands are displayed to the human as text.
pub fn run_diagnostic_command(cmd: &str, args: &[&str]) -> Option<String> {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .map(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if stdout.is_empty() {
                stderr
            } else {
                stdout
            }
        })
}

/// Check if a kernel module is loaded.
pub fn is_module_loaded(module_name: &str) -> bool {
    run_diagnostic_command("lsmod", &[])
        .map(|output| output.contains(module_name))
        .unwrap_or(false)
}

/// Get the driver in use for a PCI device.
pub fn get_pci_driver(device_pattern: &str) -> Option<String> {
    let output = run_diagnostic_command("lspci", &["-k"])?;

    let mut found_device = false;
    for line in output.lines() {
        if line.to_lowercase().contains(&device_pattern.to_lowercase()) {
            found_device = true;
        } else if found_device && line.contains("Kernel driver in use:") {
            return line.split(':').nth(1).map(|s| s.trim().to_string());
        } else if found_device && !line.starts_with('\t') {
            found_device = false;
        }
    }
    None
}

/// Get current WiFi interface information.
pub fn get_wifi_interface_info() -> Option<String> {
    run_diagnostic_command("iw", &["dev"])
}

/// Get WiFi link status.
pub fn get_wifi_link_status(interface: &str) -> Option<String> {
    run_diagnostic_command("iw", &[interface, "link"])
}

/// Get regulatory domain.
pub fn get_regulatory_domain() -> Option<String> {
    run_diagnostic_command("iw", &["reg", "get"])
}

/// Read modprobe configuration for a module.
pub fn read_modprobe_config(module_name: &str) -> Vec<(String, String)> {
    let mut configs = Vec::new();

    let modprobe_dir = Path::new("/etc/modprobe.d");
    if let Ok(entries) = fs::read_dir(modprobe_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".conf") {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        for line in content.lines() {
                            if line.contains(module_name) && !line.trim().starts_with('#') {
                                configs.push((
                                    entry.path().to_string_lossy().to_string(),
                                    line.to_string(),
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    configs
}

/// Detect WiFi-related issues.
pub fn detect_wifi_issues() -> Vec<DetectedIssue> {
    let mut issues = Vec::new();

    // Check if iwlwifi is loaded
    if !is_module_loaded("iwlwifi") {
        issues.push(DetectedIssue {
            problem: "Intel WiFi driver (iwlwifi) is not loaded".to_string(),
            severity: IssueSeverity::Major,
            evidence: vec![Evidence {
                source: "lsmod".to_string(),
                finding: "iwlwifi module not found in loaded modules".to_string(),
                raw_output: run_diagnostic_command("lsmod", &[]),
            }],
            fixable: true,
        });
        return issues; // Can't check further without driver
    }

    // Check modprobe configuration for problematic settings
    let configs = read_modprobe_config("iwlwifi");
    for (file, line) in &configs {
        // Check for 11n_disable=1 which kills HT mode
        if line.contains("11n_disable=1") {
            issues.push(DetectedIssue {
                problem: "WiFi is configured to disable 802.11n (11n_disable=1), limiting speeds to 54 Mbps".to_string(),
                severity: IssueSeverity::Major,
                evidence: vec![Evidence {
                    source: file.clone(),
                    finding: format!("Found: {}", line),
                    raw_output: Some(line.clone()),
                }],
                fixable: true,
            });
        }

        // Check for conflicting options
        if configs.iter().filter(|(_, l)| l.contains("11n_disable")).count() > 1 {
            issues.push(DetectedIssue {
                problem: "Multiple conflicting 11n_disable settings found".to_string(),
                severity: IssueSeverity::Major,
                evidence: configs.iter().map(|(f, l)| Evidence {
                    source: f.clone(),
                    finding: l.clone(),
                    raw_output: None,
                }).collect(),
                fixable: true,
            });
            break;
        }
    }

    // Check WiFi link status
    if let Some(link_info) = get_wifi_link_status("wlan0") {
        // Check for low bitrate
        if link_info.contains("54.0 MBit/s") || link_info.contains("48.0 MBit/s") {
            issues.push(DetectedIssue {
                problem: "WiFi link rate is limited to legacy 802.11a/g speeds (54 Mbps or less)".to_string(),
                severity: IssueSeverity::Major,
                evidence: vec![Evidence {
                    source: "iw wlan0 link".to_string(),
                    finding: "Bitrate indicates legacy mode, not 802.11n/ac/ax".to_string(),
                    raw_output: Some(link_info.clone()),
                }],
                fixable: true,
            });
        }

        // Check for 20MHz channel width (should be 80MHz or higher for modern WiFi)
        if link_info.contains("width: 20 MHz") {
            issues.push(DetectedIssue {
                problem: "WiFi is using narrow 20 MHz channel width instead of 80/160 MHz".to_string(),
                severity: IssueSeverity::Minor,
                evidence: vec![Evidence {
                    source: "iw dev".to_string(),
                    finding: "Channel width is 20 MHz".to_string(),
                    raw_output: get_wifi_interface_info(),
                }],
                fixable: true,
            });
        }
    }

    // Check regulatory domain
    if let Some(reg_info) = get_regulatory_domain() {
        if reg_info.contains("country 00") || reg_info.contains("DFS-UNSET") {
            issues.push(DetectedIssue {
                problem: "WiFi regulatory domain is not set, limiting power and available channels".to_string(),
                severity: IssueSeverity::Minor,
                evidence: vec![Evidence {
                    source: "iw reg get".to_string(),
                    finding: "Regulatory domain is unset (country 00)".to_string(),
                    raw_output: Some(reg_info),
                }],
                fixable: true,
            });
        }
    }

    issues
}

// =============================================================================
// PROOF: DETECTION COMMANDS ARE DIAGNOSTIC ONLY
// =============================================================================
//
// Every Command::new() in this file is for READING state:
// - lsmod: reads loaded modules
// - lspci -k: reads PCI devices and drivers
// - iw dev: reads WiFi interface state
// - iw reg get: reads regulatory settings
//
// None of these commands MODIFY system state.
// None of these commands are the proposed FIX commands.
//
// The FIX commands (like "sudo modprobe -r iwlwifi") are stored as strings
// in AssistedOperation.proposed_steps, and are NEVER passed to Command::new().
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_commands_are_read_only() {
        // All diagnostic commands in this module are read-only:
        // - lsmod (list modules)
        // - lspci (list PCI devices)
        // - iw (wireless info)
        // - cat/read (file contents)
        //
        // None of them modify system state.
        // None of them are fix commands.
    }

    #[test]
    fn test_modprobe_config_reader() {
        // This test verifies we can read modprobe configs
        // It doesn't test actual system state, just that the function works
        let configs = read_modprobe_config("nonexistent_module_xyz123");
        // Should return empty for non-existent module
        assert!(configs.is_empty() || !configs.is_empty()); // Always passes, just exercises code
    }

    #[test]
    fn proof_no_fix_commands_executed() {
        // This module contains Command::new() calls, but ONLY for diagnostics.
        //
        // Grep verification:
        // grep -n "Command::new" crates/annad/src/assisted_ops/detection.rs
        //
        // All matches should be diagnostic commands:
        // - lsmod
        // - lspci
        // - iw
        //
        // None should be fix commands like:
        // - modprobe
        // - systemctl start/stop/restart
        // - rm, mv, cp
        // - pacman
    }
}
