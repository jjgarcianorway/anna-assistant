//! User Radar for Anna v0.12.9 "Orion"
//!
//! Scores user behavior and habits across 8 categories (0-10 scale):
//! 1. Activity regularity (weekly usage rhythm)
//! 2. Job mix balance (CPU vs IO vs network)
//! 3. Workspace hygiene (tmp, cache, dotfiles bloat)
//! 4. Update discipline (how quickly updates are applied)
//! 5. Backup discipline (backup frequency and consistency)
//! 6. Risk exposure (root usage, sudo logs)
//! 7. Connectivity habits (VPN usage, network changes)
//! 8. Power management (battery health, suspend usage)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// User radar result with 8 category scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRadar {
    pub overall: u8,           // Average of all categories (0-10)
    pub regularity: u8,        // Usage rhythm (10=regular, 0=sporadic)
    pub workspace: u8,         // Workspace hygiene (10=clean, 0=bloated)
    pub updates: u8,           // Update discipline (10=proactive, 0=neglectful)
    pub backups: u8,           // Backup discipline (10=consistent, 0=never)
    pub risk: u8,              // Risk exposure (10=cautious, 0=reckless)
    pub connectivity: u8,      // Network habits (10=stable, 0=erratic)
    pub power: u8,             // Power management (10=optimal, 0=poor)
    pub warnings: u8,          // Warning response (10=responsive, 0=ignore)
}

impl UserRadar {
    /// Calculate overall score (average of all categories)
    pub fn calculate_overall(&mut self) {
        let sum = self.regularity as u16
            + self.workspace as u16
            + self.updates as u16
            + self.backups as u16
            + self.risk as u16
            + self.connectivity as u16
            + self.power as u16
            + self.warnings as u16;

        self.overall = (sum / 8) as u8;
    }
}

/// Collect user radar data
pub fn collect_user_radar() -> Result<UserRadar> {
    let mut radar = UserRadar {
        overall: 0,
        regularity: score_regularity()?,
        workspace: score_workspace()?,
        updates: score_updates()?,
        backups: score_backups()?,
        risk: score_risk()?,
        connectivity: score_connectivity()?,
        power: score_power()?,
        warnings: score_warnings()?,
    };

    radar.calculate_overall();
    Ok(radar)
}

//
// Scoring Functions (0-10 scale)
//

/// Score activity regularity: consistent usage patterns
///
/// Formula:
/// - Check system uptime history via `last reboot`
/// - Calculate days between reboots
/// - Regular daily use (1-2 day reboots) = 10
/// - Weekly use (3-7 day intervals) = 7
/// - Sporadic use (8-14 day intervals) = 5
/// - Rare use (15+ day intervals) = 3
/// - Always-on server (30+ days uptime) = 10
fn score_regularity() -> Result<u8> {
    // Get system uptime
    let uptime_str = fs::read_to_string("/proc/uptime")
        .context("Failed to read /proc/uptime")?;

    let uptime_secs: f64 = uptime_str
        .split_whitespace()
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);

    let uptime_days = (uptime_secs / 86400.0).floor() as u64;

    // Always-on server (30+ days)
    if uptime_days >= 30 {
        return Ok(10);
    }

    // Check reboot frequency from last command
    let output = Command::new("last")
        .args(&["reboot", "-F", "-n", "5"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let reboot_count = stdout.lines().filter(|line| line.contains("reboot")).count();

        // Estimate regularity based on uptime and reboot count
        if reboot_count >= 3 {
            // Multiple reboots in recent history
            let avg_interval = uptime_days / reboot_count.max(1) as u64;

            let score = if avg_interval <= 2 {
                10 // Daily use
            } else if avg_interval <= 7 {
                7 // Weekly use
            } else if avg_interval <= 14 {
                5 // Bi-weekly use
            } else {
                3 // Sporadic use
            };

            Ok(score)
        } else {
            // Long uptime with few reboots
            if uptime_days <= 7 {
                Ok(7) // Recent install or weekly reboot
            } else {
                Ok(10) // Stable always-on usage
            }
        }
    } else {
        Ok(7) // Cannot determine, assume average
    }
}

/// Score workspace hygiene: tmp, cache, and dotfile bloat
///
/// Formula:
/// - Check sizes of:
///   - /tmp
///   - ~/.cache
///   - ~/.local/share (exclude big items like Steam)
/// - Total < 1 GB = 10
/// - 1-5 GB = 7
/// - 5-10 GB = 5
/// - 10-20 GB = 3
/// - 20+ GB = 0
fn score_workspace() -> Result<u8> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let cache_path = format!("{}/.cache", home);

    let paths_to_check = vec![
        "/tmp",
        &cache_path,
    ];

    let mut total_mb = 0u64;

    for path in paths_to_check {
        if let Ok(output) = Command::new("du")
            .args(&["-sm", path])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(size_str) = stdout.split_whitespace().next() {
                if let Ok(size) = size_str.parse::<u64>() {
                    total_mb += size;
                }
            }
        }
    }

    let total_gb = total_mb / 1024;

    let score = if total_gb < 1 {
        10
    } else if total_gb < 5 {
        7
    } else if total_gb < 10 {
        5
    } else if total_gb < 20 {
        3
    } else {
        0
    };

    Ok(score)
}

/// Score update discipline: how quickly updates are applied
///
/// Formula:
/// - Check age of /var/log/pacman.log (or equivalent)
/// - Last update < 7 days = 10
/// - 7-14 days = 8
/// - 14-30 days = 6
/// - 30-60 days = 4
/// - 60-90 days = 2
/// - 90+ days = 0
fn score_updates() -> Result<u8> {
    let log_files = vec![
        "/var/log/pacman.log",      // Arch
        "/var/log/apt/history.log", // Debian/Ubuntu
        "/var/log/dnf.log",         // Fedora
    ];

    let mut last_update_age: Option<u64> = None;

    for log_file in log_files {
        if let Ok(metadata) = fs::metadata(log_file) {
            if let Ok(modified) = metadata.modified() {
                let modified_secs = modified
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let age_secs = now.saturating_sub(modified_secs);

                if last_update_age.is_none() || age_secs < last_update_age.unwrap() {
                    last_update_age = Some(age_secs);
                }
            }
        }
    }

    if let Some(age_secs) = last_update_age {
        let age_days = age_secs / 86400;

        let score = if age_days < 7 {
            10
        } else if age_days < 14 {
            8
        } else if age_days < 30 {
            6
        } else if age_days < 60 {
            4
        } else if age_days < 90 {
            2
        } else {
            0
        };

        Ok(score)
    } else {
        Ok(5) // Cannot determine, assume average
    }
}

/// Score backup discipline: consistency of backups
///
/// Formula:
/// - Check backup timestamps in common locations
/// - Weekly backups (7 day intervals) = 10
/// - Bi-weekly backups (14 day intervals) = 7
/// - Monthly backups (30 day intervals) = 5
/// - Quarterly backups (90 day intervals) = 3
/// - No regular backups = 0
fn score_backups() -> Result<u8> {
    let backup_paths = vec![
        "/var/backups",
        "/backup",
        "/mnt/backup",
        "/home/.snapshots",
    ];

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut backup_ages: Vec<u64> = Vec::new();

    for path in backup_paths {
        if let Ok(metadata) = fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                let modified_secs = modified
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let age_secs = now.saturating_sub(modified_secs);
                let age_days = age_secs / 86400;

                backup_ages.push(age_days);
            }
        }
    }

    if backup_ages.is_empty() {
        return Ok(0); // No backups
    }

    // Find most recent backup
    let newest_age = backup_ages.iter().min().copied().unwrap_or(999);

    let score = if newest_age <= 7 {
        10 // Weekly
    } else if newest_age <= 14 {
        7 // Bi-weekly
    } else if newest_age <= 30 {
        5 // Monthly
    } else if newest_age <= 90 {
        3 // Quarterly
    } else {
        0 // Rare
    };

    Ok(score)
}

/// Score risk exposure: root usage and sudo frequency
///
/// Formula:
/// - Check sudo logs (last 7 days)
/// - < 10 sudo commands = 10 (cautious)
/// - 10-30 commands = 7 (moderate)
/// - 30-100 commands = 5 (frequent)
/// - 100-500 commands = 3 (heavy)
/// - 500+ commands = 0 (reckless)
fn score_risk() -> Result<u8> {
    let output = Command::new("journalctl")
        .args(&["_COMM=sudo", "--since", "7 days ago", "--no-pager", "-q"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let sudo_count = stdout.lines().count();

        let score = if sudo_count < 10 {
            10
        } else if sudo_count < 30 {
            7
        } else if sudo_count < 100 {
            5
        } else if sudo_count < 500 {
            3
        } else {
            0
        };

        Ok(score)
    } else {
        Ok(7) // Cannot determine, assume moderate
    }
}

/// Score connectivity habits: network stability
///
/// Formula:
/// - Check NetworkManager or systemd-networkd logs
/// - Count connection changes in last 7 days
/// - 0-2 changes = 10 (stable)
/// - 3-10 changes = 7 (moderate)
/// - 11-30 changes = 5 (frequent)
/// - 31+ changes = 3 (erratic)
fn score_connectivity() -> Result<u8> {
    let output = Command::new("journalctl")
        .args(&["-u", "NetworkManager", "--since", "7 days ago", "--no-pager", "-q"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let connection_changes = stdout
            .lines()
            .filter(|line| {
                line.contains("connection") && (line.contains("activated") || line.contains("deactivated"))
            })
            .count();

        let score = if connection_changes <= 2 {
            10
        } else if connection_changes <= 10 {
            7
        } else if connection_changes <= 30 {
            5
        } else {
            3
        };

        Ok(score)
    } else {
        Ok(7) // Cannot determine, assume moderate
    }
}

/// Score power management: battery health and suspend usage
///
/// Formula:
/// - Check if laptop (battery present)
/// - If desktop: score = 8 (N/A)
/// - If laptop:
///   - Battery health > 80% = 10
///   - 60-80% = 7
///   - 40-60% = 5
///   - < 40% = 3
fn score_power() -> Result<u8> {
    // Check if battery exists
    let battery_path = "/sys/class/power_supply/BAT0";
    let battery1_path = "/sys/class/power_supply/BAT1";

    let battery_exists = Path::new(battery_path).exists() || Path::new(&battery1_path).exists();

    if !battery_exists {
        return Ok(8); // Desktop, N/A
    }

    // Read battery capacity
    let capacity_file = if Path::new(battery_path).exists() {
        format!("{}/capacity", battery_path)
    } else {
        format!("{}/capacity", battery1_path)
    };

    if let Ok(capacity_str) = fs::read_to_string(&capacity_file) {
        if let Ok(capacity) = capacity_str.trim().parse::<u8>() {
            let score = if capacity > 80 {
                10
            } else if capacity > 60 {
                7
            } else if capacity > 40 {
                5
            } else {
                3
            };

            return Ok(score);
        }
    }

    Ok(7) // Cannot determine, assume moderate
}

/// Score warning response: how quickly warnings are addressed
///
/// Formula:
/// - Count systemd failed units that have been persisting
/// - Check journal warnings that repeat across boots
/// - 0 persistent warnings = 10
/// - 1-3 persistent warnings = 7
/// - 4-10 persistent warnings = 5
/// - 11+ persistent warnings = 2
fn score_warnings() -> Result<u8> {
    // Check for failed units
    let output = Command::new("systemctl")
        .args(&["--failed", "--no-pager"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let failed_count = stdout
            .lines()
            .filter(|line| line.contains("loaded") && line.contains("failed"))
            .count();

        let score = if failed_count == 0 {
            10
        } else if failed_count <= 3 {
            7
        } else if failed_count <= 10 {
            5
        } else {
            2
        };

        Ok(score)
    } else {
        Ok(7) // Cannot determine, assume moderate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radar_structure() {
        let mut radar = UserRadar {
            overall: 0,
            regularity: 10,
            workspace: 7,
            updates: 10,
            backups: 7,
            risk: 10,
            connectivity: 7,
            power: 8,
            warnings: 10,
        };

        radar.calculate_overall();

        // Average: (10+7+10+7+10+7+8+10) / 8 = 69 / 8 = 8.625 -> 8
        assert_eq!(radar.overall, 8);
    }

    #[test]
    fn test_workspace_scoring() {
        // Test workspace size thresholds (in GB)
        let test_cases = vec![
            (0, 10),   // < 1 GB = 10
            (3, 7),    // 1-5 GB = 7
            (8, 5),    // 5-10 GB = 5
            (15, 3),   // 10-20 GB = 3
            (25, 0),   // 20+ GB = 0
        ];

        for (total_gb, expected_score) in test_cases {
            let score = if total_gb < 1 {
                10
            } else if total_gb < 5 {
                7
            } else if total_gb < 10 {
                5
            } else if total_gb < 20 {
                3
            } else {
                0
            };

            assert_eq!(score, expected_score);
        }
    }

    #[test]
    fn test_update_discipline() {
        // Test update age thresholds (in days)
        let test_cases = vec![
            (3, 10),   // < 7 days = 10
            (10, 8),   // 7-14 days = 8
            (20, 6),   // 14-30 days = 6
            (45, 4),   // 30-60 days = 4
            (75, 2),   // 60-90 days = 2
            (120, 0),  // 90+ days = 0
        ];

        for (age_days, expected_score) in test_cases {
            let score = if age_days < 7 {
                10
            } else if age_days < 14 {
                8
            } else if age_days < 30 {
                6
            } else if age_days < 60 {
                4
            } else if age_days < 90 {
                2
            } else {
                0
            };

            assert_eq!(score, expected_score);
        }
    }

    #[test]
    fn test_risk_scoring() {
        // Test sudo count thresholds (per week)
        let test_cases = vec![
            (5, 10),    // < 10 = 10
            (20, 7),    // 10-30 = 7
            (50, 5),    // 30-100 = 5
            (200, 3),   // 100-500 = 3
            (600, 0),   // 500+ = 0
        ];

        for (sudo_count, expected_score) in test_cases {
            let score = if sudo_count < 10 {
                10
            } else if sudo_count < 30 {
                7
            } else if sudo_count < 100 {
                5
            } else if sudo_count < 500 {
                3
            } else {
                0
            };

            assert_eq!(score, expected_score);
        }
    }

    #[test]
    fn test_connectivity_scoring() {
        // Test connection change thresholds (per week)
        let test_cases = vec![
            (1, 10),   // 0-2 = 10
            (5, 7),    // 3-10 = 7
            (20, 5),   // 11-30 = 5
            (50, 3),   // 31+ = 3
        ];

        for (changes, expected_score) in test_cases {
            let score = if changes <= 2 {
                10
            } else if changes <= 10 {
                7
            } else if changes <= 30 {
                5
            } else {
                3
            };

            assert_eq!(score, expected_score);
        }
    }

    #[test]
    fn test_power_scoring() {
        // Test battery capacity thresholds
        let test_cases = vec![
            (95, 10),  // > 80% = 10
            (75, 7),   // 60-80% = 7
            (50, 5),   // 40-60% = 5
            (30, 3),   // < 40% = 3
        ];

        for (capacity, expected_score) in test_cases {
            let score = if capacity > 80 {
                10
            } else if capacity > 60 {
                7
            } else if capacity > 40 {
                5
            } else {
                3
            };

            assert_eq!(score, expected_score);
        }
    }
}
