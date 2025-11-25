//! System Query - Direct system state queries
//!
//! Provides real system data by querying the system directly
//! This serves as both a fallback when annad isn't available
//! and a template for what annad should provide via RPC

use anna_common::telemetry::*;
use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// Query current system telemetry
pub fn query_system_telemetry() -> Result<SystemTelemetry> {
    Ok(SystemTelemetry {
        timestamp: chrono::Utc::now(),
        hardware: query_hardware()?,
        disks: query_disks()?,
        memory: query_memory()?,
        cpu: query_cpu()?,
        packages: query_packages()?,
        services: query_services()?,
        network: query_network()?,
        security: query_security()?,
        desktop: query_desktop().ok(),
        boot: query_boot().ok(),
        audio: query_audio()?,
    })
}

fn query_hardware() -> Result<HardwareInfo> {
    // Check if battery exists
    let has_battery = Path::new("/sys/class/power_supply/BAT0").exists()
        || Path::new("/sys/class/power_supply/BAT1").exists();

    // Determine machine type
    let machine_type = if has_battery {
        MachineType::Laptop
    } else {
        // Check for server indicators
        let hostname_output = Command::new("hostnamectl")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        if hostname_output.contains("server") || hostname_output.contains("headless") {
            MachineType::Server
        } else {
            MachineType::Desktop
        }
    };

    // Get CPU model
    let cpu_model = std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|content| {
            content
                .lines()
                .find(|line| line.starts_with("model name"))
                .and_then(|line| line.split(':').nth(1))
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| "Unknown CPU".to_string());

    // Get total RAM from /proc/meminfo
    let total_ram_mb = std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|content| {
            content
                .lines()
                .find(|line| line.starts_with("MemTotal:"))
                .and_then(|line| {
                    line.split_whitespace()
                        .nth(1)
                        .and_then(|s| s.parse::<u64>().ok())
                        .map(|kb| kb / 1024)
                })
        })
        .unwrap_or(0);

    // Check for GPU
    let has_gpu = Command::new("lspci")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|output| output.contains("VGA") || output.contains("3D"))
        .unwrap_or(false);

    let gpu_info = if has_gpu {
        Command::new("lspci")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|output| {
                output
                    .lines()
                    .find(|line| line.contains("VGA") || line.contains("3D"))
                    .map(|line| line.to_string())
            })
    } else {
        None
    };

    Ok(HardwareInfo {
        cpu_model,
        total_ram_mb,
        machine_type,
        has_battery,
        has_gpu,
        gpu_info,
    })
}

fn query_disks() -> Result<Vec<DiskInfo>> {
    let output = Command::new("df")
        .arg("-BM")
        .arg("--output=target,size,used,pcent,fstype")
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut disks = Vec::new();

    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            let mount_point = parts[0].to_string();
            let total_mb = parts[1].trim_end_matches('M').parse::<u64>().unwrap_or(0);
            let used_mb = parts[2].trim_end_matches('M').parse::<u64>().unwrap_or(0);
            let usage_percent = parts[3].trim_end_matches('%').parse::<f64>().unwrap_or(0.0);
            let fs_type = parts[4].to_string();

            // Only include real filesystems
            if mount_point.starts_with('/')
                && !mount_point.starts_with("/sys")
                && !mount_point.starts_with("/proc")
                && !mount_point.starts_with("/dev")
                && !mount_point.starts_with("/run")
            {
                disks.push(DiskInfo {
                    mount_point,
                    total_mb,
                    used_mb,
                    usage_percent,
                    fs_type,
                    smart_status: None, // TODO: Query SMART data
                });
            }
        }
    }

    Ok(disks)
}

fn query_memory() -> Result<MemoryInfo> {
    let content = std::fs::read_to_string("/proc/meminfo")?;

    let mut total_mb = 0;
    let mut available_mb = 0;
    let mut swap_total_mb = 0;
    let mut swap_used_mb = 0;

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let value_kb = parts[1].parse::<u64>().unwrap_or(0);
            let value_mb = value_kb / 1024;

            match parts[0] {
                "MemTotal:" => total_mb = value_mb,
                "MemAvailable:" => available_mb = value_mb,
                "SwapTotal:" => swap_total_mb = value_mb,
                "SwapFree:" => {
                    let swap_free_mb = value_mb;
                    swap_used_mb = swap_total_mb.saturating_sub(swap_free_mb);
                }
                _ => {}
            }
        }
    }

    let used_mb = total_mb.saturating_sub(available_mb);
    let usage_percent = if total_mb > 0 {
        (used_mb as f64 / total_mb as f64) * 100.0
    } else {
        0.0
    };

    Ok(MemoryInfo {
        total_mb,
        available_mb,
        used_mb,
        swap_total_mb,
        swap_used_mb,
        usage_percent,
    })
}

fn query_cpu() -> Result<CpuInfo> {
    // Get number of cores
    let cores = std::fs::read_to_string("/proc/cpuinfo")?
        .lines()
        .filter(|line| line.starts_with("processor"))
        .count() as u32;

    // Get load averages
    let loadavg = std::fs::read_to_string("/proc/loadavg")?;
    let parts: Vec<&str> = loadavg.split_whitespace().collect();

    let load_avg_1min = parts
        .first()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    let load_avg_5min = parts
        .get(1)
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    Ok(CpuInfo {
        cores,
        load_avg_1min,
        load_avg_5min,
        usage_percent: None, // Would need sampling over time
    })
}

fn query_packages() -> Result<PackageInfo> {
    // Get total installed packages
    let total_installed = Command::new("pacman")
        .args(["-Q"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.lines().count() as u64)
        .unwrap_or(0);

    // Get updates available (v6.37.0: try yay first for both repos + AUR)
    let updates_available = Command::new("yay")
        .arg("-Qu")
        .output()
        .or_else(|_| Command::new("pacman").arg("-Qu").output())
        .or_else(|_| Command::new("checkupdates").output())
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.lines().filter(|l| !l.is_empty()).count() as u64)
        .unwrap_or(0);

    // Get orphaned packages
    let orphaned = Command::new("pacman")
        .args(["-Qtdq"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.lines().filter(|line| !line.is_empty()).count() as u64)
        .unwrap_or(0);

    // Get pacman cache size
    let cache_size_mb = Command::new("du")
        .args(["-sm", "/var/cache/pacman/pkg"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| {
            s.split_whitespace()
                .next()
                .and_then(|size| size.parse::<f64>().ok())
        })
        .unwrap_or(0.0);

    // Get last update time from pacman log
    let last_update = std::fs::read_to_string("/var/log/pacman.log")
        .ok()
        .and_then(|content| {
            content
                .lines()
                .rev()
                .find(|line| line.contains("starting full system upgrade"))
                .and_then(|line| {
                    line.split('[')
                        .nth(1)
                        .and_then(|s| s.split(']').next())
                        .and_then(|timestamp| {
                            chrono::DateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S%z")
                                .ok()
                                .map(|dt| dt.with_timezone(&chrono::Utc))
                        })
                })
        });

    Ok(PackageInfo {
        total_installed,
        updates_available,
        orphaned,
        cache_size_mb,
        last_update,
    })
}

fn query_services() -> Result<ServiceInfo> {
    // Get failed systemd units
    let output = Command::new("systemctl")
        .args(["list-units", "--failed", "--no-pager", "--no-legend"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut failed_units = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if !parts.is_empty() {
            let name = parts[0].to_string();
            let unit_type = name.split('.').next_back().unwrap_or("service").to_string();

            failed_units.push(FailedUnit {
                name,
                unit_type,
                failed_since: None, // TODO: Parse from systemctl status
                message: None,
            });
        }
    }

    // Count total units
    let total_output = Command::new("systemctl")
        .args(["list-units", "--no-pager", "--no-legend"])
        .output()?;

    let total_units = String::from_utf8_lossy(&total_output.stdout)
        .lines()
        .count() as u64;

    Ok(ServiceInfo {
        total_units,
        failed_units,
        recently_restarted: Vec::new(), // TODO: Parse from journal
    })
}

fn query_network() -> Result<NetworkInfo> {
    // Check if connected
    let is_connected = Command::new("ping")
        .args(["-c", "1", "-W", "2", "8.8.8.8"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Get primary interface
    let primary_interface = Command::new("ip")
        .args(["route", "show", "default"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| {
            s.split_whitespace()
                .position(|w| w == "dev")
                .and_then(|i| s.split_whitespace().nth(i + 1))
                .map(|s| s.to_string())
        });

    // Check firewall
    let firewall_active = Command::new("systemctl")
        .args(["is-active", "ufw"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
        || Command::new("systemctl")
            .args(["is-active", "iptables"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

    let firewall_type = if firewall_active {
        if Command::new("which")
            .arg("ufw")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            Some("ufw".to_string())
        } else {
            Some("iptables".to_string())
        }
    } else {
        None
    };

    Ok(NetworkInfo {
        is_connected,
        primary_interface,
        firewall_active,
        firewall_type,
    })
}

fn query_security() -> Result<SecurityInfo> {
    // Count failed SSH attempts from journal
    let failed_ssh_attempts = Command::new("journalctl")
        .args([
            "-u",
            "sshd",
            "--since",
            "1 week ago",
            "-g",
            "Failed password",
        ])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.lines().count() as u64)
        .unwrap_or(0);

    // Check for auto-updates
    let auto_updates_enabled =
        Path::new("/etc/systemd/system/timers.target.wants/pacman-auto-update.timer").exists()
            || Command::new("systemctl")
                .args(["is-enabled", "pacman-auto-update.timer"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

    Ok(SecurityInfo {
        failed_ssh_attempts,
        auto_updates_enabled,
        audit_warnings: Vec::new(), // TODO: Implement security audits
    })
}

fn query_desktop() -> Result<DesktopInfo> {
    let de_name = std::env::var("XDG_CURRENT_DESKTOP").ok();
    let session_type = std::env::var("XDG_SESSION_TYPE").ok();

    // Try to get window manager
    let wm_name = std::env::var("WINDOW_MANAGER")
        .ok()
        .or_else(|| std::env::var("DESKTOP_SESSION").ok());

    // Count monitors
    let monitor_count = Command::new("xrandr")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.lines().filter(|line| line.contains(" connected")).count() as u32)
        .unwrap_or(1);

    Ok(DesktopInfo {
        de_name,
        wm_name,
        display_server: session_type,
        monitor_count,
    })
}

fn query_boot() -> Result<BootInfo> {
    // Get last boot time using systemd-analyze
    let output = Command::new("systemd-analyze").output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    let last_boot_time_secs = stdout
        .lines()
        .find(|line| line.contains("="))
        .and_then(|line| {
            line.split('=')
                .next_back()
                .and_then(|s| s.split_whitespace().next())
                .and_then(|time_str| {
                    // Parse formats like "5.123s" or "1min 30.5s"
                    if time_str.contains("min") {
                        // TODO: Parse minutes + seconds
                        None
                    } else {
                        time_str.trim_end_matches('s').parse::<f64>().ok()
                    }
                })
        })
        .unwrap_or(0.0);

    Ok(BootInfo {
        last_boot_time_secs,
        avg_boot_time_secs: None, // Would need historical data
        trend: None,
    })
}

fn query_audio() -> Result<AudioTelemetry> {
    // Check for sound hardware (same logic as in telemetry.rs)
    let has_sound_hardware = check_sound_hardware();

    // Check PipeWire services
    let pipewire_running = Command::new("systemctl")
        .args(["--user", "is-active", "pipewire"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let wireplumber_running = Command::new("systemctl")
        .args(["--user", "is-active", "wireplumber"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let pipewire_pulse_running = Command::new("systemctl")
        .args(["--user", "is-active", "pipewire-pulse"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    Ok(AudioTelemetry {
        has_sound_hardware,
        pipewire_running,
        wireplumber_running,
        pipewire_pulse_running,
    })
}

fn check_sound_hardware() -> bool {
    // Method 1: aplay -l (ALSA)
    if let Ok(output) = Command::new("aplay").arg("-l").output() {
        if output.status.success() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                if stdout.contains("card ") {
                    return true;
                }
            }
        }
    }

    // Method 2: Check /proc/asound/cards
    if let Ok(content) = std::fs::read_to_string("/proc/asound/cards") {
        if !content.trim().is_empty() && !content.contains("no soundcards") {
            return true;
        }
    }

    // Method 3: Check for sound devices in /dev/snd
    if let Ok(entries) = std::fs::read_dir("/dev/snd") {
        if entries.count() > 0 {
            return true;
        }
    }

    false
}
