//! System cleanup auto-healing operations.

use anyhow::Result;
use tracing::{info, debug};
use super::types::{HealingLog, HealingResult};

/// Clean systemd journal if >1GB
pub async fn clean_journal_if_large(log: &mut HealingLog) -> Result<Option<String>> {
    let output = std::process::Command::new("journalctl")
        .args(["--disk-usage"])
        .output()?;

    let usage_str = String::from_utf8_lossy(&output.stdout);

    // Parse "Archived and active journals take up 1.2G in the file system."
    let size_gb: Option<f32> = usage_str
        .split_whitespace()
        .find(|s| s.ends_with('G'))
        .and_then(|s| s.trim_end_matches('G').parse().ok());

    if let Some(size) = size_gb {
        if size > 1.0 {
            info!("Journal is {:.1}GB, cleaning to last 30 days", size);

            let result = std::process::Command::new("journalctl")
                .args(["--vacuum-time=30d"])
                .output();

            match result {
                Ok(output) if output.status.success() => {
                    let message = format!("Cleaned systemd journal ({:.1}GB → kept last 30 days)", size);
                    log.record(
                        format!("Journal {:.1}GB", size),
                        "journalctl --vacuum-time=30d".to_string(),
                        HealingResult::Success(message.clone()),
                    );
                    Ok(Some(message))
                }
                _ => {
                    log.record(
                        format!("Journal {:.1}GB", size),
                        "journalctl --vacuum-time=30d".to_string(),
                        HealingResult::Failed("Failed to vacuum journal".to_string()),
                    );
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

/// Clean /tmp and /var/tmp if they're large (>2GB combined)
pub async fn clean_tmp_if_large(log: &mut HealingLog) -> Result<Option<String>> {
    let tmp_size = std::process::Command::new("du")
        .args(["-sb", "/tmp"])
        .output()
        .ok()
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<u64>().ok())
        })
        .unwrap_or(0);

    let var_tmp_size = std::process::Command::new("du")
        .args(["-sb", "/var/tmp"])
        .output()
        .ok()
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<u64>().ok())
        })
        .unwrap_or(0);

    let total_gb = (tmp_size + var_tmp_size) as f64 / (1024.0 * 1024.0 * 1024.0);

    if total_gb > 2.0 {
        info!("Temp directories are {:.1}GB, cleaning old files", total_gb);

        // Clean files older than 7 days in /tmp and /var/tmp
        let clean_tmp = std::process::Command::new("find")
            .args(["/tmp", "-type", "f", "-atime", "+7", "-delete"])
            .output();

        let clean_var_tmp = std::process::Command::new("find")
            .args(["/var/tmp", "-type", "f", "-atime", "+7", "-delete"])
            .output();

        if clean_tmp.is_ok() && clean_var_tmp.is_ok() {
            let message = format!("Cleaned temp directories ({:.1}GB, removed files older than 7 days)", total_gb);
            log.record(
                format!("Temp directories {:.1}GB", total_gb),
                "find -atime +7 -delete".to_string(),
                HealingResult::Success(message.clone()),
            );
            Ok(Some(message))
        } else {
            log.record(
                format!("Temp directories {:.1}GB", total_gb),
                "find -atime +7 -delete".to_string(),
                HealingResult::Failed("Could not clean temp directories".to_string()),
            );
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

/// Fix common permission issues
pub async fn fix_common_permissions(log: &mut HealingLog) -> Result<Option<String>> {
    // Check if user is in anna group (common issue after install)
    let username = std::env::var("USER").unwrap_or_else(|_| "root".to_string());

    let output = std::process::Command::new("groups")
        .arg(&username)
        .output()?;

    let groups = String::from_utf8_lossy(&output.stdout);

    if !groups.contains("anna") && username != "root" {
        debug!("User {} not in anna group, suggesting manual fix", username);
        // Don't auto-fix this - requires pkexec/sudo
        // This will be caught by suggestions system
        Ok(None)
    } else {
        Ok(None)
    }
}
