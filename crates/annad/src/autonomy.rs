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

    log.save()?;
    Ok(())
}

/// Run Tier 2 extended tasks
async fn run_tier2_tasks() -> Result<()> {
    let mut log = AutonomyLog::load().unwrap_or_default();

    // Task 4: Remove old kernels (keep 2 latest)
    if let Ok(action) = remove_old_kernels().await {
        log.record(action);
    }

    // Task 5: Clean tmp directories
    if let Ok(action) = clean_tmp_dirs().await {
        log.record(action);
    }

    log.save()?;
    Ok(())
}

/// Run Tier 3 full autonomous tasks
async fn run_tier3_tasks() -> Result<()> {
    let mut log = AutonomyLog::load().unwrap_or_default();

    // Task 6: Update mirrorlist (if old)
    if let Ok(action) = update_mirrorlist().await {
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
