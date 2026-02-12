//! Disk-related auto-healing operations.

use anyhow::Result;
use tracing::info;
use super::types::{HealingLog, HealingResult};

/// Clean pacman cache if >5GB
pub async fn clean_pacman_cache_if_large(log: &mut HealingLog) -> Result<Option<String>> {
    let output = std::process::Command::new("du")
        .args(["-sb", "/var/cache/pacman/pkg"])
        .output()?;

    let size_str = String::from_utf8_lossy(&output.stdout);
    let size_bytes: u64 = size_str
        .split_whitespace()
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let size_gb = size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

    if size_gb > 5.0 {
        info!("Pacman cache is {:.1}GB, cleaning automatically", size_gb);

        // Keep last 3 versions with paccache
        let result = std::process::Command::new("paccache")
            .args(["-rk3"])
            .output();

        match result {
            Ok(output) if output.status.success() => {
                let message = format!("Cleaned pacman cache ({:.1}GB → kept last 3 versions)", size_gb);
                log.record(
                    format!("Pacman cache {:.1}GB", size_gb),
                    "paccache -rk3".to_string(),
                    HealingResult::Success(message.clone()),
                );
                Ok(Some(message))
            }
            Ok(_) => {
                log.record(
                    format!("Pacman cache {:.1}GB", size_gb),
                    "paccache -rk3".to_string(),
                    HealingResult::Failed("paccache command failed".to_string()),
                );
                Ok(None)
            }
            Err(e) => {
                log.record(
                    format!("Pacman cache {:.1}GB", size_gb),
                    "paccache -rk3".to_string(),
                    HealingResult::Failed(format!("paccache not available: {}", e)),
                );
                Ok(None)
            }
        }
    } else {
        Ok(None)
    }
}

/// Remove orphaned packages if >20
pub async fn remove_orphans_if_many(log: &mut HealingLog) -> Result<Option<String>> {
    let output = std::process::Command::new("pacman")
        .args(["-Qdtq"])
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let orphans = String::from_utf8_lossy(&output.stdout);
    let count = orphans.lines().filter(|l| !l.is_empty()).count();

    if count > 20 {
        info!("Found {} orphaned packages, removing automatically", count);

        // Remove orphans - this is safe, they're not dependencies anymore
        let result = std::process::Command::new("sh")
            .args(["-c", "pacman -Rns --noconfirm $(pacman -Qdtq)"])
            .output();

        match result {
            Ok(output) if output.status.success() => {
                let message = format!("Removed {} orphaned packages automatically", count);
                log.record(
                    format!("{} orphaned packages", count),
                    "pacman -Rns (orphans)".to_string(),
                    HealingResult::Success(message.clone()),
                );
                Ok(Some(message))
            }
            _ => {
                log.record(
                    format!("{} orphaned packages", count),
                    "pacman -Rns (orphans)".to_string(),
                    HealingResult::Failed("Could not remove orphans".to_string()),
                );
                Ok(None)
            }
        }
    } else {
        Ok(None)
    }
}
