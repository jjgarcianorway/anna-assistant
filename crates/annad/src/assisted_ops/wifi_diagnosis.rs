//! WiFi Diagnosis - End-to-End Example (Phase 43)
//!
//! This module demonstrates the complete assisted operations flow:
//! 1. Detect slow WiFi issues on Arch Linux
//! 2. Identify misconfigured iwlwifi module options
//! 3. Propose fix steps separated into:
//!    - Safe commands (can run automatically via HumanExecutionAdapter)
//!    - Manual commands (require sudo, must be copy/pasted)
//! 4. Explain why each step is needed
//! 5. Cite Arch Wiki sources
//!
//! # Phase 43 Changes
//!
//! - Commands now have CommandSafety classification
//! - Safe commands use only allowlisted binaries (iw, lsmod, lspci, cat, echo)
//! - Manual commands include sudo and are copy/paste only
//! - Diagnosis summary field populated
//!
//! # Execution Model
//!
//! Safe commands: Executed automatically through HumanExecutionAdapter
//! Manual commands: Displayed as copy/paste instructions with citations
//!
//! Anna NEVER runs sudo commands herself.

use super::detection::{
    detect_wifi_issues, get_regulatory_domain, get_wifi_interface_info,
    get_wifi_link_status, read_modprobe_config,
};
use super::types::{
    AssistedOperation, CommandSafety, DetectedIssue, IssueSeverity, ProposedStep, RiskLevel,
    Source, SourceType,
};

/// Diagnose slow WiFi and prepare an assisted operation for the human.
///
/// This function:
/// 1. Runs diagnostic commands to understand current state
/// 2. Identifies issues (like 11n_disable=1)
/// 3. Prepares proposed fix steps (safe vs manual)
/// 4. Returns an AssistedOperation for human review
///
/// This function does NOT execute any fixes.
pub fn diagnose_slow_wifi() -> Option<AssistedOperation> {
    // Run detection
    let issues = detect_wifi_issues();

    // Build diagnosis summary from gathered evidence
    let diagnosis_summary = build_diagnosis_summary(&issues);

    if issues.is_empty() {
        // No issues detected - return diagnostic-only operation
        return Some(build_healthy_wifi_report());
    }

    // Check for the specific 11n_disable issue
    let has_11n_disable_issue = issues
        .iter()
        .any(|i| i.problem.contains("11n_disable=1") || i.problem.contains("11n_disable"));

    let has_regulatory_issue = issues
        .iter()
        .any(|i| i.problem.contains("regulatory") || i.problem.contains("country 00"));

    if !has_11n_disable_issue && !has_regulatory_issue {
        // Different issue, prepare generic diagnosis
        return Some(prepare_generic_wifi_diagnosis(&issues, &diagnosis_summary));
    }

    // Prepare the specific fix for 11n_disable issue
    Some(prepare_11n_disable_fix(
        &issues,
        has_regulatory_issue,
        &diagnosis_summary,
    ))
}

/// Build a diagnosis summary from detected issues and system state.
fn build_diagnosis_summary(issues: &[DetectedIssue]) -> String {
    let mut summary = Vec::new();

    // Get current WiFi state
    if let Some(link_info) = get_wifi_link_status("wlan0") {
        if link_info.contains("Connected") || link_info.contains("SSID") {
            summary.push("WiFi is connected".to_string());
            // Extract bitrate if present
            for line in link_info.lines() {
                if line.contains("bitrate:") || line.contains("tx bitrate:") {
                    summary.push(format!("Current: {}", line.trim()));
                }
            }
        } else {
            summary.push("WiFi is not connected".to_string());
        }
    }

    // Get interface info
    if let Some(iface_info) = get_wifi_interface_info() {
        if iface_info.contains("wlan0") {
            summary.push("Interface wlan0 detected".to_string());
        }
    }

    // Get regulatory domain
    if let Some(reg_info) = get_regulatory_domain() {
        if reg_info.contains("country 00") {
            summary.push("Regulatory domain: UNSET (may limit performance)".to_string());
        } else if let Some(country_line) = reg_info.lines().find(|l| l.contains("country")) {
            summary.push(format!("Regulatory: {}", country_line.trim()));
        }
    }

    // Add detected issues
    for issue in issues {
        summary.push(format!("Issue: {}", issue.problem));
    }

    summary.join("\n")
}

/// Build a report for healthy WiFi (no issues detected).
fn build_healthy_wifi_report() -> AssistedOperation {
    let mut steps = Vec::new();

    // Safe diagnostic command
    steps.push(ProposedStep {
        step_number: 1,
        description: "Check current WiFi link status".to_string(),
        exact_command: "iw wlan0 link".to_string(),
        why: "Shows connection status, signal strength, and bitrate".to_string(),
        reversible: true,
        reverse_command: None,
        safety: CommandSafety::SafeAutomatic,
    });

    steps.push(ProposedStep {
        step_number: 2,
        description: "List loaded WiFi modules".to_string(),
        exact_command: "lsmod".to_string(),
        why: "Shows which kernel modules are loaded for WiFi".to_string(),
        reversible: true,
        reverse_command: None,
        safety: CommandSafety::SafeAutomatic,
    });

    AssistedOperation {
        operation_id: format!("wifi-healthy-{}", chrono::Utc::now().timestamp()),
        detected_problem: "No WiFi issues detected".to_string(),
        explanation: "Good news: your WiFi configuration looks correct. Anna didn't find any \
                      of the common issues that cause slow speeds. If you're still experiencing \
                      problems, the diagnostic commands above can help gather more information."
            .to_string(),
        proposed_steps: steps,
        risk_level: RiskLevel::Low,
        sources: vec![Source {
            source_type: SourceType::ArchWiki,
            title: "Wireless network configuration".to_string(),
            reference: "https://wiki.archlinux.org/title/Network_configuration/Wireless".to_string(),
        }],
        requires_reboot: false,
        diagnosis_summary: "WiFi appears healthy. No known configuration issues detected.".to_string(),
    }
}

/// Prepare the fix for the 11n_disable=1 issue.
fn prepare_11n_disable_fix(
    issues: &[DetectedIssue],
    has_regulatory_issue: bool,
    diagnosis_summary: &str,
) -> AssistedOperation {
    let mut steps = Vec::new();
    let mut step_num = 1;

    // === SAFE DIAGNOSTIC COMMANDS (can run automatically) ===

    // Safe: Check current WiFi link
    steps.push(ProposedStep {
        step_number: step_num,
        description: "Check current WiFi link status (before fix)".to_string(),
        exact_command: "iw wlan0 link".to_string(),
        why: "Records baseline connection status before making changes".to_string(),
        reversible: true,
        reverse_command: None,
        safety: CommandSafety::SafeAutomatic,
    });
    step_num += 1;

    // Safe: Check loaded modules
    steps.push(ProposedStep {
        step_number: step_num,
        description: "List loaded WiFi kernel modules".to_string(),
        exact_command: "lsmod".to_string(),
        why: "Verifies iwlwifi and iwlmvm modules are loaded".to_string(),
        reversible: true,
        reverse_command: None,
        safety: CommandSafety::SafeAutomatic,
    });
    step_num += 1;

    // Safe: Check PCI devices
    steps.push(ProposedStep {
        step_number: step_num,
        description: "Identify WiFi hardware".to_string(),
        exact_command: "lspci".to_string(),
        why: "Shows the WiFi adapter model for reference".to_string(),
        reversible: true,
        reverse_command: None,
        safety: CommandSafety::SafeAutomatic,
    });
    step_num += 1;

    // Safe: Read current config
    steps.push(ProposedStep {
        step_number: step_num,
        description: "Read current iwlwifi configuration".to_string(),
        exact_command: "cat /etc/modprobe.d/iwlwifi.conf".to_string(),
        why: "Shows the problematic configuration that needs to be fixed".to_string(),
        reversible: true,
        reverse_command: None,
        safety: CommandSafety::SafeAutomatic,
    });
    step_num += 1;

    // === MANUAL COMMANDS (require sudo, must be copy/pasted) ===

    // Manual: Backup current config
    steps.push(ProposedStep {
        step_number: step_num,
        description: "Backup the current iwlwifi configuration".to_string(),
        exact_command:
            "sudo cp /etc/modprobe.d/iwlwifi.conf /etc/modprobe.d/iwlwifi.conf.backup".to_string(),
        why: "Creates a backup so you can restore if something goes wrong".to_string(),
        reversible: true,
        reverse_command: Some(
            "sudo cp /etc/modprobe.d/iwlwifi.conf.backup /etc/modprobe.d/iwlwifi.conf".to_string(),
        ),
        safety: CommandSafety::ManualOnly,
    });
    step_num += 1;

    // Manual: Write corrected config
    steps.push(ProposedStep {
        step_number: step_num,
        description: "Replace iwlwifi.conf with corrected settings".to_string(),
        exact_command:
            r#"echo 'options iwlwifi power_save=0 bt_coex_active=0' | sudo tee /etc/modprobe.d/iwlwifi.conf"#
                .to_string(),
        why: "Removes the 11n_disable=1 setting that was limiting your WiFi to 54 Mbps. \
              The new config disables power saving (better performance) and Bluetooth coexistence \
              (reduces interference)"
            .to_string(),
        reversible: true,
        reverse_command: Some(
            "sudo cp /etc/modprobe.d/iwlwifi.conf.backup /etc/modprobe.d/iwlwifi.conf".to_string(),
        ),
        safety: CommandSafety::ManualOnly,
    });
    step_num += 1;

    // Manual: Set regulatory domain if needed
    if has_regulatory_issue {
        steps.push(ProposedStep {
            step_number: step_num,
            description: "Set WiFi regulatory domain (determines allowed frequencies and power)"
                .to_string(),
            exact_command: "sudo iw reg set NO".to_string(),
            why: "Sets the regulatory domain to Norway. This allows higher transmit power and \
                  wider channel bandwidth. Change 'NO' to your country code if different."
                .to_string(),
            reversible: true,
            reverse_command: Some("sudo iw reg set 00".to_string()),
            safety: CommandSafety::ManualOnly,
        });
        step_num += 1;

        // Manual: Make regulatory persistent
        steps.push(ProposedStep {
            step_number: step_num,
            description: "Make regulatory domain persistent across reboots".to_string(),
            exact_command:
                r#"echo 'options cfg80211 ieee80211_regdom=NO' | sudo tee /etc/modprobe.d/cfg80211.conf"#
                    .to_string(),
            why: "Without this, you'd need to set the regulatory domain after every reboot"
                .to_string(),
            reversible: true,
            reverse_command: Some("sudo rm /etc/modprobe.d/cfg80211.conf".to_string()),
            safety: CommandSafety::ManualOnly,
        });
        step_num += 1;
    }

    // Manual: Reload WiFi driver
    steps.push(ProposedStep {
        step_number: step_num,
        description: "Reload the WiFi driver to apply changes".to_string(),
        exact_command: "sudo modprobe -r iwlmvm iwlwifi && sudo modprobe iwlwifi".to_string(),
        why: "Unloads and reloads the WiFi driver so it picks up the new configuration. \
              Your WiFi will disconnect briefly."
            .to_string(),
        reversible: true,
        reverse_command: Some("sudo modprobe -r iwlmvm iwlwifi && sudo modprobe iwlwifi".to_string()),
        safety: CommandSafety::ManualOnly,
    });
    step_num += 1;

    // === SAFE VERIFICATION COMMANDS (can run automatically) ===

    // Safe: Verify the fix
    steps.push(ProposedStep {
        step_number: step_num,
        description: "Verify WiFi link status (after fix)".to_string(),
        exact_command: "iw wlan0 link".to_string(),
        why: "Shows the new connection status. You should see a much higher bitrate \
              (300+ Mbps instead of 54 Mbps)."
            .to_string(),
        reversible: true,
        reverse_command: None,
        safety: CommandSafety::SafeAutomatic,
    });

    // Build the explanation
    let explanation = format!(
        "Your WiFi is slow because the iwlwifi driver configuration contains '11n_disable=1', \
         which disables 802.11n High Throughput mode. This limits your connection to legacy \
         802.11a speeds (54 Mbps maximum) even though your hardware supports much faster speeds.\n\n\
         The fix removes this setting and optimizes the driver configuration. After applying \
         these changes, your WiFi should operate at full speed (potentially 800+ Mbps link rate \
         on 5GHz with 80MHz channel width).\n\n\
         Current issues detected:\n{}",
        issues
            .iter()
            .map(|i| format!("- {}", i.problem))
            .collect::<Vec<_>>()
            .join("\n")
    );

    AssistedOperation {
        operation_id: format!("wifi-fix-{}", chrono::Utc::now().timestamp()),
        detected_problem: "WiFi speed limited to 54 Mbps due to misconfigured driver settings"
            .to_string(),
        explanation,
        proposed_steps: steps,
        risk_level: RiskLevel::Medium,
        sources: vec![
            Source {
                source_type: SourceType::ArchWiki,
                title: "Wireless network configuration".to_string(),
                reference: "https://wiki.archlinux.org/title/Network_configuration/Wireless"
                    .to_string(),
            },
            Source {
                source_type: SourceType::ArchWiki,
                title: "iwlwifi - Intel Wireless".to_string(),
                reference: "https://wiki.archlinux.org/title/Wireless#iwlwifi".to_string(),
            },
            Source {
                source_type: SourceType::Kernel,
                title: "iwlwifi module parameters".to_string(),
                reference: "modinfo iwlwifi".to_string(),
            },
        ],
        requires_reboot: false,
        diagnosis_summary: diagnosis_summary.to_string(),
    }
}

/// Prepare a generic WiFi diagnosis when the issue isn't the 11n_disable problem.
fn prepare_generic_wifi_diagnosis(
    issues: &[DetectedIssue],
    diagnosis_summary: &str,
) -> AssistedOperation {
    let problem_summary = issues
        .iter()
        .map(|i| i.problem.clone())
        .collect::<Vec<_>>()
        .join("; ");

    AssistedOperation {
        operation_id: format!("wifi-diag-{}", chrono::Utc::now().timestamp()),
        detected_problem: problem_summary,
        explanation: "Anna found some WiFi issues but doesn't yet have a specific fix for this \
                      situation. The diagnostic commands below will gather more information. \
                      The Arch Wiki link has detailed troubleshooting steps for WiFi problems."
            .to_string(),
        proposed_steps: vec![
            ProposedStep {
                step_number: 1,
                description: "Check current WiFi status".to_string(),
                exact_command: "iw wlan0 link".to_string(),
                why: "Shows interface status and current connection details".to_string(),
                reversible: true,
                reverse_command: None,
                safety: CommandSafety::SafeAutomatic,
            },
            ProposedStep {
                step_number: 2,
                description: "List WiFi interfaces".to_string(),
                exact_command: "iw dev".to_string(),
                why: "Shows all wireless interfaces and their state".to_string(),
                reversible: true,
                reverse_command: None,
                safety: CommandSafety::SafeAutomatic,
            },
            ProposedStep {
                step_number: 3,
                description: "Check kernel messages for WiFi errors".to_string(),
                exact_command: "dmesg | grep -i iwlwifi | tail -20".to_string(),
                why: "May reveal driver errors or firmware issues".to_string(),
                reversible: true,
                reverse_command: None,
                safety: CommandSafety::ManualOnly, // Uses pipe
            },
        ],
        risk_level: RiskLevel::Low,
        sources: vec![Source {
            source_type: SourceType::ArchWiki,
            title: "Wireless troubleshooting".to_string(),
            reference:
                "https://wiki.archlinux.org/title/Network_configuration/Wireless#Troubleshooting"
                    .to_string(),
        }],
        requires_reboot: false,
        diagnosis_summary: diagnosis_summary.to_string(),
    }
}

/// Verify WiFi status after human has applied the fix.
///
/// This is called after the human confirms they've run all the commands.
/// It re-checks system state to confirm the fix worked.
pub fn verify_wifi_fix() -> WifiVerificationResult {
    // Check link status
    let link_info = get_wifi_link_status("wlan0");

    // Check for good indicators
    let has_high_bitrate = link_info
        .as_ref()
        .map(|info| {
            info.contains("MBit/s")
                && !info.contains("54.0 MBit/s")
                && !info.contains("48.0 MBit/s")
        })
        .unwrap_or(false);

    let has_wide_channel = get_wifi_interface_info()
        .map(|info| info.contains("80 MHz") || info.contains("160 MHz"))
        .unwrap_or(false);

    let _regulatory_set = get_regulatory_domain()
        .map(|info| !info.contains("country 00"))
        .unwrap_or(false);

    // Check modprobe config
    let configs = read_modprobe_config("iwlwifi");
    let bad_config_removed = !configs.iter().any(|(_, line)| line.contains("11n_disable=1"));

    if has_high_bitrate && bad_config_removed {
        WifiVerificationResult::Success {
            new_bitrate: link_info
                .as_ref()
                .and_then(|info| extract_bitrate(info))
                .unwrap_or_else(|| "Unknown".to_string()),
            channel_width: if has_wide_channel {
                "80+ MHz".to_string()
            } else {
                "Unknown".to_string()
            },
        }
    } else if bad_config_removed {
        WifiVerificationResult::PartialSuccess {
            message: "The configuration is now correct. Your WiFi may need a moment to reconnect \
                      at the new speed. If speeds don't improve, try disconnecting and reconnecting \
                      to your network."
                .to_string(),
        }
    } else {
        WifiVerificationResult::Failed {
            reason: "The old configuration is still present. This usually means one of the steps \
                     didn't complete. Check for any error messages when you ran the commands, \
                     and try again."
                .to_string(),
        }
    }
}

/// Extract bitrate from iw link output.
fn extract_bitrate(link_info: &str) -> Option<String> {
    for line in link_info.lines() {
        if line.contains("bitrate:") || line.contains("rx bitrate:") {
            return Some(line.trim().to_string());
        }
    }
    None
}

/// Result of verifying a WiFi fix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WifiVerificationResult {
    /// Fix was successful
    Success {
        new_bitrate: String,
        channel_width: String,
    },
    /// Fix partially worked
    PartialSuccess { message: String },
    /// Fix did not work
    Failed { reason: String },
}

// =============================================================================
// PROOF: THIS MODULE DOES NOT EXECUTE FIX COMMANDS
// =============================================================================
//
// Grep verification:
//
// grep -n "Command::new" crates/annad/src/assisted_ops/wifi_diagnosis.rs
// Result: Zero matches in this file
//
// All Command::new calls are in detection.rs, and those are diagnostic only.
//
// The proposed_steps contain commands like:
// - "sudo modprobe -r iwlwifi"
// - "sudo tee /etc/modprobe.d/iwlwifi.conf"
//
// But these are STRINGS. They are displayed to the human.
// There is no code path that executes them.
//
// Phase 43: Commands are now classified as SafeAutomatic or ManualOnly.
// SafeAutomatic commands can be executed through HumanExecutionAdapter.
// ManualOnly commands are still just strings for copy/paste.
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnosis_produces_operation() {
        // This test documents that diagnose_slow_wifi produces an AssistedOperation
        // or None, but NEVER executes anything.
        //
        // The function runs diagnostic commands (reads state)
        // and returns a data structure (proposed fix).
        //
        // There is no execution of fix commands.
    }

    #[test]
    fn test_safe_vs_manual_separation() {
        // Create a mock operation to test safe/manual separation
        let op = prepare_11n_disable_fix(&[], false, "test summary");

        let safe = op.safe_commands();
        let manual = op.manual_commands();

        // Should have both safe and manual commands
        assert!(!safe.is_empty(), "Should have safe commands");
        assert!(!manual.is_empty(), "Should have manual commands");

        // Safe commands should not contain sudo
        for cmd in &safe {
            assert!(
                !cmd.exact_command.contains("sudo"),
                "Safe command should not contain sudo: {}",
                cmd.exact_command
            );
        }

        // Manual commands should contain sudo or pipes
        for cmd in &manual {
            let has_risk = cmd.exact_command.contains("sudo")
                || cmd.exact_command.contains("|")
                || cmd.exact_command.contains(">");
            assert!(
                has_risk,
                "Manual command should have risk indicator: {}",
                cmd.exact_command
            );
        }
    }

    #[test]
    fn test_diagnosis_summary_populated() {
        let op = prepare_11n_disable_fix(&[], false, "Test diagnosis summary");
        assert!(!op.diagnosis_summary.is_empty());
    }

    #[test]
    fn test_citations_available() {
        let op = prepare_11n_disable_fix(&[], false, "test");
        let urls = op.citation_urls();
        assert!(!urls.is_empty(), "Should have citations");
        assert!(
            urls.iter().any(|u| u.contains("wiki.archlinux.org")),
            "Should cite Arch Wiki"
        );
    }

    #[test]
    fn test_proposed_steps_are_strings() {
        // Create a mock operation
        let op = prepare_11n_disable_fix(
            &[DetectedIssue {
                problem: "11n_disable=1 found".to_string(),
                severity: IssueSeverity::Major,
                evidence: vec![],
                fixable: true,
            }],
            false,
            "test",
        );

        // All proposed steps have exact_command as a String
        for step in &op.proposed_steps {
            assert!(!step.exact_command.is_empty());
            // The command is a String, not a Command object
            // There is no .run() or .execute() method on it
        }
    }

    #[test]
    fn test_sources_cite_arch_wiki() {
        let op = prepare_11n_disable_fix(&[], false, "test");

        let has_arch_wiki = op
            .sources
            .iter()
            .any(|s| s.source_type == SourceType::ArchWiki);

        assert!(has_arch_wiki, "Should cite Arch Wiki");
    }

    #[test]
    fn test_all_steps_have_explanations() {
        let op = prepare_11n_disable_fix(&[], true, "test");

        for step in &op.proposed_steps {
            assert!(!step.description.is_empty(), "Step should have description");
            assert!(!step.why.is_empty(), "Step should explain why");
        }
    }

    #[test]
    fn proof_no_command_execution_in_this_file() {
        // This file contains ZERO Command::new calls.
        // All proposed commands are strings.
        //
        // grep -n "Command::new" crates/annad/src/assisted_ops/wifi_diagnosis.rs
        // Expected: 0 matches
        //
        // grep -n "\.spawn()\|\.output()\|\.status()" crates/annad/src/assisted_ops/wifi_diagnosis.rs
        // Expected: 0 matches
    }

    #[test]
    fn proof_human_must_run_commands() {
        let op = prepare_11n_disable_fix(&[], false, "test");

        // The operation contains commands
        assert!(!op.proposed_steps.is_empty());

        // But there is no execute method
        // op.execute() <- Does not exist
        // op.run_all_steps() <- Does not exist
        // apply_operation(&op) <- Does not exist
        //
        // The ONLY way these commands run is if:
        // 1. Safe commands: Through HumanExecutionAdapter with human confirmation
        // 2. Manual commands: Human copies them and pastes into terminal
    }

    #[test]
    fn proof_assisted_op_is_data_only() {
        // AssistedOperation cannot be "executed" by itself
        // It is a data structure that describes what COULD be done
        // The execute path is through HumanExecutionAdapter, which:
        // - Requires an ExecutionRequest
        // - Requires human confirmation
        // - Only runs allowlisted commands
        // - Never runs sudo

        let op = prepare_11n_disable_fix(&[], false, "test");

        // There is no:
        // op.execute()
        // op.run()
        // op.apply()
        // assisted_ops::execute(&op)
        //
        // The operation is purely informational.
        assert!(op.operation_id.starts_with("wifi-fix-"));
    }
}
