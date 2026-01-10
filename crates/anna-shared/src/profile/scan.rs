//! System scanning - Gathers hardware and config information.
//!
//! Uses standard Linux tools to discover system state.
//! NO HARDCODING - just captures what's there.

use anyhow::Result;
use std::process::Command;

use super::*;
use crate::profile::configs::scan_configs;
use crate::profile::detect::*;
use crate::profile::hardware::scan_hardware;

/// Perform a full system scan and return a profile
pub fn scan_system() -> Result<SystemProfile> {
    tracing::info!("Scanning system profile...");

    let mut profile = SystemProfile::default();

    // Scan hardware
    profile.hardware = scan_hardware()?;

    // Scan configs
    profile.configs = scan_configs()?;

    // Scan system info
    profile.system = scan_system_info()?;

    // Update timestamp
    profile.last_updated = Some(chrono::Utc::now().to_rfc3339());

    tracing::info!(
        "System scan complete: {} PCI devices, {} configs",
        profile.hardware.pci_devices.len(),
        profile.configs.modprobe.len()
            + profile.configs.udev_rules.len()
            + profile.configs.systemd_overrides.len()
    );

    Ok(profile)
}

/// Scan system information
fn scan_system_info() -> Result<SystemInfo> {
    let mut info = SystemInfo::default();

    // OS info from /etc/os-release
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("NAME=") {
                info.os_name = Some(line[5..].trim_matches('"').to_string());
            } else if line.starts_with("VERSION=") {
                info.os_version = Some(line[8..].trim_matches('"').to_string());
            }
        }
    }

    // Kernel version
    if let Ok(output) = Command::new("uname").arg("-r").output() {
        if output.status.success() {
            info.kernel = Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }
    }

    // Hostname
    if let Ok(output) = Command::new("hostname").output() {
        if output.status.success() {
            info.hostname = Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }
    }

    // Desktop environment
    info.desktop = std::env::var("XDG_CURRENT_DESKTOP").ok();

    // Display server (check for running display managers)
    info.display_server = detect_display_server();

    // Enhanced profile detection (v0.0.863)
    info.bootloader = detect_bootloader();
    info.shell = detect_shell();
    info.editor = detect_editor();
    info.aur_helper = detect_aur_helper();
    info.root_filesystem = detect_root_filesystem();
    info.display_manager = detect_display_manager();
    info.audio_system = detect_audio_system();

    tracing::info!(
        "Enhanced profile: bootloader={:?}, shell={:?}, editor={:?}, fs={:?}",
        info.bootloader,
        info.shell,
        info.editor,
        info.root_filesystem
    );

    Ok(info)
}
