//! Disk setup and partitioning
//!
//! Phase 0.8: Disk management for installation
//! Citation: [archwiki:Installation_guide#Partition_the_disks]

use super::types::{DiskSetupMode, InstallConfig, StepResult};
use anyhow::Context;
use std::process::Command;
use tracing::{info, warn};

/// Setup disks according to configuration
///
/// Handles both manual and automatic partitioning modes
pub async fn setup_disks(config: &InstallConfig, dry_run: bool) -> StepResult {
    match &config.disk_setup {
        DiskSetupMode::Manual {
            root_partition,
            boot_partition,
            swap_partition,
        } => {
            setup_manual_partitions(
                root_partition,
                boot_partition,
                swap_partition.as_deref(),
                dry_run,
            )
            .await
        }
        DiskSetupMode::AutoBtrfs {
            target_disk,
            create_swap,
            swap_size_gb,
        } => setup_auto_btrfs(target_disk, *create_swap, *swap_size_gb, dry_run).await,
    }
}

/// Setup manual partitions (user-provided layout)
async fn setup_manual_partitions(
    root_partition: &str,
    boot_partition: &str,
    swap_partition: Option<&str>,
    dry_run: bool,
) -> StepResult {
    let mut actions = Vec::new();

    // Validate partitions exist
    if !dry_run {
        if !std::path::Path::new(&format!("/dev/{}", root_partition)).exists() {
            return Err(anyhow::anyhow!(
                "Root partition not found: {}",
                root_partition
            ));
        }
        if !std::path::Path::new(&format!("/dev/{}", boot_partition)).exists() {
            return Err(anyhow::anyhow!(
                "Boot partition not found: {}",
                boot_partition
            ));
        }
    }

    // Format root partition (ext4)
    let root_cmd = format!("mkfs.ext4 -F /dev/{}", root_partition);
    if dry_run {
        info!("[DRY-RUN] Would execute: {}", root_cmd);
        actions.push(format!("[dry-run] {}", root_cmd));
    } else {
        info!("Executing: {}", root_cmd);
        let output = Command::new("mkfs.ext4")
            .args(&["-F", &format!("/dev/{}", root_partition)])
            .output()
            .context("Failed to format root partition")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("mkfs.ext4 failed: {}", stderr));
        }
        actions.push(format!("formatted {} as ext4", root_partition));
    }

    // Format boot partition (FAT32)
    let boot_cmd = format!("mkfs.fat -F32 /dev/{}", boot_partition);
    if dry_run {
        info!("[DRY-RUN] Would execute: {}", boot_cmd);
        actions.push(format!("[dry-run] {}", boot_cmd));
    } else {
        info!("Executing: {}", boot_cmd);
        let output = Command::new("mkfs.fat")
            .args(&["-F32", &format!("/dev/{}", boot_partition)])
            .output()
            .context("Failed to format boot partition")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("mkfs.fat failed: {}", stderr));
        }
        actions.push(format!("formatted {} as FAT32", boot_partition));
    }

    // Setup swap if requested
    if let Some(swap) = swap_partition {
        let swap_cmd = format!("mkswap /dev/{}", swap);
        if dry_run {
            info!("[DRY-RUN] Would execute: {}", swap_cmd);
            actions.push(format!("[dry-run] {}", swap_cmd));
        } else {
            info!("Executing: {}", swap_cmd);
            let output = Command::new("mkswap")
                .arg(&format!("/dev/{}", swap))
                .output()
                .context("Failed to create swap")?;

            if !output.status.success() {
                warn!("mkswap failed, continuing without swap");
            } else {
                actions.push(format!("created swap on {}", swap));

                // Enable swap
                let _ = Command::new("swapon")
                    .arg(&format!("/dev/{}", swap))
                    .output();
                actions.push(format!("enabled swap on {}", swap));
            }
        }
    }

    // Mount partitions
    if dry_run {
        actions.push("[dry-run] mount partitions to /mnt".to_string());
    } else {
        // Mount root
        std::fs::create_dir_all("/mnt").ok();
        let output = Command::new("mount")
            .args(&[&format!("/dev/{}", root_partition), "/mnt"])
            .output()
            .context("Failed to mount root partition")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to mount root partition"));
        }
        actions.push("mounted root to /mnt".to_string());

        // Mount boot
        std::fs::create_dir_all("/mnt/boot").ok();
        let output = Command::new("mount")
            .args(&[&format!("/dev/{}", boot_partition), "/mnt/boot"])
            .output()
            .context("Failed to mount boot partition")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to mount boot partition"));
        }
        actions.push("mounted boot to /mnt/boot".to_string());
    }

    Ok(actions.join("; "))
}

/// Setup automatic btrfs partitioning
async fn setup_auto_btrfs(
    _target_disk: &str,
    _create_swap: bool,
    _swap_size_gb: u32,
    dry_run: bool,
) -> StepResult {
    // Placeholder for automatic btrfs setup
    // Full implementation would use parted/fdisk to create partitions
    // and btrfs subvolumes (@, @home, @snapshots, etc.)

    if dry_run {
        Ok("[dry-run] would create btrfs layout with subvolumes".to_string())
    } else {
        Err(anyhow::anyhow!(
            "Automatic btrfs setup not yet implemented. Use manual mode."
        ))
    }
}
