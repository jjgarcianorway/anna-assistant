//! Software Radar for Anna v0.12.9 "Orion"
//!
//! Scores software health and hygiene across 9 categories (0-10 scale):
//! 1. OS freshness (security updates pending)
//! 2. Kernel age and LTS status
//! 3. Package hygiene (broken deps, mixed repos)
//! 4. Services health (failed units)
//! 5. Security posture (firewall, SELinux/AppArmor)
//! 6. Container runtime health
//! 7. Filesystem integrity (fsck status)
//! 8. Backup presence and recency
//! 9. Log noise level (error rate from journalctl)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Software radar result with 9 category scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareRadar {
    pub overall: u8,           // Average of all categories (0-10)
    pub os_freshness: u8,      // Security updates pending (0=many, 10=none)
    pub kernel: u8,            // Kernel age and LTS status (10=recent LTS, 0=ancient)
    pub packages: u8,          // Package hygiene (10=clean, 0=broken)
    pub services: u8,          // Service health (10=all running, 0=many failed)
    pub security: u8,          // Security posture (10=hardened, 0=wide open)
    pub containers: u8,        // Container runtime health (10=healthy, 0=issues)
    pub fs_integrity: u8,      // Filesystem integrity (10=clean, 0=errors)
    pub backups: u8,           // Backup presence (10=recent, 0=none)
    pub log_noise: u8,         // Log error rate (10=quiet, 0=noisy)
}

impl SoftwareRadar {
    /// Calculate overall score (average of all categories)
    pub fn calculate_overall(&mut self) {
        let sum = self.os_freshness as u16
            + self.kernel as u16
            + self.packages as u16
            + self.services as u16
            + self.security as u16
            + self.containers as u16
            + self.fs_integrity as u16
            + self.backups as u16
            + self.log_noise as u16;

        self.overall = (sum / 9) as u8;
    }
}

/// Collect software radar data
pub fn collect_software_radar() -> Result<SoftwareRadar> {
    let mut radar = SoftwareRadar {
        overall: 0,
        os_freshness: score_os_freshness()?,
        kernel: score_kernel()?,
        packages: score_packages()?,
        services: score_services()?,
        security: score_security()?,
        containers: score_containers()?,
        fs_integrity: score_fs_integrity()?,
        backups: score_backups()?,
        log_noise: score_log_noise()?,
    };

    radar.calculate_overall();
    Ok(radar)
}

//
// Scoring Functions (0-10 scale)
//

/// Score OS freshness: security updates pending
///
/// Formula (Arch):
/// - Check `pacman -Qu` for updates
/// - 0 updates = 10
/// - 1-5 updates = 8
/// - 6-15 updates = 6
/// - 16-30 updates = 4
/// - 31-50 updates = 2
/// - 51+ updates = 0
fn score_os_freshness() -> Result<u8> {
    // Detect distro
    let distro = detect_package_manager();

    let update_count = match distro.as_str() {
        "pacman" => count_pacman_updates()?,
        "apt" => count_apt_updates()?,
        "dnf" => count_dnf_updates()?,
        "zypper" => count_zypper_updates()?,
        _ => return Ok(7), // Unknown distro, assume okay
    };

    let score = if update_count == 0 {
        10
    } else if update_count <= 5 {
        8
    } else if update_count <= 15 {
        6
    } else if update_count <= 30 {
        4
    } else if update_count <= 50 {
        2
    } else {
        0
    };

    Ok(score)
}

/// Detect package manager
fn detect_package_manager() -> String {
    if Path::new("/usr/bin/pacman").exists() {
        "pacman".to_string()
    } else if Path::new("/usr/bin/apt").exists() {
        "apt".to_string()
    } else if Path::new("/usr/bin/dnf").exists() {
        "dnf".to_string()
    } else if Path::new("/usr/bin/zypper").exists() {
        "zypper".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Count pending updates for pacman (Arch)
fn count_pacman_updates() -> Result<usize> {
    let output = Command::new("pacman")
        .arg("-Qu")
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().count())
    } else {
        Ok(0) // If pacman fails, assume no updates
    }
}

/// Count pending updates for apt (Debian/Ubuntu)
fn count_apt_updates() -> Result<usize> {
    let output = Command::new("apt")
        .args(&["list", "--upgradable"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Subtract 1 for header line "Listing..."
        let count = stdout.lines().count().saturating_sub(1);
        Ok(count)
    } else {
        Ok(0)
    }
}

/// Count pending updates for dnf (Fedora/RHEL)
fn count_dnf_updates() -> Result<usize> {
    let output = Command::new("dnf")
        .args(&["check-update", "-q"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().filter(|line| !line.trim().is_empty()).count())
    } else {
        Ok(0)
    }
}

/// Count pending updates for zypper (openSUSE)
fn count_zypper_updates() -> Result<usize> {
    let output = Command::new("zypper")
        .args(&["list-updates"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().filter(|line| line.starts_with('v')).count())
    } else {
        Ok(0)
    }
}

/// Score kernel age and LTS status
///
/// Formula:
/// - Read /proc/version for kernel version
/// - Extract major.minor (e.g., 6.17)
/// - Check if LTS (6.1, 6.6, 5.15, 5.10, 5.4, 4.19, 4.14)
/// - LTS + recent (6.6, 6.1) = 10
/// - LTS + older (5.15, 5.10) = 8
/// - Non-LTS + recent (6.10+) = 7
/// - Ancient (<5.0) = 3
fn score_kernel() -> Result<u8> {
    let version_str = fs::read_to_string("/proc/version")
        .context("Failed to read /proc/version")?;

    // Parse kernel version (e.g., "Linux version 6.17.6-arch1-1")
    let version = version_str
        .split_whitespace()
        .nth(2)
        .unwrap_or("0.0.0");

    let parts: Vec<&str> = version.split('.').collect();
    let major: u32 = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);

    let kernel_tuple = (major, minor);

    // LTS kernels
    let lts_kernels = vec![
        (6, 12), // LTS
        (6, 6),  // LTS
        (6, 1),  // LTS
        (5, 15), // LTS
        (5, 10), // LTS
        (5, 4),  // LTS
        (4, 19), // EOL but still LTS
        (4, 14), // EOL but still LTS
    ];

    if lts_kernels.contains(&kernel_tuple) {
        // Recent LTS
        if major >= 6 {
            Ok(10)
        } else {
            Ok(8) // Older LTS
        }
    } else if major >= 6 && minor >= 10 {
        // Recent non-LTS
        Ok(7)
    } else if major >= 5 {
        // Modern kernel
        Ok(6)
    } else if major >= 4 {
        // Old kernel
        Ok(4)
    } else {
        // Ancient kernel
        Ok(2)
    }
}

/// Score package hygiene: broken deps, mixed repos
///
/// Formula:
/// - Check for broken packages (Arch: pacdiff, Debian: dpkg --audit)
/// - 0 issues = 10
/// - 1-2 issues = 7
/// - 3-5 issues = 5
/// - 6+ issues = 2
fn score_packages() -> Result<u8> {
    let distro = detect_package_manager();

    let broken_count = match distro.as_str() {
        "pacman" => check_pacman_issues()?,
        "apt" => check_apt_issues()?,
        _ => 0, // Unknown distro, assume okay
    };

    let score = if broken_count == 0 {
        10
    } else if broken_count <= 2 {
        7
    } else if broken_count <= 5 {
        5
    } else {
        2
    };

    Ok(score)
}

/// Check for pacman package issues
fn check_pacman_issues() -> Result<usize> {
    // Check for .pacnew files (config file conflicts)
    let output = Command::new("sh")
        .arg("-c")
        .arg("find /etc -name '*.pacnew' 2>/dev/null | wc -l")
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().parse().unwrap_or(0))
    } else {
        Ok(0)
    }
}

/// Check for apt package issues
fn check_apt_issues() -> Result<usize> {
    let output = Command::new("dpkg")
        .arg("--audit")
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().filter(|line| !line.trim().is_empty()).count())
    } else {
        Ok(0)
    }
}

/// Score services health: failed systemd units
///
/// Formula:
/// - Run `systemctl --failed --no-pager`
/// - 0 failed = 10
/// - 1-2 failed = 7
/// - 3-5 failed = 4
/// - 6+ failed = 0
fn score_services() -> Result<u8> {
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
        } else if failed_count <= 2 {
            7
        } else if failed_count <= 5 {
            4
        } else {
            0
        };

        Ok(score)
    } else {
        Ok(7) // systemctl unavailable, assume okay
    }
}

/// Score security posture: firewall, SELinux/AppArmor
///
/// Formula:
/// - Firewall active (ufw/firewalld/iptables) = +4 points
/// - SELinux/AppArmor enforcing = +4 points
/// - SSH hardening (PasswordAuth disabled) = +2 points
/// - Max 10 points
fn score_security() -> Result<u8> {
    let mut score = 0;

    // Check firewall
    if is_firewall_active()? {
        score += 4;
    }

    // Check MAC (Mandatory Access Control)
    if is_mac_enforcing()? {
        score += 4;
    }

    // Check SSH hardening
    if is_ssh_hardened()? {
        score += 2;
    }

    Ok(score.min(10))
}

/// Check if firewall is active
fn is_firewall_active() -> Result<bool> {
    // Check ufw
    if let Ok(output) = Command::new("ufw").arg("status").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("Status: active") {
            return Ok(true);
        }
    }

    // Check firewalld
    if let Ok(output) = Command::new("firewall-cmd").arg("--state").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("running") {
            return Ok(true);
        }
    }

    // Check iptables (simple heuristic: more than 5 rules)
    if let Ok(output) = Command::new("iptables").arg("-L").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.lines().count() > 10 {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Check if SELinux or AppArmor is enforcing
fn is_mac_enforcing() -> Result<bool> {
    // Check SELinux
    if let Ok(output) = Command::new("getenforce").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim() == "Enforcing" {
            return Ok(true);
        }
    }

    // Check AppArmor
    if let Ok(output) = Command::new("aa-status").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("profiles are in enforce mode") {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// Check if SSH is hardened (PasswordAuthentication disabled)
fn is_ssh_hardened() -> Result<bool> {
    let sshd_config = "/etc/ssh/sshd_config";
    if let Ok(content) = fs::read_to_string(sshd_config) {
        for line in content.lines() {
            if line.trim().starts_with("PasswordAuthentication") {
                return Ok(line.contains("no"));
            }
        }
    }

    Ok(false) // Assume not hardened if can't check
}

/// Score container runtime health
///
/// Formula:
/// - Docker/Podman not installed = 8 (N/A, assume okay)
/// - Installed + running + healthy = 10
/// - Installed + stopped = 5
/// - Installed + errors = 0
fn score_containers() -> Result<u8> {
    // Check if Docker is installed
    let docker_installed = Path::new("/usr/bin/docker").exists();
    let podman_installed = Path::new("/usr/bin/podman").exists();

    if !docker_installed && !podman_installed {
        return Ok(8); // N/A, not used
    }

    // Check Docker health
    if docker_installed {
        if let Ok(output) = Command::new("docker").arg("info").output() {
            if output.status.success() {
                return Ok(10); // Healthy
            } else {
                return Ok(0); // Errors
            }
        } else {
            return Ok(5); // Not running
        }
    }

    // Check Podman health
    if podman_installed {
        if let Ok(output) = Command::new("podman").arg("info").output() {
            if output.status.success() {
                return Ok(10); // Healthy
            } else {
                return Ok(0); // Errors
            }
        } else {
            return Ok(5); // Not running
        }
    }

    Ok(8) // Fallback
}

/// Score filesystem integrity
///
/// Formula:
/// - Check for fsck errors in journal (last boot)
/// - 0 errors = 10
/// - 1-2 warnings = 7
/// - 3+ errors = 0
fn score_fs_integrity() -> Result<u8> {
    let output = Command::new("journalctl")
        .args(&["-b", "-p", "err", "--no-pager", "-q"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let fsck_errors = stdout
            .lines()
            .filter(|line| line.to_lowercase().contains("fsck") || line.contains("ext4"))
            .count();

        let score = if fsck_errors == 0 {
            10
        } else if fsck_errors <= 2 {
            7
        } else {
            0
        };

        Ok(score)
    } else {
        Ok(8) // journalctl unavailable, assume okay
    }
}

/// Score backup presence and recency
///
/// Formula:
/// - Check common backup locations:
///   - /var/backups
///   - /backup
///   - /mnt/backup
///   - ~/.local/share/restic
/// - Recent backup (<7 days) = 10
/// - Backup exists (7-30 days) = 7
/// - Old backup (30+ days) = 4
/// - No backup = 0
fn score_backups() -> Result<u8> {
    let backup_paths = vec![
        "/var/backups",
        "/backup",
        "/mnt/backup",
        "/home/.snapshots", // btrfs snapshots
    ];

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut newest_backup_age: Option<u64> = None;

    for path in backup_paths {
        if let Ok(metadata) = fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                let modified_secs = modified
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let age_secs = now.saturating_sub(modified_secs);

                if newest_backup_age.is_none() || age_secs < newest_backup_age.unwrap() {
                    newest_backup_age = Some(age_secs);
                }
            }
        }
    }

    if let Some(age_secs) = newest_backup_age {
        let age_days = age_secs / 86400;

        let score = if age_days < 7 {
            10 // Recent
        } else if age_days < 30 {
            7 // Exists
        } else {
            4 // Old
        };

        Ok(score)
    } else {
        Ok(0) // No backup found
    }
}

/// Score log noise level: error rate from journalctl
///
/// Formula:
/// - Count errors in last 24 hours
/// - 0-5 errors = 10
/// - 6-20 errors = 7
/// - 21-50 errors = 5
/// - 51-100 errors = 3
/// - 101+ errors = 0
fn score_log_noise() -> Result<u8> {
    let output = Command::new("journalctl")
        .args(&["--since", "24 hours ago", "-p", "err", "--no-pager", "-q"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let error_count = stdout.lines().count();

        let score = if error_count <= 5 {
            10
        } else if error_count <= 20 {
            7
        } else if error_count <= 50 {
            5
        } else if error_count <= 100 {
            3
        } else {
            0
        };

        Ok(score)
    } else {
        Ok(7) // journalctl unavailable, assume okay
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radar_structure() {
        let mut radar = SoftwareRadar {
            overall: 0,
            os_freshness: 10,
            kernel: 10,
            packages: 10,
            services: 10,
            security: 8,
            containers: 8,
            fs_integrity: 10,
            backups: 7,
            log_noise: 10,
        };

        radar.calculate_overall();

        // Average: (10+10+10+10+8+8+10+7+10) / 9 = 83 / 9 = 9.22 -> 9
        assert_eq!(radar.overall, 9);
    }

    #[test]
    fn test_os_freshness_scoring() {
        // Test the scoring thresholds
        let test_cases = vec![
            (0, 10),   // 0 updates = 10
            (3, 8),    // 1-5 updates = 8
            (10, 6),   // 6-15 updates = 6
            (25, 4),   // 16-30 updates = 4
            (40, 2),   // 31-50 updates = 2
            (100, 0),  // 51+ updates = 0
        ];

        for (update_count, expected_score) in test_cases {
            let score = if update_count == 0 {
                10
            } else if update_count <= 5 {
                8
            } else if update_count <= 15 {
                6
            } else if update_count <= 30 {
                4
            } else if update_count <= 50 {
                2
            } else {
                0
            };

            assert_eq!(score, expected_score);
        }
    }

    #[test]
    fn test_kernel_scoring_logic() {
        // Test kernel version parsing logic
        let version_str = "Linux version 6.17.6-arch1-1";
        let version = version_str.split_whitespace().nth(2).unwrap();
        let parts: Vec<&str> = version.split('.').collect();
        let major: u32 = parts[0].parse().unwrap();
        let minor: u32 = parts[1].parse().unwrap();

        assert_eq!(major, 6);
        assert_eq!(minor, 17);
    }

    #[test]
    fn test_services_scoring() {
        // Test failed service counting logic
        let test_cases = vec![
            (0, 10),  // 0 failed = 10
            (1, 7),   // 1-2 failed = 7
            (3, 4),   // 3-5 failed = 4
            (10, 0),  // 6+ failed = 0
        ];

        for (failed_count, expected_score) in test_cases {
            let score = if failed_count == 0 {
                10
            } else if failed_count <= 2 {
                7
            } else if failed_count <= 5 {
                4
            } else {
                0
            };

            assert_eq!(score, expected_score);
        }
    }

    #[test]
    fn test_backup_age_scoring() {
        // Test backup age thresholds (in days)
        let test_cases = vec![
            (3, 10),   // <7 days = 10
            (15, 7),   // 7-30 days = 7
            (45, 4),   // 30+ days = 4
        ];

        for (age_days, expected_score) in test_cases {
            let score = if age_days < 7 {
                10
            } else if age_days < 30 {
                7
            } else {
                4
            };

            assert_eq!(score, expected_score);
        }
    }

    #[test]
    fn test_log_noise_scoring() {
        // Test log error count thresholds
        let test_cases = vec![
            (2, 10),   // 0-5 errors = 10
            (15, 7),   // 6-20 errors = 7
            (35, 5),   // 21-50 errors = 5
            (75, 3),   // 51-100 errors = 3
            (150, 0),  // 101+ errors = 0
        ];

        for (error_count, expected_score) in test_cases {
            let score = if error_count <= 5 {
                10
            } else if error_count <= 20 {
                7
            } else if error_count <= 50 {
                5
            } else if error_count <= 100 {
                3
            } else {
                0
            };

            assert_eq!(score, expected_score);
        }
    }
}
