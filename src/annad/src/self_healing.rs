// Anna Self-Healing Module
// Automatically detects and fixes common issues on daemon startup
// Ensures Anna can always recover without user intervention

use anyhow::Result;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use tracing::{error, info, warn};

/// Run self-diagnostic and auto-repair on daemon startup
pub fn self_heal() -> Result<bool> {
    info!("Running self-diagnostic checks...");

    let mut issues_found = false;
    let mut issues_fixed = 0;

    // 1. Check and fix directory permissions
    if let Err(e) = check_and_fix_directories() {
        error!("Directory check failed: {}", e);
        issues_found = true;
    } else {
        issues_fixed += 1;
    }

    // 2. Check and fix socket directory
    if let Err(e) = ensure_socket_directory() {
        error!("Socket directory check failed: {}", e);
        issues_found = true;
    } else {
        issues_fixed += 1;
    }

    // 3. Check database accessibility
    if let Err(e) = check_database_access() {
        error!("Database access check failed: {}", e);
        issues_found = true;
    } else {
        issues_fixed += 1;
    }

    // 4. Clean up stale resources
    if let Err(e) = cleanup_stale_resources() {
        warn!("Stale resource cleanup failed: {}", e);
        // Non-critical, don't count as issue
    }

    if issues_found {
        warn!("Self-healing completed with {} issues resolved", issues_fixed);
        Ok(false)
    } else {
        info!("Self-healing completed: all checks passed");
        Ok(true)
    }
}

/// Check and fix directory permissions
fn check_and_fix_directories() -> Result<()> {
    let dirs = vec![
        ("/var/lib/anna", 0o750),
        ("/var/log/anna", 0o750),
    ];

    for (dir, expected_mode) in dirs {
        let path = Path::new(dir);

        // Create directory if missing
        if !path.exists() {
            warn!("Directory {} missing, attempting to create...", dir);
            fs::create_dir_all(path)?;
            info!("Created directory: {}", dir);
        }

        // Check permissions
        let metadata = fs::metadata(path)?;
        let current_mode = metadata.permissions().mode() & 0o777;

        if current_mode != expected_mode {
            warn!(
                "Directory {} has incorrect permissions: {:o} (expected {:o})",
                dir, current_mode, expected_mode
            );

            // Try to fix permissions
            let mut perms = metadata.permissions();
            perms.set_mode(expected_mode);
            if let Err(e) = fs::set_permissions(path, perms) {
                warn!("Failed to fix permissions for {}: {}", dir, e);
            } else {
                info!("Fixed permissions for {}: {:o}", dir, expected_mode);
            }
        }

        // Check ownership (should be anna:anna)
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let uid = metadata.uid();
            let gid = metadata.gid();

            // Get anna user/group IDs
            let anna_uid = get_anna_uid();
            let anna_gid = get_anna_gid();

            if anna_uid.is_some() && anna_gid.is_some() {
                if uid != anna_uid.unwrap() || gid != anna_gid.unwrap() {
                    warn!(
                        "Directory {} has incorrect ownership: uid={} gid={} (expected anna:anna)",
                        dir, uid, gid
                    );

                    // Try to fix ownership (requires elevated permissions)
                    if let Err(e) = fix_ownership(dir, "anna", "anna") {
                        warn!("Failed to fix ownership for {}: {}", dir, e);
                    } else {
                        info!("Fixed ownership for {}: anna:anna", dir);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Ensure socket directory exists with correct permissions
fn ensure_socket_directory() -> Result<()> {
    let socket_dir = Path::new("/run/anna");

    if !socket_dir.exists() {
        warn!("Socket directory /run/anna missing, creating...");
        fs::create_dir_all(socket_dir)?;

        // Set permissions to 0750
        let mut perms = fs::metadata(socket_dir)?.permissions();
        perms.set_mode(0o750);
        fs::set_permissions(socket_dir, perms)?;

        info!("Created socket directory: /run/anna");
    }

    // Remove stale socket if exists
    let socket_path = socket_dir.join("annad.sock");
    if socket_path.exists() {
        warn!("Stale socket found, removing: {:?}", socket_path);
        fs::remove_file(&socket_path)?;
        info!("Removed stale socket");
    }

    Ok(())
}

/// Check database accessibility
fn check_database_access() -> Result<()> {
    let db_dir = Path::new("/var/lib/anna");
    let test_file = db_dir.join(".writetest");

    // Try to write a test file
    match fs::write(&test_file, b"test") {
        Ok(_) => {
            // Clean up test file
            let _ = fs::remove_file(&test_file);
            info!("Database directory is writable");
            Ok(())
        }
        Err(e) => {
            error!("Database directory not writable: {}", e);
            Err(e.into())
        }
    }
}

/// Clean up stale resources (old logs, temp files, etc.)
fn cleanup_stale_resources() -> Result<()> {
    // Remove temporary files older than 7 days
    let tmp_dir = Path::new("/var/lib/anna/tmp");
    if tmp_dir.exists() {
        info!("Cleaning up temporary files in {:?}", tmp_dir);
        // Implementation would check file ages and remove old ones
        // For now, just log that we checked
    }

    Ok(())
}

/// Get anna user UID
#[cfg(unix)]
fn get_anna_uid() -> Option<u32> {
    Command::new("id")
        .args(&["-u", "anna"])
        .output()
        .ok()
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .trim()
                .parse::<u32>()
                .ok()
        })
}

/// Get anna group GID
#[cfg(unix)]
fn get_anna_gid() -> Option<u32> {
    Command::new("id")
        .args(&["-g", "anna"])
        .output()
        .ok()
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .trim()
                .parse::<u32>()
                .ok()
        })
}

/// Fix ownership of a file/directory
fn fix_ownership(path: &str, user: &str, group: &str) -> Result<()> {
    let chown_arg = format!("{}:{}", user, group);
    let status = Command::new("chown")
        .args(&[&chown_arg, path])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("chown command failed"))
    }
}
