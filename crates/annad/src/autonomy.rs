//! Low-level autonomy system
//!
//! Handles safe automatic maintenance tasks based on autonomy tier

use anna_common::{AutonomyAction, AutonomyLog, AutonomyTier, Config};
use anyhow::Result;
use std::process::Command;
use tracing::info;

/// Execute autonomous maintenance tasks based on tier
pub async fn run_autonomous_maintenance() -> Result<()> {
    info!("Running autonomous maintenance check");

    let config = Config::load()?;
    let tier = config.autonomy.tier;

    match tier {
        AutonomyTier::AdviseOnly => {
            info!("Tier 0 (Advise Only): No automatic actions");
            // Just monitor and report
            check_system_health().await?;
        }
        AutonomyTier::SafeAutoApply => {
            info!("Tier 1 (Safe Auto Apply): Running safe maintenance");
            // Run safe automatic tasks
            run_tier1_tasks().await?;
        }
        AutonomyTier::SemiAutonomous => {
            info!("Tier 2 (Semi-Autonomous): Running extended maintenance");
            run_tier1_tasks().await?;
            run_tier2_tasks().await?;
        }
        AutonomyTier::FullyAutonomous => {
            info!("Tier 3 (Fully Autonomous): Running full maintenance");
            run_tier1_tasks().await?;
            run_tier2_tasks().await?;
            run_tier3_tasks().await?;
        }
    }

    Ok(())
}

/// Check system health (Tier 0 - monitoring only)
async fn check_system_health() -> Result<()> {
    info!("Checking system health (monitor mode)");

    // Check orphan packages
    let orphan_count = count_orphan_packages();
    if orphan_count > 10 {
        info!("Found {} orphan packages (would clean in Tier 1+)", orphan_count);
    }

    // Check cache size
    let cache_size_gb = get_cache_size_gb();
    if cache_size_gb > 5.0 {
        info!("Package cache is {:.1}GB (would clean in Tier 1+)", cache_size_gb);
    }

    // Check log size
    let log_size_gb = get_log_size_gb();
    if log_size_gb > 1.0 {
        info!("System logs are {:.1}GB (would rotate in Tier 1+)", log_size_gb);
    }

    Ok(())
}

/// Run Tier 1 safe automatic tasks
async fn run_tier1_tasks() -> Result<()> {
    let mut log = AutonomyLog::load().unwrap_or_default();

    // Task 1: Clean orphan packages (safe - only truly orphaned packages)
    if count_orphan_packages() > 10 {
        if let Ok(action) = clean_orphan_packages().await {
            log.record(action);
        }
    }

    // Task 2: Clean package cache (safe - keeps recent versions)
    if get_cache_size_gb() > 5.0 {
        if let Ok(action) = clean_package_cache().await {
            log.record(action);
        }
    }

    // Task 3: Clean systemd journal (safe - keeps 30 days)
    if get_log_size_gb() > 1.0 {
        if let Ok(action) = clean_journal().await {
            log.record(action);
        }
    }

    // Task 4: Update package database (safe - just refreshes package info)
    if needs_package_db_update() {
        if let Ok(action) = update_package_database().await {
            log.record(action);
        }
    }

    // Task 5: Check for failed systemd services (monitoring - doesn't fix)
    if let Ok(action) = check_failed_services().await {
        if !action.output.is_empty() {
            log.record(action);
        }
    }

    // Task 6: Check disk SMART status (monitoring - alerts on disk issues)
    if let Ok(action) = check_disk_smart_status().await {
        if !action.output.is_empty() {
            log.record(action);
        }
    }

    // Task 7: Check for available updates (monitoring - doesn't install)
    if let Ok(action) = check_available_updates().await {
        if !action.output.is_empty() {
            log.record(action);
        }
    }

    log.save()?;
    Ok(())
}

/// Run Tier 2 extended tasks
async fn run_tier2_tasks() -> Result<()> {
    let mut log = AutonomyLog::load().unwrap_or_default();

    // Task 6: Remove old kernels (keep 2 latest)
    if let Ok(action) = remove_old_kernels().await {
        log.record(action);
    }

    // Task 7: Clean tmp directories
    if let Ok(action) = clean_tmp_dirs().await {
        log.record(action);
    }

    // Task 8: Clean user cache (safe - only known app caches)
    if let Ok(action) = clean_user_caches().await {
        log.record(action);
    }

    // Task 9: Remove broken symlinks from home
    if let Ok(action) = clean_broken_symlinks().await {
        log.record(action);
    }

    // Task 10: Optimize pacman database
    if let Ok(action) = optimize_pacman_db().await {
        log.record(action);
    }

    // Task 11: Clean old coredumps
    if let Ok(action) = clean_coredumps().await {
        log.record(action);
    }

    // Task 12: Clean development tool caches (pip, cargo, npm)
    if let Ok(action) = clean_dev_caches().await {
        log.record(action);
    }

    // Task 13: Optimize btrfs filesystems if detected
    if let Ok(action) = optimize_btrfs().await {
        log.record(action);
    }

    log.save()?;
    Ok(())
}

/// Run Tier 3 full autonomous tasks
async fn run_tier3_tasks() -> Result<()> {
    let mut log = AutonomyLog::load().unwrap_or_default();

    // Task 14: Update mirrorlist (if old)
    if let Ok(action) = update_mirrorlist().await {
        log.record(action);
    }

    // Task 15: Apply security updates (only packages marked as security-related)
    if let Ok(action) = apply_security_updates().await {
        log.record(action);
    }

    // Task 16: Backup important configs
    if let Ok(action) = backup_system_configs().await {
        log.record(action);
    }

    // Task 17: Rebuild font cache if stale
    if let Ok(action) = rebuild_font_cache().await {
        log.record(action);
    }

    // Task 18: Update AUR packages if AUR helper detected
    if let Ok(action) = update_aur_packages().await {
        log.record(action);
    }

    // Task 19: Auto-update Anna itself (Tier 3 only)
    if let Ok(action) = auto_update_anna().await {
        if action.success {
            log.record(action);
        }
    }

    log.save()?;
    Ok(())
}

/// Count orphan packages
fn count_orphan_packages() -> usize {
    if let Ok(output) = Command::new("pacman").args(&["-Qtdq"]).output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.lines().count();
        }
    }
    0
}

/// Get package cache size in GB
fn get_cache_size_gb() -> f64 {
    if let Ok(output) = Command::new("du")
        .args(&["-sb", "/var/cache/pacman/pkg"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(size_str) = stdout.split_whitespace().next() {
            if let Ok(bytes) = size_str.parse::<f64>() {
                return bytes / 1024.0 / 1024.0 / 1024.0;
            }
        }
    }
    0.0
}

/// Get log size in GB
fn get_log_size_gb() -> f64 {
    if let Ok(output) = Command::new("du")
        .args(&["-sb", "/var/log/journal"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(size_str) = stdout.split_whitespace().next() {
            if let Ok(bytes) = size_str.parse::<f64>() {
                return bytes / 1024.0 / 1024.0 / 1024.0;
            }
        }
    }
    0.0
}

/// Clean orphan packages
async fn clean_orphan_packages() -> Result<AutonomyAction> {
    info!("Cleaning orphan packages");

    let start_time = chrono::Utc::now();
    let command = "pacman -Qtdq | pacman -Rns --noconfirm -";

    let output = Command::new("sh")
        .args(&["-c", command])
        .output()?;

    let success = output.status.success();
    let output_str = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(AutonomyAction {
        action_type: "clean_orphans".to_string(),
        executed_at: start_time,
        description: "Removed orphaned packages that are no longer needed".to_string(),
        command_run: command.to_string(),
        success,
        output: output_str,
        can_undo: false, // Can't easily reinstall orphans
        undo_command: None,
    })
}

/// Clean package cache
async fn clean_package_cache() -> Result<AutonomyAction> {
    info!("Cleaning package cache");

    let start_time = chrono::Utc::now();
    let command = "paccache -r -k 3"; // Keep 3 most recent versions

    let output = Command::new("sh")
        .args(&["-c", command])
        .output()?;

    let success = output.status.success();
    let output_str = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(AutonomyAction {
        action_type: "clean_cache".to_string(),
        executed_at: start_time,
        description: "Cleaned package cache, keeping 3 most recent versions".to_string(),
        command_run: command.to_string(),
        success,
        output: output_str,
        can_undo: false,
        undo_command: None,
    })
}

/// Clean systemd journal
async fn clean_journal() -> Result<AutonomyAction> {
    info!("Cleaning systemd journal");

    let start_time = chrono::Utc::now();
    let command = "journalctl --vacuum-time=30d"; // Keep 30 days

    let output = Command::new("sh")
        .args(&["-c", command])
        .output()?;

    let success = output.status.success();
    let output_str = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(AutonomyAction {
        action_type: "clean_journal".to_string(),
        executed_at: start_time,
        description: "Cleaned systemd journal, keeping last 30 days".to_string(),
        command_run: command.to_string(),
        success,
        output: output_str,
        can_undo: false,
        undo_command: None,
    })
}

/// Remove old kernels
async fn remove_old_kernels() -> Result<AutonomyAction> {
    info!("Removing old kernels");

    let start_time = chrono::Utc::now();

    // Get list of installed kernels
    let output = Command::new("pacman")
        .args(&["-Q"])
        .output()?;

    let installed = String::from_utf8_lossy(&output.stdout);
    let mut kernels: Vec<String> = installed
        .lines()
        .filter(|line| line.starts_with("linux ") || line.starts_with("linux-"))
        .map(|line| line.split_whitespace().next().unwrap_or("").to_string())
        .collect();

    // Keep current kernel and one previous
    if kernels.len() <= 2 {
        return Ok(AutonomyAction {
            action_type: "remove_old_kernels".to_string(),
            executed_at: start_time,
            description: "No old kernels to remove".to_string(),
            command_run: "none".to_string(),
            success: true,
            output: "Keeping all installed kernels".to_string(),
            can_undo: false,
            undo_command: None,
        });
    }

    kernels.sort();
    kernels.reverse(); // Newest first
    let to_remove: Vec<String> = kernels.iter().skip(2).cloned().collect();

    let command = format!("pacman -R --noconfirm {}", to_remove.join(" "));

    let output = Command::new("sh")
        .args(&["-c", &command])
        .output()?;

    let success = output.status.success();
    let output_str = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(AutonomyAction {
        action_type: "remove_old_kernels".to_string(),
        executed_at: start_time,
        description: format!("Removed {} old kernel(s)", to_remove.len()),
        command_run: command,
        success,
        output: output_str,
        can_undo: true,
        undo_command: Some(format!("pacman -S {}", to_remove.join(" "))),
    })
}

/// Clean tmp directories
async fn clean_tmp_dirs() -> Result<AutonomyAction> {
    info!("Cleaning tmp directories");

    let start_time = chrono::Utc::now();
    let command = "find /tmp -type f -atime +7 -delete"; // Delete files not accessed in 7 days

    let output = Command::new("sh")
        .args(&["-c", command])
        .output()?;

    let success = output.status.success();
    let output_str = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(AutonomyAction {
        action_type: "clean_tmp".to_string(),
        executed_at: start_time,
        description: "Cleaned /tmp of files older than 7 days".to_string(),
        command_run: command.to_string(),
        success,
        output: output_str,
        can_undo: false,
        undo_command: None,
    })
}

/// Update mirrorlist
async fn update_mirrorlist() -> Result<AutonomyAction> {
    info!("Updating mirrorlist");

    let start_time = chrono::Utc::now();

    // Check if reflector is installed
    if !Command::new("which").arg("reflector").output()?.status.success() {
        return Ok(AutonomyAction {
            action_type: "update_mirrorlist".to_string(),
            executed_at: start_time,
            description: "Skipped: reflector not installed".to_string(),
            command_run: "none".to_string(),
            success: true,
            output: "reflector not available".to_string(),
            can_undo: false,
            undo_command: None,
        });
    }

    let command = "reflector --latest 20 --protocol https --sort rate --save /etc/pacman.d/mirrorlist";

    let output = Command::new("sh")
        .args(&["-c", command])
        .output()?;

    let success = output.status.success();
    let output_str = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(AutonomyAction {
        action_type: "update_mirrorlist".to_string(),
        executed_at: start_time,
        description: "Updated mirrorlist with fastest 20 HTTPS mirrors".to_string(),
        command_run: command.to_string(),
        success,
        output: output_str,
        can_undo: false,
        undo_command: None,
    })
}

/// Get recent autonomous actions
#[allow(dead_code)]
pub fn get_recent_actions(count: usize) -> Vec<AutonomyAction> {
    if let Ok(log) = AutonomyLog::load() {
        return log.recent(count).into_iter().cloned().collect();
    }
    Vec::new()
}

/// Check if package database needs update (older than 1 day)
fn needs_package_db_update() -> bool {
    if let Ok(metadata) = std::fs::metadata("/var/lib/pacman/sync/core.db") {
        if let Ok(modified) = metadata.modified() {
            if let Ok(elapsed) = modified.elapsed() {
                return elapsed.as_secs() > 86400; // 1 day
            }
        }
    }
    true // If we can't check, assume it needs update
}

/// Update package database (pacman -Sy)
async fn update_package_database() -> Result<AutonomyAction> {
    info!("Updating package database");

    let start_time = chrono::Utc::now();
    let command = "pacman -Sy --noconfirm";

    let output = Command::new("sh")
        .args(&["-c", command])
        .output()?;

    let success = output.status.success();
    let output_str = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(AutonomyAction {
        action_type: "update_package_db".to_string(),
        executed_at: start_time,
        description: "Updated package database to get latest package information".to_string(),
        command_run: command.to_string(),
        success,
        output: output_str,
        can_undo: false,
        undo_command: None,
    })
}

/// Check for failed systemd services
async fn check_failed_services() -> Result<AutonomyAction> {
    info!("Checking for failed systemd services");

    let start_time = chrono::Utc::now();
    let command = "systemctl --failed --no-pager";

    let output = Command::new("sh")
        .args(&["-c", command])
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout).to_string();

    // Check if there are actually failed services
    let has_failures = output_str.lines().count() > 2 && !output_str.contains("0 loaded units");

    Ok(AutonomyAction {
        action_type: "check_failed_services".to_string(),
        executed_at: start_time,
        description: if has_failures {
            "Found failed systemd services - user attention needed".to_string()
        } else {
            "No failed systemd services".to_string()
        },
        command_run: command.to_string(),
        success: true,
        output: if has_failures { output_str } else { String::new() },
        can_undo: false,
        undo_command: None,
    })
}

/// Clean user cache directories (safe apps only)
async fn clean_user_caches() -> Result<AutonomyAction> {
    info!("Cleaning user cache directories");

    let start_time = chrono::Utc::now();
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());

    // List of safe caches to clean
    let cache_dirs = vec![
        format!("{}/.cache/thumbnails", home),
        format!("{}/.cache/mozilla/firefox/*/cache2", home),
        format!("{}/.cache/chromium/Default/Cache", home),
        format!("{}/.cache/yarn", home),
        format!("{}/.cache/npm", home),
    ];

    let mut cleaned_count = 0;
    let mut total_freed = 0u64;

    for cache_dir in cache_dirs {
        // Get size before cleaning
        if let Ok(output) = Command::new("du").args(&["-sb", &cache_dir]).output() {
            if let Some(size_str) = String::from_utf8_lossy(&output.stdout).split_whitespace().next() {
                if let Ok(size) = size_str.parse::<u64>() {
                    total_freed += size;
                }
            }
        }

        // Clean the cache
        if let Ok(_) = Command::new("find")
            .args(&[&cache_dir, "-type", "f", "-delete"])
            .output()
        {
            cleaned_count += 1;
        }
    }

    let freed_mb = total_freed as f64 / 1024.0 / 1024.0;

    Ok(AutonomyAction {
        action_type: "clean_user_caches".to_string(),
        executed_at: start_time,
        description: format!("Cleaned {} user cache directories, freed {:.1}MB", cleaned_count, freed_mb),
        command_run: "find ~/.cache/... -type f -delete".to_string(),
        success: true,
        output: format!("Cleaned caches: {}\nFreed: {:.1}MB", cleaned_count, freed_mb),
        can_undo: false,
        undo_command: None,
    })
}

/// Remove broken symlinks from home directory
async fn clean_broken_symlinks() -> Result<AutonomyAction> {
    info!("Cleaning broken symlinks");

    let start_time = chrono::Utc::now();
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let command = format!("find {} -maxdepth 3 -xtype l -delete", home);

    let output = Command::new("sh")
        .args(&["-c", &command])
        .output()?;

    let success = output.status.success();

    Ok(AutonomyAction {
        action_type: "clean_broken_symlinks".to_string(),
        executed_at: start_time,
        description: "Removed broken symbolic links from home directory".to_string(),
        command_run: command,
        success,
        output: "Broken symlinks removed".to_string(),
        can_undo: false,
        undo_command: None,
    })
}

/// Optimize pacman database
async fn optimize_pacman_db() -> Result<AutonomyAction> {
    info!("Optimizing pacman database");

    let start_time = chrono::Utc::now();

    // Check if pacman-optimize exists
    if !Command::new("which").arg("pacman-optimize").output()?.status.success() {
        return Ok(AutonomyAction {
            action_type: "optimize_pacman_db".to_string(),
            executed_at: start_time,
            description: "Skipped: pacman-optimize not available".to_string(),
            command_run: "none".to_string(),
            success: true,
            output: "pacman-optimize not installed".to_string(),
            can_undo: false,
            undo_command: None,
        });
    }

    let command = "pacman-optimize && sync";

    let output = Command::new("sh")
        .args(&["-c", command])
        .output()?;

    let success = output.status.success();
    let output_str = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(AutonomyAction {
        action_type: "optimize_pacman_db".to_string(),
        executed_at: start_time,
        description: "Optimized pacman database for better performance".to_string(),
        command_run: command.to_string(),
        success,
        output: output_str,
        can_undo: false,
        undo_command: None,
    })
}

/// Apply security updates (only security-related packages)
async fn apply_security_updates() -> Result<AutonomyAction> {
    info!("Checking for security updates");

    let start_time = chrono::Utc::now();

    // Get list of updates available
    let check_output = Command::new("pacman")
        .args(&["-Qu"])
        .output()?;

    let updates_str = String::from_utf8_lossy(&check_output.stdout);

    // Look for security-related packages (linux kernel, glibc, openssl, systemd, etc.)
    let security_packages = vec!["linux", "glibc", "openssl", "systemd", "sudo", "openssh"];
    let mut security_updates = Vec::new();

    for line in updates_str.lines() {
        let pkg_name = line.split_whitespace().next().unwrap_or("");
        if security_packages.iter().any(|sp| pkg_name.starts_with(sp)) {
            security_updates.push(pkg_name.to_string());
        }
    }

    if security_updates.is_empty() {
        return Ok(AutonomyAction {
            action_type: "apply_security_updates".to_string(),
            executed_at: start_time,
            description: "No security updates available".to_string(),
            command_run: "none".to_string(),
            success: true,
            output: "System is up to date".to_string(),
            can_undo: false,
            undo_command: None,
        });
    }

    // Apply the security updates
    let command = format!("pacman -S --noconfirm {}", security_updates.join(" "));

    let output = Command::new("sh")
        .args(&["-c", &command])
        .output()?;

    let success = output.status.success();
    let output_str = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(AutonomyAction {
        action_type: "apply_security_updates".to_string(),
        executed_at: start_time,
        description: format!("Applied {} security update(s)", security_updates.len()),
        command_run: command,
        success,
        output: output_str,
        can_undo: false,
        undo_command: None,
    })
}

/// Backup important system configs
async fn backup_system_configs() -> Result<AutonomyAction> {
    info!("Backing up system configs");

    let start_time = chrono::Utc::now();
    let backup_dir = "/var/lib/anna/backups";
    let timestamp = start_time.format("%Y%m%d_%H%M%S");
    let backup_path = format!("{}/config_{}", backup_dir, timestamp);

    // Create backup directory
    std::fs::create_dir_all(&backup_path)?;

    // List of important configs to backup
    let configs = vec![
        "/etc/pacman.conf",
        "/etc/makepkg.conf",
        "/etc/fstab",
        "/etc/mkinitcpio.conf",
        "/etc/systemd/system",
    ];

    let mut backed_up = Vec::new();

    for config in configs {
        if std::path::Path::new(config).exists() {
            let filename = config.replace("/", "_");
            let dest = format!("{}/{}", backup_path, filename);

            if let Ok(_) = Command::new("cp")
                .args(&["-r", config, &dest])
                .output()
            {
                backed_up.push(config.to_string());
            }
        }
    }

    Ok(AutonomyAction {
        action_type: "backup_configs".to_string(),
        executed_at: start_time,
        description: format!("Backed up {} config files to {}", backed_up.len(), backup_path),
        command_run: format!("cp -r /etc/* {}", backup_path),
        success: true,
        output: format!("Backup location: {}\nFiles backed up: {:?}", backup_path, backed_up),
        can_undo: false,
        undo_command: None,
    })
}

/// Check disk SMART status for health warnings
async fn check_disk_smart_status() -> Result<AutonomyAction> {
    info!("Checking disk SMART status");

    let start_time = chrono::Utc::now();

    // Check if smartctl is installed
    if !Command::new("which").arg("smartctl").output()?.status.success() {
        return Ok(AutonomyAction {
            action_type: "check_smart_status".to_string(),
            executed_at: start_time,
            description: "Skipped: smartmontools not installed".to_string(),
            command_run: "none".to_string(),
            success: true,
            output: String::new(),
            can_undo: false,
            undo_command: None,
        });
    }

    // Get list of disks
    let output = Command::new("lsblk")
        .args(&["-d", "-n", "-o", "NAME,TYPE"])
        .output()?;

    let disk_list = String::from_utf8_lossy(&output.stdout);
    let mut warnings = Vec::new();

    for line in disk_list.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[1] == "disk" {
            let disk = parts[0];
            let device = format!("/dev/{}", disk);

            // Check SMART health
            if let Ok(smart_output) = Command::new("smartctl")
                .args(&["-H", &device])
                .output()
            {
                let smart_str = String::from_utf8_lossy(&smart_output.stdout);
                if smart_str.contains("FAILED") || smart_str.contains("WARN") {
                    warnings.push(format!("{}: SMART health check failed", disk));
                }
            }
        }
    }

    let output_str = if warnings.is_empty() {
        String::new()
    } else {
        warnings.join("\n")
    };

    Ok(AutonomyAction {
        action_type: "check_smart_status".to_string(),
        executed_at: start_time,
        description: if warnings.is_empty() {
            "All disks healthy".to_string()
        } else {
            format!("⚠ Disk health warnings detected: {}", warnings.len())
        },
        command_run: "smartctl -H /dev/*".to_string(),
        success: true,
        output: output_str,
        can_undo: false,
        undo_command: None,
    })
}

/// Check for available updates (monitoring only)
async fn check_available_updates() -> Result<AutonomyAction> {
    info!("Checking for available updates");

    let start_time = chrono::Utc::now();
    let command = "pacman -Qu";

    let output = Command::new("sh")
        .args(&["-c", command])
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout).to_string();
    let update_count = output_str.lines().count();

    Ok(AutonomyAction {
        action_type: "check_updates".to_string(),
        executed_at: start_time,
        description: if update_count > 0 {
            format!("ℹ {} package update(s) available", update_count)
        } else {
            "System is up to date".to_string()
        },
        command_run: command.to_string(),
        success: true,
        output: if update_count > 0 && update_count <= 20 {
            output_str
        } else if update_count > 20 {
            format!("{} updates available (run 'pacman -Qu' to see all)", update_count)
        } else {
            String::new()
        },
        can_undo: false,
        undo_command: None,
    })
}

/// Clean old coredumps
async fn clean_coredumps() -> Result<AutonomyAction> {
    info!("Cleaning old coredumps");

    let start_time = chrono::Utc::now();
    let coredump_dir = "/var/lib/systemd/coredump";

    if !std::path::Path::new(coredump_dir).exists() {
        return Ok(AutonomyAction {
            action_type: "clean_coredumps".to_string(),
            executed_at: start_time,
            description: "No coredump directory found".to_string(),
            command_run: "none".to_string(),
            success: true,
            output: String::new(),
            can_undo: false,
            undo_command: None,
        });
    }

    // Remove coredumps older than 7 days
    let command = format!("find {} -type f -mtime +7 -delete", coredump_dir);

    let output = Command::new("sh")
        .args(&["-c", &command])
        .output()?;

    let success = output.status.success();

    Ok(AutonomyAction {
        action_type: "clean_coredumps".to_string(),
        executed_at: start_time,
        description: "Removed coredumps older than 7 days".to_string(),
        command_run: command,
        success,
        output: "Old coredumps cleaned".to_string(),
        can_undo: false,
        undo_command: None,
    })
}

/// Clean development tool caches (pip, cargo, npm)
async fn clean_dev_caches() -> Result<AutonomyAction> {
    info!("Cleaning development tool caches");

    let start_time = chrono::Utc::now();
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());

    let mut cleaned = Vec::new();
    let mut total_freed = 0u64;

    // Clean pip cache
    let pip_cache = format!("{}/.cache/pip", home);
    if std::path::Path::new(&pip_cache).exists() {
        if let Ok(output) = Command::new("du").args(&["-sb", &pip_cache]).output() {
            if let Some(size_str) = String::from_utf8_lossy(&output.stdout).split_whitespace().next() {
                if let Ok(size) = size_str.parse::<u64>() {
                    total_freed += size;
                }
            }
        }
        let _ = Command::new("rm").args(&["-rf", &pip_cache]).output();
        cleaned.push("pip");
    }

    // Clean cargo cache (registry and git, but keep binaries)
    let cargo_registry = format!("{}/.cargo/registry/cache", home);
    if std::path::Path::new(&cargo_registry).exists() {
        if let Ok(output) = Command::new("du").args(&["-sb", &cargo_registry]).output() {
            if let Some(size_str) = String::from_utf8_lossy(&output.stdout).split_whitespace().next() {
                if let Ok(size) = size_str.parse::<u64>() {
                    total_freed += size;
                }
            }
        }
        let _ = Command::new("rm").args(&["-rf", &cargo_registry]).output();
        cleaned.push("cargo");
    }

    // Clean npm cache
    if Command::new("which").arg("npm").output()?.status.success() {
        let npm_output = Command::new("npm")
            .args(&["cache", "clean", "--force"])
            .output();
        if npm_output.is_ok() {
            cleaned.push("npm");
        }
    }

    let freed_mb = total_freed as f64 / 1024.0 / 1024.0;

    Ok(AutonomyAction {
        action_type: "clean_dev_caches".to_string(),
        executed_at: start_time,
        description: if cleaned.is_empty() {
            "No development caches to clean".to_string()
        } else {
            format!("Cleaned {} tool caches, freed {:.1}MB", cleaned.len(), freed_mb)
        },
        command_run: "rm -rf ~/.cache/pip ~/.cargo/registry/cache && npm cache clean".to_string(),
        success: true,
        output: format!("Cleaned: {}\nFreed: {:.1}MB", cleaned.join(", "), freed_mb),
        can_undo: false,
        undo_command: None,
    })
}

/// Optimize btrfs filesystems if detected
async fn optimize_btrfs() -> Result<AutonomyAction> {
    info!("Optimizing btrfs filesystems");

    let start_time = chrono::Utc::now();

    // Check for btrfs filesystems
    let output = Command::new("findmnt")
        .args(&["-t", "btrfs", "-n", "-o", "TARGET"])
        .output()?;

    let btrfs_mounts = String::from_utf8_lossy(&output.stdout);
    let mount_points: Vec<&str> = btrfs_mounts.lines().collect();

    if mount_points.is_empty() {
        return Ok(AutonomyAction {
            action_type: "optimize_btrfs".to_string(),
            executed_at: start_time,
            description: "No btrfs filesystems detected".to_string(),
            command_run: "none".to_string(),
            success: true,
            output: String::new(),
            can_undo: false,
            undo_command: None,
        });
    }

    let mut optimized = Vec::new();

    for mount in mount_points {
        // Run balance on the filesystem (limited to avoid long operations)
        let balance_cmd = format!("btrfs balance start -dusage=50 -musage=50 {}", mount);
        if let Ok(_) = Command::new("sh").args(&["-c", &balance_cmd]).output() {
            optimized.push(mount.to_string());
        }
    }

    Ok(AutonomyAction {
        action_type: "optimize_btrfs".to_string(),
        executed_at: start_time,
        description: format!("Optimized {} btrfs filesystem(s)", optimized.len()),
        command_run: "btrfs balance start -dusage=50 -musage=50".to_string(),
        success: true,
        output: format!("Optimized: {:?}", optimized),
        can_undo: false,
        undo_command: None,
    })
}

/// Rebuild font cache if stale
async fn rebuild_font_cache() -> Result<AutonomyAction> {
    info!("Checking font cache");

    let start_time = chrono::Utc::now();
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let font_cache = format!("{}/.cache/fontconfig", home);

    // Check if font cache exists and is older than 30 days
    let needs_rebuild = if let Ok(metadata) = std::fs::metadata(&font_cache) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(elapsed) = modified.elapsed() {
                elapsed.as_secs() > 2592000 // 30 days
            } else {
                false
            }
        } else {
            true
        }
    } else {
        true
    };

    if !needs_rebuild {
        return Ok(AutonomyAction {
            action_type: "rebuild_font_cache".to_string(),
            executed_at: start_time,
            description: "Font cache is fresh".to_string(),
            command_run: "none".to_string(),
            success: true,
            output: String::new(),
            can_undo: false,
            undo_command: None,
        });
    }

    let command = "fc-cache -f -v";

    let output = Command::new("sh")
        .args(&["-c", command])
        .output()?;

    let success = output.status.success();

    Ok(AutonomyAction {
        action_type: "rebuild_font_cache".to_string(),
        executed_at: start_time,
        description: "Rebuilt font cache".to_string(),
        command_run: command.to_string(),
        success,
        output: "Font cache rebuilt successfully".to_string(),
        can_undo: false,
        undo_command: None,
    })
}

/// Update AUR packages if AUR helper detected
async fn update_aur_packages() -> Result<AutonomyAction> {
    info!("Checking for AUR updates");

    let start_time = chrono::Utc::now();

    // Detect AUR helper
    let helpers = vec!["yay", "paru", "trizen", "pikaur"];
    let mut detected_helper: Option<String> = None;

    for helper in helpers {
        if Command::new("which").arg(helper).output()?.status.success() {
            detected_helper = Some(helper.to_string());
            break;
        }
    }

    if detected_helper.is_none() {
        return Ok(AutonomyAction {
            action_type: "update_aur_packages".to_string(),
            executed_at: start_time,
            description: "No AUR helper detected".to_string(),
            command_run: "none".to_string(),
            success: true,
            output: String::new(),
            can_undo: false,
            undo_command: None,
        });
    }

    let helper = detected_helper.unwrap();
    let command = format!("{} -Syu --noconfirm --aur", helper);

    let output = Command::new("sh")
        .args(&["-c", &command])
        .output()?;

    let success = output.status.success();
    let output_str = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(AutonomyAction {
        action_type: "update_aur_packages".to_string(),
        executed_at: start_time,
        description: format!("Updated AUR packages using {}", helper),
        command_run: command,
        success,
        output: output_str,
        can_undo: false,
        undo_command: None,
    })
}

/// Auto-update Anna herself (Tier 3 only)
async fn auto_update_anna() -> Result<AutonomyAction> {
    info!("Checking for Anna updates");

    let start_time = chrono::Utc::now();

    // Check for updates
    match anna_common::updater::check_for_updates().await {
        Ok(update_info) => {
            if !update_info.is_update_available {
                return Ok(AutonomyAction {
                    action_type: "auto_update_anna".to_string(),
                    executed_at: start_time,
                    description: "Already on latest version".to_string(),
                    command_run: "check_for_updates".to_string(),
                    success: true,
                    output: format!("Current version: {}", update_info.current_version),
                    can_undo: false,
                    undo_command: None,
                });
            }

            info!("Update available: {} -> {}", update_info.current_version, update_info.latest_version);

            // Perform the update
            match anna_common::updater::perform_update(&update_info).await {
                Ok(()) => {
                    info!("Auto-update successful: {}", update_info.latest_version);

                    // Send notification
                    let _ = Command::new("notify-send")
                        .arg("--app-name=Anna Assistant")
                        .arg("--icon=system-software-update")
                        .arg("Anna Updated Automatically")
                        .arg(&format!("Updated to {} in the background", update_info.latest_version))
                        .spawn();

                    Ok(AutonomyAction {
                        action_type: "auto_update_anna".to_string(),
                        executed_at: start_time,
                        description: format!("Auto-updated Anna from {} to {}",
                            update_info.current_version, update_info.latest_version),
                        command_run: "perform_update".to_string(),
                        success: true,
                        output: format!("Updated to {}\nDaemon restarted", update_info.latest_version),
                        can_undo: false,
                        undo_command: None,
                    })
                }
                Err(e) => {
                    info!("Auto-update failed: {}", e);
                    Ok(AutonomyAction {
                        action_type: "auto_update_anna".to_string(),
                        executed_at: start_time,
                        description: "Auto-update failed".to_string(),
                        command_run: "perform_update".to_string(),
                        success: false,
                        output: format!("Error: {}", e),
                        can_undo: false,
                        undo_command: None,
                    })
                }
            }
        }
        Err(e) => {
            Ok(AutonomyAction {
                action_type: "auto_update_anna".to_string(),
                executed_at: start_time,
                description: "Could not check for updates".to_string(),
                command_run: "check_for_updates".to_string(),
                success: false,
                output: format!("Error: {}", e),
                can_undo: false,
                undo_command: None,
            })
        }
    }
}
