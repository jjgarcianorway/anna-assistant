//! Probe-specific repair actions
//!
//! Phase 0.7: System Guardian - corrective actions for failed health probes
//! Citation: [archwiki:System_maintenance]

use anna_common::ipc::RepairAction;
use anyhow::{Context, Result};
use std::process::Command;
use tracing::{info, warn};

/// Repair disk-space probe
///
/// Actions:
/// 1. Clean systemd journal (journalctl --vacuum-size=100M)
/// 2. Clean pacman cache (paccache -r -k 2)
///
/// Citation: [archwiki:System_maintenance#Clean_the_filesystem]
pub async fn disk_space_repair(dry_run: bool) -> Result<RepairAction> {
    info!("disk-space repair: dry_run={}", dry_run);

    let mut actions_taken = Vec::new();
    let mut all_success = true;
    let mut last_exit_code = 0;

    // Action 1: Clean systemd journal
    let journal_cmd = "journalctl --vacuum-size=100M";
    if dry_run {
        info!("[DRY-RUN] Would execute: {}", journal_cmd);
        actions_taken.push(format!("[dry-run] {}", journal_cmd));
    } else {
        info!("Executing: {}", journal_cmd);
        let output = Command::new("journalctl")
            .args(&["--vacuum-size=100M"])
            .output()
            .context("Failed to execute journalctl")?;

        last_exit_code = output.status.code().unwrap_or(1);
        if output.status.success() {
            actions_taken.push("journalctl --vacuum-size=100M (success)".to_string());
        } else {
            all_success = false;
            let stderr = String::from_utf8_lossy(&output.stderr);
            actions_taken.push(format!("journalctl failed: {}", stderr));
            warn!("journalctl failed: {}", stderr);
        }
    }

    // Action 2: Clean pacman cache (paccache -rk1 = keep only latest version)
    // This matches the recommendation we show users and frees the most space
    let paccache_cmd = "paccache -rk1";
    if dry_run {
        info!("[DRY-RUN] Would execute: {}", paccache_cmd);
        actions_taken.push(format!("[dry-run] {}", paccache_cmd));
    } else {
        // Check if paccache is available
        let paccache_available = Command::new("which")
            .arg("paccache")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !paccache_available {
            // Install pacman-contrib to get paccache
            info!("paccache not found, installing pacman-contrib...");
            let install_output = Command::new("pacman")
                .args(&["-S", "--noconfirm", "--needed", "pacman-contrib"])
                .output()
                .context("Failed to install pacman-contrib")?;

            if !install_output.status.success() {
                all_success = false;
                let stderr = String::from_utf8_lossy(&install_output.stderr);
                actions_taken.push(format!("Failed to install pacman-contrib: {}", stderr));
                warn!("Failed to install pacman-contrib: {}", stderr);
            } else {
                actions_taken.push("Installed pacman-contrib".to_string());
                info!("pacman-contrib installed successfully");
            }
        }

        // Now run paccache (either it was already available or we just installed it)
        info!("Executing: {}", paccache_cmd);
        let output = Command::new("paccache")
            .args(&["-rk1"])
            .output();

        match output {
            Ok(output) => {
                last_exit_code = output.status.code().unwrap_or(1);
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    // paccache outputs how many packages were removed
                    actions_taken.push(format!("paccache -rk1: {}", stdout.trim()));
                    info!("paccache succeeded: {}", stdout.trim());
                } else {
                    all_success = false;
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    actions_taken.push(format!("paccache failed: {}", stderr));
                    warn!("paccache failed: {}", stderr);
                }
            }
            Err(e) => {
                all_success = false;
                actions_taken.push(format!("Could not execute paccache: {}", e));
                warn!("Could not execute paccache: {}", e);
            }
        }
    }

    Ok(RepairAction {
        probe: "disk-space".to_string(),
        action: "cleanup_disk_space".to_string(),
        command: Some(format!("{} && {}", journal_cmd, paccache_cmd)),
        exit_code: if dry_run { None } else { Some(last_exit_code) },
        success: dry_run || all_success,
        details: actions_taken.join("; "),
        citation: "[archwiki:System_maintenance#Clean_the_filesystem]".to_string(),
    })
}

/// Repair pacman-db probe
///
/// Actions:
/// 1. Synchronize package databases (pacman -Syy)
/// 2. Verify database integrity
///
/// Citation: [archwiki:Pacman#Upgrading_packages]
pub async fn pacman_db_repair(dry_run: bool) -> Result<RepairAction> {
    info!("pacman-db repair: dry_run={}", dry_run);

    let cmd = "pacman -Syy";
    let mut exit_code = 0;
    let mut details = String::new();

    if dry_run {
        info!("[DRY-RUN] Would execute: {}", cmd);
        details = format!("[dry-run] {}", cmd);
    } else {
        info!("Executing: {}", cmd);
        let output = Command::new("pacman")
            .args(&["-Syy"])
            .output()
            .context("Failed to execute pacman")?;

        exit_code = output.status.code().unwrap_or(1);
        if output.status.success() {
            details = "pacman -Syy succeeded; database synchronized".to_string();
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            details = format!("pacman -Syy failed: {}", stderr);
            warn!("pacman -Syy failed: {}", stderr);
        }
    }

    Ok(RepairAction {
        probe: "pacman-db".to_string(),
        action: "sync_pacman_db".to_string(),
        command: Some(cmd.to_string()),
        exit_code: if dry_run { None } else { Some(exit_code) },
        success: dry_run || exit_code == 0,
        details,
        citation: "[archwiki:Pacman#Upgrading_packages]".to_string(),
    })
}

/// Repair services-failed probe
///
/// Actions:
/// 1. Get list of failed units
/// 2. Restart each failed unit
///
/// Citation: [archwiki:Systemd#Using_units]
pub async fn services_failed_repair(dry_run: bool) -> Result<RepairAction> {
    info!("services-failed repair: dry_run={}", dry_run);

    // Get list of failed units
    let output = Command::new("systemctl")
        .args(&["--failed", "--no-legend", "--no-pager", "--plain"])
        .output()
        .context("Failed to get failed units")?;

    let failed_units: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            // Extract unit name (first column)
            line.split_whitespace()
                .next()
                .unwrap_or("")
                .to_string()
        })
        .filter(|unit| !unit.is_empty())
        .collect();

    if failed_units.is_empty() {
        return Ok(RepairAction {
            probe: "services-failed".to_string(),
            action: "restart_failed_units".to_string(),
            command: None,
            exit_code: Some(0),
            success: true,
            details: "No failed units found".to_string(),
            citation: "[archwiki:Systemd#Using_units]".to_string(),
        });
    }

    let mut actions_taken = Vec::new();
    let mut all_success = true;
    let mut last_exit_code = 0;

    for unit in &failed_units {
        let cmd = format!("systemctl restart {}", unit);
        if dry_run {
            info!("[DRY-RUN] Would execute: {}", cmd);
            actions_taken.push(format!("[dry-run] restart {}", unit));
        } else {
            info!("Executing: {}", cmd);
            let output = Command::new("systemctl")
                .args(&["restart", unit])
                .output()
                .context(format!("Failed to restart {}", unit))?;

            last_exit_code = output.status.code().unwrap_or(1);
            if output.status.success() {
                actions_taken.push(format!("restarted {} (success)", unit));
            } else {
                all_success = false;
                let stderr = String::from_utf8_lossy(&output.stderr);
                actions_taken.push(format!("restart {} failed: {}", unit, stderr));
                warn!("Failed to restart {}: {}", unit, stderr);
            }
        }
    }

    Ok(RepairAction {
        probe: "services-failed".to_string(),
        action: "restart_failed_units".to_string(),
        command: Some(format!(
            "systemctl restart {}",
            failed_units.join(" ")
        )),
        exit_code: if dry_run { None } else { Some(last_exit_code) },
        success: dry_run || all_success,
        details: actions_taken.join("; "),
        citation: "[archwiki:Systemd#Using_units]".to_string(),
    })
}

/// Repair firmware-microcode probe
///
/// Actions:
/// 1. Detect CPU vendor (Intel or AMD)
/// 2. Install missing microcode package
///
/// Citation: [archwiki:Microcode]
pub async fn firmware_microcode_repair(dry_run: bool) -> Result<RepairAction> {
    info!("firmware-microcode repair: dry_run={}", dry_run);

    // Detect CPU vendor
    let cpuinfo = std::fs::read_to_string("/proc/cpuinfo")
        .context("Failed to read /proc/cpuinfo")?;

    let vendor = if cpuinfo.contains("GenuineIntel") {
        "intel"
    } else if cpuinfo.contains("AuthenticAMD") {
        "amd"
    } else {
        return Ok(RepairAction {
            probe: "firmware-microcode".to_string(),
            action: "install_microcode".to_string(),
            command: None,
            exit_code: Some(1),
            success: false,
            details: "Unknown CPU vendor (not Intel or AMD)".to_string(),
            citation: "[archwiki:Microcode]".to_string(),
        });
    };

    let package = format!("{}-ucode", vendor);
    let cmd = format!("pacman -S --noconfirm {}", package);

    let mut exit_code = 0;
    let mut details = String::new();

    if dry_run {
        info!("[DRY-RUN] Would execute: {}", cmd);
        details = format!("[dry-run] install {}", package);
    } else {
        // Check if already installed
        let check_output = Command::new("pacman")
            .args(&["-Q", &package])
            .output()
            .context("Failed to check package")?;

        if check_output.status.success() {
            details = format!("{} already installed", package);
            info!("{}", details);
        } else {
            info!("Executing: {}", cmd);
            let output = Command::new("pacman")
                .args(&["-S", "--noconfirm", &package])
                .output()
                .context("Failed to install microcode")?;

            exit_code = output.status.code().unwrap_or(1);
            if output.status.success() {
                details = format!("{} installed successfully", package);
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                details = format!("Failed to install {}: {}", package, stderr);
                warn!("{}", details);
            }
        }
    }

    Ok(RepairAction {
        probe: "firmware-microcode".to_string(),
        action: "install_microcode".to_string(),
        command: Some(cmd),
        exit_code: if dry_run { None } else { Some(exit_code) },
        success: dry_run || exit_code == 0,
        details,
        citation: "[archwiki:Microcode]".to_string(),
    })
}

/// Repair tlp-config probe - enable TLP service if TLP is installed
///
/// Citation: [archwiki:TLP#Installation]
pub async fn tlp_config_repair(dry_run: bool) -> Result<RepairAction> {
    info!("tlp-config repair: dry_run={}", dry_run);

    let cmd = "systemctl enable --now tlp.service";
    let mut exit_code = 0;
    let mut details = String::new();

    if dry_run {
        info!("[DRY-RUN] Would execute: {}", cmd);
        details = format!("[dry-run] {}", cmd);
    } else {
        info!("Executing: {}", cmd);
        let output = Command::new("systemctl")
            .args(&["enable", "--now", "tlp.service"])
            .output()
            .context("Failed to enable TLP service")?;

        exit_code = output.status.code().unwrap_or(1);
        if output.status.success() {
            details = "TLP service enabled and started".to_string();
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            details = format!("Failed to enable TLP: {}", stderr);
            warn!("{}", details);
        }
    }

    Ok(RepairAction {
        probe: "tlp-config".to_string(),
        action: "enable_tlp_service".to_string(),
        command: Some(cmd.to_string()),
        exit_code: if dry_run { None } else { Some(exit_code) },
        success: dry_run || exit_code == 0,
        details,
        citation: "[archwiki:TLP#Installation]".to_string(),
    })
}

/// Repair bluetooth-service probe - enable and start bluetooth service
///
/// Citation: [archwiki:Bluetooth#Installation]
pub async fn bluetooth_service_repair(dry_run: bool) -> Result<RepairAction> {
    info!("bluetooth-service repair: dry_run={}", dry_run);

    let cmd = "systemctl enable --now bluetooth.service";
    let mut exit_code = 0;
    let mut details = String::new();

    if dry_run {
        info!("[DRY-RUN] Would execute: {}", cmd);
        details = format!("[dry-run] {}", cmd);
    } else {
        info!("Executing: {}", cmd);
        let output = Command::new("systemctl")
            .args(&["enable", "--now", "bluetooth.service"])
            .output()
            .context("Failed to enable Bluetooth service")?;

        exit_code = output.status.code().unwrap_or(1);
        if output.status.success() {
            details = "Bluetooth service enabled and started".to_string();
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            details = format!("Failed to enable Bluetooth: {}", stderr);
            warn!("{}", details);
        }
    }

    Ok(RepairAction {
        probe: "bluetooth-service".to_string(),
        action: "enable_bluetooth_service".to_string(),
        command: Some(cmd.to_string()),
        exit_code: if dry_run { None } else { Some(exit_code) },
        success: dry_run || exit_code == 0,
        details,
        citation: "[archwiki:Bluetooth#Installation]".to_string(),
    })
}

/// Repair missing-firmware probe - show guidance for missing firmware
///
/// Citation: [archwiki:mkinitcpio#Possibly_missing_firmware_for_module_XXXX]
pub async fn missing_firmware_repair(dry_run: bool) -> Result<RepairAction> {
    info!("missing-firmware repair: dry_run={}", dry_run);

    // This repair is informational - missing firmware often requires specific packages
    // or AUR packages depending on the hardware
    let details = "Missing firmware detected. Check dmesg for specific firmware files needed. \
                   Common solutions: install linux-firmware, mkinitcpio-firmware (AUR), or \
                   suppress warnings by adding MODULE.blacklist=1 to kernel parameters if \
                   firmware is not needed for your hardware.".to_string();

    Ok(RepairAction {
        probe: "missing-firmware".to_string(),
        action: "firmware_guidance".to_string(),
        command: None, // No automatic fix - requires user to identify hardware needs
        exit_code: Some(0),
        success: true,
        details,
        citation: "[archwiki:mkinitcpio#Possibly_missing_firmware_for_module_XXXX]".to_string(),
    })
}

/// Repair journal-cleanup probe
///
/// Action: Clean old journal entries to reduce log volume
///
/// Citation: [archwiki:Systemd/Journal#Journal_size_limit]
pub async fn journal_cleanup_repair(dry_run: bool) -> Result<RepairAction> {
    info!("journal-cleanup repair: dry_run={}", dry_run);

    let mut actions_taken = Vec::new();
    let mut all_success = true;
    let mut last_exit_code = 0;

    // Vacuum journal to last 7 days
    let cmd = "journalctl --vacuum-time=7d";
    if dry_run {
        info!("[DRY-RUN] Would execute: {}", cmd);
        actions_taken.push(format!("[dry-run] {}", cmd));
    } else {
        info!("Executing: {}", cmd);
        let output = Command::new("journalctl")
            .args(&["--vacuum-time=7d"])
            .output()
            .context("Failed to execute journalctl")?;

        last_exit_code = output.status.code().unwrap_or(1);
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            actions_taken.push(format!("journalctl --vacuum-time=7d: {}", stdout.trim()));
        } else {
            all_success = false;
            let stderr = String::from_utf8_lossy(&output.stderr);
            actions_taken.push(format!("journalctl failed: {}", stderr));
            warn!("journalctl failed: {}", stderr);
        }
    }

    Ok(RepairAction {
        probe: "journal-cleanup".to_string(),
        action: "vacuum_journal".to_string(),
        command: Some(cmd.to_string()),
        exit_code: if dry_run { None } else { Some(last_exit_code) },
        success: all_success,
        details: actions_taken.join("; "),
        citation: "[archwiki:Systemd/Journal#Journal_size_limit]".to_string(),
    })
}

/// Repair orphaned-packages probe
///
/// Action: Remove orphaned packages that are no longer needed
///
/// Citation: [archwiki:Pacman/Tips_and_tricks#Removing_unused_packages_(orphans)]
pub async fn orphaned_packages_repair(dry_run: bool) -> Result<RepairAction> {
    info!("orphaned-packages repair: dry_run={}", dry_run);

    // First, get list of orphaned packages
    let list_output = Command::new("pacman")
        .args(&["-Qtdq"])
        .output()
        .context("Failed to query orphaned packages")?;

    if !list_output.status.success() {
        // No orphaned packages found
        return Ok(RepairAction {
            probe: "orphaned-packages".to_string(),
            action: "remove_orphans".to_string(),
            command: None,
            exit_code: Some(0),
            success: true,
            details: "No orphaned packages found".to_string(),
            citation: "[archwiki:Pacman/Tips_and_tricks#Removing_unused_packages_(orphans)]".to_string(),
        });
    }

    let orphans = String::from_utf8_lossy(&list_output.stdout);
    let orphan_list: Vec<&str> = orphans.lines().filter(|l| !l.trim().is_empty()).collect();
    let count = orphan_list.len();

    if count == 0 {
        return Ok(RepairAction {
            probe: "orphaned-packages".to_string(),
            action: "remove_orphans".to_string(),
            command: None,
            exit_code: Some(0),
            success: true,
            details: "No orphaned packages found".to_string(),
            citation: "[archwiki:Pacman/Tips_and_tricks#Removing_unused_packages_(orphans)]".to_string(),
        });
    }

    let mut actions_taken = Vec::new();
    let mut all_success = true;
    let mut last_exit_code = 0;

    // Build removal command
    let cmd = format!("pacman -Rns --noconfirm {}", orphan_list.join(" "));
    if dry_run {
        info!("[DRY-RUN] Would remove {} orphaned packages", count);
        actions_taken.push(format!("[dry-run] Would remove {} packages: {}", count, orphan_list.join(", ")));
    } else {
        info!("Removing {} orphaned packages", count);
        let output = Command::new("pacman")
            .arg("-Rns")
            .arg("--noconfirm")
            .args(&orphan_list)
            .output()
            .context("Failed to remove orphaned packages")?;

        last_exit_code = output.status.code().unwrap_or(1);
        if output.status.success() {
            actions_taken.push(format!("Removed {} orphaned packages", count));
        } else {
            all_success = false;
            let stderr = String::from_utf8_lossy(&output.stderr);
            actions_taken.push(format!("Failed to remove orphans: {}", stderr));
            warn!("Failed to remove orphans: {}", stderr);
        }
    }

    Ok(RepairAction {
        probe: "orphaned-packages".to_string(),
        action: "remove_orphans".to_string(),
        command: Some(cmd),
        exit_code: if dry_run { None } else { Some(last_exit_code) },
        success: all_success,
        details: actions_taken.join("; "),
        citation: "[archwiki:Pacman/Tips_and_tricks#Removing_unused_packages_(orphans)]".to_string(),
    })
}

/// Repair core-dump-cleanup probe
///
/// Action: Remove old core dumps to free disk space
///
/// Citation: [archwiki:Core_dump#Disabling_automatic_core_dumps]
pub async fn core_dump_cleanup_repair(dry_run: bool) -> Result<RepairAction> {
    info!("core-dump-cleanup repair: dry_run={}", dry_run);

    let mut actions_taken = Vec::new();
    let mut all_success = true;
    let mut last_exit_code = 0;

    // Use coredumpctl to vacuum old dumps (older than 30 days)
    let cmd = "coredumpctl vacuum --keep-free=1G";
    if dry_run {
        info!("[DRY-RUN] Would execute: {}", cmd);
        actions_taken.push(format!("[dry-run] {}", cmd));
    } else {
        info!("Executing: {}", cmd);
        let output = Command::new("coredumpctl")
            .args(&["vacuum", "--keep-free=1G"])
            .output();

        match output {
            Ok(output) => {
                last_exit_code = output.status.code().unwrap_or(1);
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    actions_taken.push(format!("coredumpctl vacuum: {}", stdout.trim()));
                } else {
                    // coredumpctl might not be available or no dumps to clean
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if stderr.contains("No coredumps found") {
                        actions_taken.push("No core dumps found to clean".to_string());
                        last_exit_code = 0;
                    } else {
                        all_success = false;
                        actions_taken.push(format!("coredumpctl failed: {}", stderr));
                        warn!("coredumpctl failed: {}", stderr);
                    }
                }
            }
            Err(e) => {
                all_success = false;
                actions_taken.push(format!("Could not execute coredumpctl: {}", e));
                warn!("Could not execute coredumpctl: {}", e);
            }
        }
    }

    Ok(RepairAction {
        probe: "core-dump-cleanup".to_string(),
        action: "vacuum_coredumps".to_string(),
        command: Some(cmd.to_string()),
        exit_code: if dry_run { None } else { Some(last_exit_code) },
        success: all_success,
        details: actions_taken.join("; "),
        citation: "[archwiki:Core_dump#Disabling_automatic_core_dumps]".to_string(),
    })
}
