//! Cache warmup for fallback commands.
//! v0.0.940: Expanded list of commands to pre-cache at startup

use std::process::Command;
use tracing::{debug, info};

use crate::core_loop::cache::{cache_command, get_cached_command};
use crate::core_loop::profile::get_system_profile;

/// v0.0.940: Expanded list of commands to pre-cache at startup
const WARMUP_COMMANDS: &[&str] = &[
    // System info
    "uname -r",
    "uname -a",
    "hostname",
    "hostnamectl",
    "cat /etc/os-release",
    "uptime -p",
    // CPU
    "lscpu | head -20",
    "nproc",
    "cat /proc/loadavg",
    // Memory
    "free -h",
    // Disk
    "df -h",
    "lsblk",
    "df -Th",
    // Network
    "ip addr",
    "ip -4 addr show | grep inet | grep -v 127.0.0.1",
    "ip route | grep default",
    // Hardware
    "lspci | grep -i vga",
    "lspci | grep -i 3d",
    // Services
    "systemctl --failed",
    // Packages
    "pacman -Q | wc -l",
    // Boot
    "systemd-analyze",
    // User
    "whoami",
    "id",
];

/// Warm up the command cache with static system info (called at daemon startup)
/// v0.0.940: Expanded from 8 to 25+ commands for comprehensive pre-caching
pub fn warm_up_cache() {
    info!("Warming up command cache with static system info...");

    let mut cached_count = 0;
    for cmd in WARMUP_COMMANDS {
        if get_cached_command(cmd).is_some() {
            continue;
        }
        // Use timeout to prevent hanging on slow commands
        match Command::new("timeout")
            .arg("2s")
            .arg("sh")
            .arg("-c")
            .arg(cmd)
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    let result = String::from_utf8_lossy(&output.stdout).to_string();
                    if !result.trim().is_empty() {
                        cache_command(cmd, &result);
                        cached_count += 1;
                    }
                }
            }
            Err(e) => debug!("Cache warm-up failed for '{}': {}", cmd, e),
        }
    }

    // v0.0.940: Also cache profile-specific commands if profile is available
    let profile = get_system_profile();
    let mut profile_cached = 0;

    // GPU-specific warmup - check PCI devices for NVIDIA
    let has_nvidia = profile.hardware.pci_devices.iter().any(|d| {
        d.vendor.to_lowercase().contains("nvidia")
    });
    if has_nvidia {
        if let Ok(output) = Command::new("timeout").arg("2s").arg("nvidia-smi").output() {
            if output.status.success() {
                let result = String::from_utf8_lossy(&output.stdout).to_string();
                if !result.trim().is_empty() {
                    cache_command("nvidia-smi", &result);
                    profile_cached += 1;
                }
            }
        }
    }

    // Audio-specific warmup
    let audio = profile.system.audio_system.as_deref().unwrap_or("");
    if audio.to_lowercase().contains("pipewire") {
        if let Ok(output) = Command::new("timeout").arg("2s").arg("sh").arg("-c").arg("wpctl status | head -30").output() {
            if output.status.success() {
                let result = String::from_utf8_lossy(&output.stdout).to_string();
                if !result.trim().is_empty() {
                    cache_command("wpctl status | head -30", &result);
                    profile_cached += 1;
                }
            }
        }
    } else {
        if let Ok(output) = Command::new("timeout").arg("2s").arg("sh").arg("-c").arg("pactl info | head -15").output() {
            if output.status.success() {
                let result = String::from_utf8_lossy(&output.stdout).to_string();
                if !result.trim().is_empty() {
                    cache_command("pactl info | head -15", &result);
                    profile_cached += 1;
                }
            }
        }
    }

    // Sensors warmup (if available)
    if let Ok(output) = Command::new("timeout").arg("2s").arg("sensors").output() {
        if output.status.success() {
            let result = String::from_utf8_lossy(&output.stdout).to_string();
            if !result.trim().is_empty() {
                cache_command("sensors", &result);
                profile_cached += 1;
            }
        }
    }

    info!("Cache warm-up complete: {} static + {} profile-specific commands pre-cached", cached_count, profile_cached);
}
