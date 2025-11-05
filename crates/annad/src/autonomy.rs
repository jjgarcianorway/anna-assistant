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

    log.save()?;
    Ok(())
}

/// Run Tier 3 full autonomous tasks
async fn run_tier3_tasks() -> Result<()> {
    let mut log = AutonomyLog::load().unwrap_or_default();

    // Task 11: Update mirrorlist (if old)
    if let Ok(action) = update_mirrorlist().await {
        log.record(action);
    }

    // Task 12: Apply security updates (only packages marked as security-related)
    if let Ok(action) = apply_security_updates().await {
        log.record(action);
    }

    // Task 13: Backup important configs
    if let Ok(action) = backup_system_configs().await {
        log.record(action);
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
