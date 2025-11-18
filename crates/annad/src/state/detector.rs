//! State detection logic - pure read operations only
//!
//! Detection order:
//! 1. iso_live (highest priority)
//! 2. recovery_candidate
//! 3. configured/degraded
//! 4. post_install_minimal
//! 5. unknown (fallback)
//!
//! Citation: [archwiki:installation_guide], [archwiki:system_maintenance]

use super::types::{NetworkStatus, StateDetails, StateDetection, SystemState};
use anyhow::Result;
use std::fs;
use std::path::Path;
use tracing::{debug, info};

/// Detect current system state using pure read operations
///
/// This function performs NO writes and has NO side effects.
/// All detection is based on filesystem inspection only.
///
/// Returns StateDetection with metadata.
pub fn detect_state() -> Result<StateDetection> {
    info!("Detecting system state");

    // Collect metadata first
    let uefi = is_uefi_system();
    let disks = detect_block_devices();
    let network = detect_network_status();
    let state_file_present = Path::new("/etc/anna/state.json").exists();

    // State detection in priority order
    let state = if is_iso_live() {
        debug!("Detected: iso_live (running from Arch ISO)");
        SystemState::IsoLive
    } else if is_recovery_candidate() {
        debug!("Detected: recovery_candidate (broken system found)");
        SystemState::RecoveryCandidate
    } else if let Some(configured_or_degraded) = check_configured_or_degraded() {
        debug!("Detected: {:?} (managed system)", configured_or_degraded);
        configured_or_degraded
    } else if is_post_install_minimal() {
        debug!("Detected: post_install_minimal (fresh Arch, no Anna state)");
        SystemState::PostInstallMinimal
    } else {
        debug!("Detected: unknown (unable to classify)");
        SystemState::Unknown
    };

    let health_ok = if matches!(state, SystemState::Configured | SystemState::Degraded) {
        Some(state == SystemState::Configured)
    } else {
        None
    };

    let citation = state.citation().to_string();

    Ok(StateDetection {
        state,
        detected_at: chrono::Utc::now().to_rfc3339(),
        details: StateDetails {
            uefi,
            disks,
            network,
            state_file_present,
            health_ok,
        },
        citation,
    })
}

/// Check if running from Arch ISO
///
/// Detection: /run/archiso directory exists
/// Citation: [archwiki:installation_guide:boot_the_live_environment]
fn is_iso_live() -> bool {
    Path::new("/run/archiso").exists()
}

/// Check if system is a recovery candidate
///
/// Detection: Mountable Linux root found with /etc/os-release but not current boot
/// Citation: [archwiki:chroot]
///
/// Note: Simplified detection for Phase 0.2 - checks for common indicators
fn is_recovery_candidate() -> bool {
    // Phase 0.2: Stub detection - real implementation will scan block devices
    // For now, check if we're in a chroot-like environment
    false
}

/// Check if system is post-install minimal
///
/// Detection: /etc/arch-release exists but no /etc/anna/state.json
/// Citation: [archwiki:installation_guide:configure_the_system]
fn is_post_install_minimal() -> bool {
    let has_arch_release = Path::new("/etc/arch-release").exists();
    let has_anna_state = Path::new("/etc/anna/state.json").exists();

    has_arch_release && !has_anna_state
}

/// Check if system is configured or degraded
///
/// Returns Some(Configured) if state file exists and health checks pass
/// Returns Some(Degraded) if state file exists but health checks fail
/// Returns None if state file doesn't exist
///
/// Citation: [archwiki:system_maintenance]
fn check_configured_or_degraded() -> Option<SystemState> {
    let state_file = Path::new("/etc/anna/state.json");

    if !state_file.exists() {
        return None;
    }

    // Try to read and parse state file
    let valid_state_file = fs::read_to_string(state_file)
        .ok()
        .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
        .is_some();

    if !valid_state_file {
        debug!("State file exists but is not valid JSON");
        return Some(SystemState::Degraded);
    }

    // Phase 0.2: Stub health check - always returns OK if state file is valid
    // Real implementation will check systemd units, journal errors, etc.
    let health_ok = health_check_stub();

    if health_ok {
        Some(SystemState::Configured)
    } else {
        Some(SystemState::Degraded)
    }
}

/// Stub health check for Phase 0.2
///
/// Returns true if no obvious health issues detected
/// Real implementation will check:
/// - systemd failed units
/// - critical journal errors
/// - filesystem health
/// - network connectivity
///
/// Citation: [archwiki:system_maintenance#check_for_errors]
fn health_check_stub() -> bool {
    // Phase 0.2: Always return true (optimistic)
    // Phase 1+: Implement real health checks
    true
}

/// Detect if system is UEFI or BIOS
///
/// Detection: /sys/firmware/efi directory exists
/// Citation: [man:efibootmgr(8)]
fn is_uefi_system() -> bool {
    Path::new("/sys/firmware/efi").exists()
}

/// Detect block devices
///
/// Returns list of block device names (e.g., "sda", "nvme0n1")
/// Citation: [man:lsblk(8)]
fn detect_block_devices() -> Vec<String> {
    // Phase 0.2: Read from /sys/block
    let block_dir = Path::new("/sys/block");

    if !block_dir.exists() {
        return vec![];
    }

    fs::read_dir(block_dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| entry.file_name().into_string().ok())
                .filter(|name| !name.starts_with("loop") && !name.starts_with("ram"))
                .collect()
        })
        .unwrap_or_default()
}

/// Detect network status
///
/// Returns network connectivity metadata
/// Citation: [archwiki:network_configuration]
fn detect_network_status() -> NetworkStatus {
    // Phase 0.2: Basic detection from /sys/class/net
    let net_dir = Path::new("/sys/class/net");

    let has_interface = if net_dir.exists() {
        fs::read_dir(net_dir)
            .ok()
            .map(|entries| {
                entries.filter_map(|e| e.ok()).any(|e| {
                    let name = e.file_name();
                    let name_str = name.to_string_lossy();
                    // Skip loopback
                    !name_str.starts_with("lo")
                })
            })
            .unwrap_or(false)
    } else {
        false
    };

    // Phase 0.2: Stub route and DNS checks
    let has_route = has_interface && Path::new("/proc/net/route").exists();
    let can_resolve = false; // Phase 0.2: Skip DNS check to avoid network calls

    NetworkStatus {
        has_interface,
        has_route,
        can_resolve,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_state_succeeds() {
        // Should not panic and return a valid state
        let result = detect_state();
        assert!(result.is_ok());

        let detection = result.unwrap();
        assert!(!detection.detected_at.is_empty());
        assert!(!detection.citation.is_empty());
    }

    #[test]
    fn test_is_uefi_system() {
        // Just ensure it doesn't panic
        let _ = is_uefi_system();
    }

    #[test]
    fn test_detect_block_devices() {
        let disks = detect_block_devices();
        // Test completes successfully if we get here (disks is a valid Vec)
        let _ = disks.len();
    }

    #[test]
    fn test_detect_network_status() {
        let status = detect_network_status();
        // Test completes successfully if we get here (has_interface is a valid boolean)
        let _ = status.has_interface;
    }

    // Mock filesystem tests
    #[test]
    fn test_is_post_install_minimal_logic() {
        // This test documents the logic without filesystem access
        // Real test would use tempdir with mock files
        let has_arch = true;
        let has_anna = false;

        assert!(has_arch && !has_anna);
    }

    #[test]
    fn test_health_check_stub() {
        // Phase 0.2: Always returns true
        assert!(health_check_stub());
    }
}
