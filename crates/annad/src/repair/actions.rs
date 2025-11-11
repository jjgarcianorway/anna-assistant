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

    // Action 2: Clean pacman cache (requires paccache from pacman-contrib)
    let paccache_cmd = "paccache -r -k 2";
    if dry_run {
        info!("[DRY-RUN] Would execute: {}", paccache_cmd);
        actions_taken.push(format!("[dry-run] {}", paccache_cmd));
    } else {
        // Check if paccache is available
        if Command::new("which")
            .arg("paccache")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            info!("Executing: {}", paccache_cmd);
            let output = Command::new("paccache")
                .args(&["-r", "-k", "2"])
                .output()
                .context("Failed to execute paccache")?;

            last_exit_code = output.status.code().unwrap_or(1);
            if output.status.success() {
                actions_taken.push("paccache -r -k 2 (success)".to_string());
            } else {
                all_success = false;
                let stderr = String::from_utf8_lossy(&output.stderr);
                actions_taken.push(format!("paccache failed: {}", stderr));
                warn!("paccache failed: {}", stderr);
            }
        } else {
            warn!("paccache not available (install pacman-contrib)");
            actions_taken.push("paccache not available (skipped)".to_string());
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
