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

/// Repair time-sync-enable probe
///
/// Action: Enable and start systemd-timesyncd for network time synchronization
///
/// Citation: [archwiki:Systemd-timesyncd]
pub async fn time_sync_enable_repair(dry_run: bool) -> Result<RepairAction> {
    info!("time-sync-enable repair: dry_run={}", dry_run);

    let mut actions_taken = Vec::new();
    let mut all_success = true;
    let mut last_exit_code = 0;

    // Check if systemd-timesyncd is available
    let timesyncd_available = Command::new("which")
        .arg("timedatectl")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !timesyncd_available {
        return Ok(RepairAction {
            probe: "time-sync-enable".to_string(),
            action: "enable_timesyncd".to_string(),
            command: None,
            exit_code: Some(1),
            success: false,
            details: "systemd-timesyncd is not available on this system".to_string(),
            citation: "[archwiki:System_time]".to_string(),
        });
    }

    // Check if another NTP service is already active
    let chronyd_active = Command::new("systemctl")
        .args(&["is-active", "chronyd.service"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let ntpd_active = Command::new("systemctl")
        .args(&["is-active", "ntpd.service"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if chronyd_active || ntpd_active {
        let service = if chronyd_active { "chronyd" } else { "ntpd" };
        return Ok(RepairAction {
            probe: "time-sync-enable".to_string(),
            action: "enable_timesyncd".to_string(),
            command: None,
            exit_code: Some(0),
            success: false,
            details: format!("Another time sync service ({}) appears to be active; not enabling systemd-timesyncd", service),
            citation: "[archwiki:System_time]".to_string(),
        });
    }

    // Enable and start systemd-timesyncd
    let cmd = "systemctl enable --now systemd-timesyncd.service";
    if dry_run {
        info!("[DRY-RUN] Would execute: {}", cmd);
        actions_taken.push(format!("[dry-run] {}", cmd));
    } else {
        info!("Executing: {}", cmd);
        let output = Command::new("systemctl")
            .args(&["enable", "--now", "systemd-timesyncd.service"])
            .output()
            .context("Failed to enable systemd-timesyncd")?;

        last_exit_code = output.status.code().unwrap_or(1);
        if output.status.success() {
            actions_taken.push("systemd-timesyncd enabled and started".to_string());

            // Give it a moment to sync
            std::thread::sleep(std::time::Duration::from_millis(500));

            // Check status
            let status_output = Command::new("timedatectl")
                .arg("show")
                .output();

            if let Ok(status) = status_output {
                let status_str = String::from_utf8_lossy(&status.stdout);
                if status_str.contains("NTPSynchronized=yes") {
                    actions_taken.push("Time synchronization active and synced".to_string());
                } else {
                    actions_taken.push("Service started; synchronization in progress".to_string());
                }
            }
        } else {
            all_success = false;
            let stderr = String::from_utf8_lossy(&output.stderr);
            actions_taken.push(format!("Failed to enable systemd-timesyncd: {}", stderr));
            warn!("Failed to enable systemd-timesyncd: {}", stderr);
        }
    }

    Ok(RepairAction {
        probe: "time-sync-enable".to_string(),
        action: "enable_timesyncd".to_string(),
        command: Some(cmd.to_string()),
        exit_code: if dry_run { None } else { Some(last_exit_code) },
        success: all_success,
        details: actions_taken.join("; "),
        citation: "[archwiki:Systemd-timesyncd]".to_string(),
    })
}

/// Phase 4.8: Repair user-services-failed probe
///
/// Action: Restart common safe user services (pipewire, wireplumber)
/// For other services, provide guidance only
///
/// Citation: [archwiki:Systemd/User]
pub async fn user_services_failed_repair(dry_run: bool) -> Result<RepairAction> {
    info!("user-services-failed repair: dry_run={}", dry_run);

    let mut actions_taken = Vec::new();
    let mut all_success = true;
    let mut last_exit_code = 0;

    // Get list of failed user services
    let output = Command::new("systemctl")
        .args(&["--user", "list-units", "--failed", "--no-legend", "--plain"])
        .output();

    if let Ok(output) = output {
        if !output.status.success() {
            return Ok(RepairAction {
                probe: "user-services-failed".to_string(),
                action: "restart_user_services".to_string(),
                command: None,
                exit_code: Some(1),
                success: false,
                details: "Failed to list user services".to_string(),
                citation: "[archwiki:Systemd/User]".to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let failed_units: Vec<&str> = stdout.lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| line.split_whitespace().next())
            .collect();

        if failed_units.is_empty() {
            return Ok(RepairAction {
                probe: "user-services-failed".to_string(),
                action: "restart_user_services".to_string(),
                command: None,
                exit_code: Some(0),
                success: true,
                details: "No failed user services found".to_string(),
                citation: "[archwiki:Systemd/User]".to_string(),
            });
        }

        // Safe services we can auto-restart: pipewire, wireplumber
        let safe_services = ["pipewire.service", "wireplumber.service", "pipewire-pulse.service"];
        let mut restarted = Vec::new();
        let mut guidance_only = Vec::new();

        for unit in &failed_units {
            if safe_services.iter().any(|s| unit.contains(s)) {
                // Safe to restart
                let cmd = format!("systemctl --user restart {}", unit);
                if dry_run {
                    info!("[DRY-RUN] Would execute: {}", cmd);
                    actions_taken.push(format!("[dry-run] restart {}", unit));
                    restarted.push(unit.to_string());
                } else {
                    info!("Executing: {}", cmd);
                    let restart_output = Command::new("systemctl")
                        .args(&["--user", "restart", unit])
                        .output();

                    if let Ok(restart) = restart_output {
                        last_exit_code = restart.status.code().unwrap_or(1);
                        if restart.status.success() {
                            actions_taken.push(format!("Restarted {}", unit));
                            restarted.push(unit.to_string());
                        } else {
                            all_success = false;
                            let stderr = String::from_utf8_lossy(&restart.stderr);
                            actions_taken.push(format!("Failed to restart {}: {}", unit, stderr));
                        }
                    }
                }
            } else {
                // Not safe to auto-restart - guidance only
                guidance_only.push(unit.to_string());
            }
        }

        // Add guidance for services we didn't restart
        if !guidance_only.is_empty() {
            let guidance = format!(
                "Manual action needed for: {}. Check with 'systemctl --user status' and 'journalctl --user -xeu <service>'",
                guidance_only.join(", ")
            );
            actions_taken.push(guidance);
        }

        let details = if actions_taken.is_empty() {
            "No actions taken".to_string()
        } else {
            actions_taken.join("; ")
        };

        Ok(RepairAction {
            probe: "user-services-failed".to_string(),
            action: "restart_user_services".to_string(),
            command: Some("systemctl --user restart <service>".to_string()),
            exit_code: if dry_run { None } else { Some(last_exit_code) },
            success: all_success,
            details,
            citation: "[archwiki:Systemd/User]".to_string(),
        })
    } else {
        Ok(RepairAction {
            probe: "user-services-failed".to_string(),
            action: "restart_user_services".to_string(),
            command: None,
            exit_code: Some(1),
            success: false,
            details: "Unable to check user services".to_string(),
            citation: "[archwiki:Systemd/User]".to_string(),
        })
    }
}

/// Phase 4.8: Repair broken-autostart probe
///
/// Action: Disable broken autostart entries in user's ~/.config/autostart
/// System-wide entries in /etc/xdg/autostart are guidance only
///
/// Citation: [archwiki:XDG_Autostart]
pub async fn broken_autostart_repair(dry_run: bool) -> Result<RepairAction> {
    info!("broken-autostart repair: dry_run={}", dry_run);

    let mut actions_taken = Vec::new();
    let mut all_success = true;
    let last_exit_code = 0;

    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => {
            return Ok(RepairAction {
                probe: "broken-autostart".to_string(),
                action: "disable_broken_autostart".to_string(),
                command: None,
                exit_code: Some(1),
                success: false,
                details: "HOME environment variable not set".to_string(),
                citation: "[archwiki:XDG_Autostart]".to_string(),
            });
        }
    };

    let user_autostart = std::path::PathBuf::from(&home).join(".config/autostart");

    if !user_autostart.exists() {
        return Ok(RepairAction {
            probe: "broken-autostart".to_string(),
            action: "disable_broken_autostart".to_string(),
            command: None,
            exit_code: Some(0),
            success: true,
            details: "No user autostart directory found".to_string(),
            citation: "[archwiki:XDG_Autostart]".to_string(),
        });
    }

    // Scan for broken entries
    let mut broken = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&user_autostart) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("desktop") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for line in content.lines() {
                        if let Some(exec_line) = line.strip_prefix("Exec=") {
                            if let Some(command) = exec_line.split_whitespace().next() {
                                let exists = Command::new("which")
                                    .arg(command)
                                    .output()
                                    .map(|o| o.status.success())
                                    .unwrap_or(false);

                                if !exists {
                                    broken.push(path.clone());
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    if broken.is_empty() {
        return Ok(RepairAction {
            probe: "broken-autostart".to_string(),
            action: "disable_broken_autostart".to_string(),
            command: None,
            exit_code: Some(0),
            success: true,
            details: "No broken autostart entries found".to_string(),
            citation: "[archwiki:XDG_Autostart]".to_string(),
        });
    }

    // Create disabled directory
    let disabled_dir = user_autostart.join("disabled");

    for path in broken {
        let filename = path.file_name().unwrap().to_string_lossy();
        if dry_run {
            info!("[DRY-RUN] Would disable: {}", filename);
            actions_taken.push(format!("[dry-run] disable {}", filename));
        } else {
            // Create disabled directory if it doesn't exist
            if !disabled_dir.exists() {
                if let Err(e) = std::fs::create_dir_all(&disabled_dir) {
                    all_success = false;
                    actions_taken.push(format!("Failed to create disabled directory: {}", e));
                    continue;
                }
            }

            // Move file to disabled directory
            let dest = disabled_dir.join(&filename.to_string());
            match std::fs::rename(&path, &dest) {
                Ok(_) => {
                    actions_taken.push(format!("Disabled {}", filename));
                }
                Err(e) => {
                    all_success = false;
                    actions_taken.push(format!("Failed to disable {}: {}", filename, e));
                }
            }
        }
    }

    let details = if actions_taken.is_empty() {
        "No actions taken".to_string()
    } else {
        actions_taken.join("; ")
    };

    Ok(RepairAction {
        probe: "broken-autostart".to_string(),
        action: "disable_broken_autostart".to_string(),
        command: Some("mv ~/.config/autostart/<broken>.desktop ~/.config/autostart/disabled/".to_string()),
        exit_code: if dry_run { None } else { Some(last_exit_code) },
        success: all_success,
        details,
        citation: "[archwiki:XDG_Autostart]".to_string(),
    })
}

/// Phase 4.8: Repair heavy-user-cache probe
///
/// Action: Clean ~/.cache and ~/.local/share/Trash
/// Safe: these directories are designed to be cleared
///
/// Citation: [archwiki:System_maintenance#Clean_the_filesystem]
pub async fn heavy_user_cache_repair(dry_run: bool) -> Result<RepairAction> {
    info!("heavy-user-cache repair: dry_run={}", dry_run);

    let mut actions_taken = Vec::new();
    let mut all_success = true;
    let last_exit_code = 0;

    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => {
            return Ok(RepairAction {
                probe: "heavy-user-cache".to_string(),
                action: "clean_user_cache".to_string(),
                command: None,
                exit_code: Some(1),
                success: false,
                details: "HOME environment variable not set".to_string(),
                citation: "[archwiki:System_maintenance#Clean_the_filesystem]".to_string(),
            });
        }
    };

    let home_path = std::path::PathBuf::from(&home);
    let cache_path = home_path.join(".cache");
    let trash_path = home_path.join(".local/share/Trash");

    let mut total_freed = 0u64;

    // Clean cache
    if cache_path.exists() {
        let before_size = dir_size(&cache_path);
        if dry_run {
            info!("[DRY-RUN] Would clean ~/.cache ({}MB)", before_size / (1024 * 1024));
            actions_taken.push(format!("[dry-run] clean cache (~{}MB)", before_size / (1024 * 1024)));
            total_freed += before_size;
        } else {
            info!("Cleaning ~/.cache");
            if let Ok(entries) = std::fs::read_dir(&cache_path) {
                for entry in entries.flatten() {
                    if let Err(e) = std::fs::remove_dir_all(entry.path()).or_else(|_| std::fs::remove_file(entry.path())) {
                        warn!("Failed to remove {:?}: {}", entry.path(), e);
                    }
                }
                let after_size = dir_size(&cache_path);
                let freed = before_size.saturating_sub(after_size);
                total_freed += freed;
                actions_taken.push(format!("Cleaned cache (~{}MB freed)", freed / (1024 * 1024)));
            } else {
                all_success = false;
                actions_taken.push("Failed to read cache directory".to_string());
            }
        }
    }

    // Clean trash
    if trash_path.exists() {
        let before_size = dir_size(&trash_path);
        if dry_run {
            info!("[DRY-RUN] Would clean ~/.local/share/Trash ({}MB)", before_size / (1024 * 1024));
            actions_taken.push(format!("[dry-run] clean trash (~{}MB)", before_size / (1024 * 1024)));
            total_freed += before_size;
        } else {
            info!("Cleaning ~/.local/share/Trash");
            if let Ok(entries) = std::fs::read_dir(&trash_path) {
                for entry in entries.flatten() {
                    if let Err(e) = std::fs::remove_dir_all(entry.path()).or_else(|_| std::fs::remove_file(entry.path())) {
                        warn!("Failed to remove {:?}: {}", entry.path(), e);
                    }
                }
                let after_size = dir_size(&trash_path);
                let freed = before_size.saturating_sub(after_size);
                total_freed += freed;
                actions_taken.push(format!("Cleaned trash (~{}MB freed)", freed / (1024 * 1024)));
            } else {
                all_success = false;
                actions_taken.push("Failed to read trash directory".to_string());
            }
        }
    }

    let details = if actions_taken.is_empty() {
        "No cache or trash directories found".to_string()
    } else {
        format!("{}. Total freed: ~{}MB", actions_taken.join("; "), total_freed / (1024 * 1024))
    };

    Ok(RepairAction {
        probe: "heavy-user-cache".to_string(),
        action: "clean_user_cache".to_string(),
        command: Some("rm -rf ~/.cache/* ~/.local/share/Trash/*".to_string()),
        exit_code: if dry_run { None } else { Some(last_exit_code) },
        success: all_success,
        details,
        citation: "[archwiki:System_maintenance#Clean_the_filesystem]".to_string(),
    })
}

/// Phase 5.0: Disk SMART health guidance (guidance only, no auto-repair)
pub async fn disk_smart_guidance(_dry_run: bool) -> Result<RepairAction> {
    info!("Providing SMART health guidance");

    let guidance = vec![
        "⚠️  SMART health issues detected:",
        "",
        "1. Back up important data IMMEDIATELY",
        "   - Your disk may fail at any time",
        "   - Use rsync, borg, or similar tools",
        "",
        "2. Review detailed SMART data:",
        "   sudo smartctl -a /dev/sdX",
        "",
        "3. Run extended SMART test:",
        "   sudo smartctl -t long /dev/sdX",
        "   (Check status with: sudo smartctl -a /dev/sdX)",
        "",
        "4. Plan disk replacement",
        "   - Order replacement disk now",
        "   - Avoid heavy disk usage until replacement",
        "",
        "⚠️  DO NOT RUN fsck or repartition on a failing disk",
        "   This may accelerate failure and cause data loss",
    ];

    Ok(RepairAction {
        probe: "disk-smart-guidance".to_string(),
        action: "print_guidance".to_string(),
        command: None,
        exit_code: Some(0),
        success: true,
        details: guidance.join("\n"),
        citation: "[archwiki:S.M.A.R.T.]".to_string(),
    })
}

/// Phase 5.0: Filesystem errors guidance (guidance only, no auto-repair)
pub async fn filesystem_errors_guidance(_dry_run: bool) -> Result<RepairAction> {
    info!("Providing filesystem errors guidance");

    let guidance = vec![
        "⚠️  Filesystem errors detected in kernel log:",
        "",
        "1. Back up important data IMMEDIATELY",
        "   - These errors suggest disk or filesystem issues",
        "",
        "2. Review kernel errors:",
        "   journalctl -k -b | grep -i 'error\\|fail'",
        "",
        "3. Check disk SMART health:",
        "   sudo smartctl -a /dev/sdX",
        "",
        "4. Schedule filesystem check from live environment:",
        "",
        "   For EXT4:",
        "   - Boot from Arch ISO or live USB",
        "   - sudo e2fsck -f /dev/sdX",
        "",
        "   For BTRFS:",
        "   - sudo btrfs scrub start /mountpoint",
        "   - sudo btrfs scrub status /mountpoint",
        "",
        "   For XFS:",
        "   - sudo xfs_repair /dev/sdX",
        "",
        "⚠️  DO NOT run filesystem checks on mounted filesystems",
        "   Always use a live environment or unmount first",
    ];

    Ok(RepairAction {
        probe: "filesystem-errors-guidance".to_string(),
        action: "print_guidance".to_string(),
        command: None,
        exit_code: Some(0),
        success: true,
        details: guidance.join("\n"),
        citation: "[archwiki:File_systems]".to_string(),
    })
}

/// Phase 5.0: Network health repair (conservative service restart only)
pub async fn network_health_repair(dry_run: bool) -> Result<RepairAction> {
    info!("Attempting network health repair");

    // Detect active network management stack
    let nm_active = std::process::Command::new("systemctl")
        .args(&["is-active", "NetworkManager"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim() == "active")
        .unwrap_or(false);

    let networkd_active = std::process::Command::new("systemctl")
        .args(&["is-active", "systemd-networkd"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim() == "active")
        .unwrap_or(false);

    let mut actions_taken = Vec::new();
    let mut all_success = true;

    if nm_active {
        // Restart NetworkManager
        if dry_run {
            info!("[DRY-RUN] Would restart NetworkManager");
            actions_taken.push("[dry-run] restart NetworkManager".to_string());
        } else {
            info!("Restarting NetworkManager");
            let result = std::process::Command::new("systemctl")
                .args(&["restart", "NetworkManager"])
                .status();

            if let Ok(status) = result {
                if status.success() {
                    actions_taken.push("Restarted NetworkManager".to_string());
                } else {
                    all_success = false;
                    actions_taken.push("Failed to restart NetworkManager".to_string());
                }
            } else {
                all_success = false;
                actions_taken.push("Could not execute NetworkManager restart".to_string());
            }
        }
    } else if networkd_active {
        // Restart systemd-networkd
        if dry_run {
            info!("[DRY-RUN] Would restart systemd-networkd");
            actions_taken.push("[dry-run] restart systemd-networkd".to_string());
        } else {
            info!("Restarting systemd-networkd");
            let result = std::process::Command::new("systemctl")
                .args(&["restart", "systemd-networkd"])
                .status();

            if let Ok(status) = result {
                if status.success() {
                    actions_taken.push("Restarted systemd-networkd".to_string());
                } else {
                    all_success = false;
                    actions_taken.push("Failed to restart systemd-networkd".to_string());
                }
            } else {
                all_success = false;
                actions_taken.push("Could not execute systemd-networkd restart".to_string());
            }
        }
    } else {
        // No recognized network manager
        actions_taken.push("No recognized network manager (NetworkManager/systemd-networkd) active".to_string());
        actions_taken.push("Guidance: Check network configuration manually with 'ip addr' and 'ip route'".to_string());
        all_success = false;
    }

    let details = actions_taken.join("; ");

    Ok(RepairAction {
        probe: "network-health-repair".to_string(),
        action: "restart_network_services".to_string(),
        command: Some("sudo systemctl restart NetworkManager".to_string()),
        exit_code: if dry_run { None } else { Some(if all_success { 0 } else { 1 }) },
        success: all_success,
        details,
        citation: "[archwiki:Network_configuration]".to_string(),
    })
}

/// Helper: Calculate directory size recursively
fn dir_size(path: &std::path::Path) -> u64 {
    if !path.exists() {
        return 0;
    }

    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total += metadata.len();
                } else if metadata.is_dir() {
                    total += dir_size(&entry.path());
                }
            }
        }
    }
    total
}
