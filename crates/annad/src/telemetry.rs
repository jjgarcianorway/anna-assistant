//! System telemetry collection
//!
//! Gathers hardware, software, and system state information.

use anna_common::{
    BatteryInfo, BluetoothStatus, CommandUsage, LocaleInfo, MediaUsageProfile, MicrocodeStatus,
    SSDInfo, StorageDevice, SwapConfiguration, SystemFacts, SystemdService,
};
use anna_common::desktop::{DesktopInfo, DesktopEnvironment, SessionType};
use anna_common::config_file::DesktopConfig;
use anna_common::boot::BootInfo;
use anna_common::audio::AudioInfo;
use anna_common::filesystem::FilesystemInfo;
use anna_common::systemd_health::SystemdHealth;
use anna_common::network_config::NetworkConfig;
use anna_common::cpu_performance::CpuPerformance;
use anna_common::graphics::GraphicsInfo;
use anna_common::security::SecurityInfo;
use anna_common::virtualization::VirtualizationInfo;
use anna_common::package_mgmt::PackageManagementInfo;
use anna_common::sensors::SensorsInfo;
use anna_common::power::PowerInfo;
use anna_common::memory_usage::MemoryUsageInfo;
use anna_common::storage::StorageInfo;
use anna_common::network_monitoring::NetworkMonitoring;
use anna_common::kernel_modules::KernelModules;
use anna_common::package_health::PackageHealth;
use anna_common::initramfs::InitramfsInfo;
use anna_common::security_features::SecurityFeatures;
use anna_common::system_health::SystemHealth;
use anna_common::orphaned_packages::OrphanedPackages;
use anna_common::cpu_throttling::CpuThrottling;
use anna_common::gpu_throttling::GpuThrottling;
use anna_common::gpu_compute::GpuComputeCapabilities;
use anna_common::voltage_monitoring::VoltageMonitoring;
use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use sysinfo::System;
use tracing::info;

/// Collect current system facts
pub async fn collect_facts() -> Result<SystemFacts> {
    info!("Collecting comprehensive system facts");

    let mut sys = System::new_all();
    sys.refresh_all();

    let hostname = get_hostname()?;
    let kernel = get_kernel_version()?;
    let cpu_model = get_cpu_model(&sys);
    let cpu_cores = sys.cpus().len();
    let total_memory_gb = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let gpu_vendor = detect_gpu();
    let storage_devices = get_storage_devices()?;
    let installed_packages = count_packages()?;
    let orphan_packages = find_orphan_packages()?;
    let network_interfaces = get_network_interfaces();
    let package_groups = detect_package_groups();

    Ok(SystemFacts {
        timestamp: Utc::now(),

        // Hardware
        hostname,
        kernel,
        cpu_model,
        cpu_cores,
        total_memory_gb,
        gpu_vendor,
        storage_devices,

        // Software & Packages
        installed_packages,
        orphan_packages,
        package_groups,

        // Network
        network_interfaces,
        has_wifi: detect_wifi(),
        has_ethernet: detect_ethernet(),

        // User Environment (using unified desktop detection module)
        shell: detect_shell(),
        desktop_environment: {
            // Detect desktop environment once and reuse
            let desktop_info = DesktopInfo::detect();

            match desktop_info.environment {
                DesktopEnvironment::None => None,
                ref env => Some(env.name().to_string()),
            }
        },
        window_manager: {
            let desktop_info = DesktopInfo::detect();
            match desktop_info.environment {
                DesktopEnvironment::None => None,
                ref env => Some(env.name().to_string()),
            }
        },
        compositor: {
            let desktop_info = DesktopInfo::detect();
            match desktop_info.session_type {
                SessionType::Wayland => match desktop_info.environment {
                    DesktopEnvironment::Hyprland => Some("Hyprland".to_string()),
                    DesktopEnvironment::Sway => Some("Sway".to_string()),
                    _ => None,
                },
                _ => None,
            }
        },
        display_server: {
            let desktop_info = DesktopInfo::detect();
            match desktop_info.session_type {
                SessionType::Wayland => Some("wayland".to_string()),
                SessionType::X11 => Some("x11".to_string()),
                SessionType::Tty => Some("tty".to_string()),
                SessionType::Unknown => None,
            }
        },
        desktop_config: DesktopConfig::parse(),
        boot_info: Some(BootInfo::detect()),
        audio_info: Some(AudioInfo::detect()),
        filesystem_info: Some(FilesystemInfo::detect()),
        systemd_health: Some(SystemdHealth::detect()),
        network_config: Some(NetworkConfig::detect()),
        cpu_performance: Some(CpuPerformance::detect()),
        graphics_info: Some(GraphicsInfo::detect()),
        security_info: Some(SecurityInfo::detect()),
        virtualization_info: Some(VirtualizationInfo::detect()),
        package_mgmt_info: Some(PackageManagementInfo::detect()),
        sensors_info: Some(SensorsInfo::detect()),
        power_info: Some(PowerInfo::detect()),
        memory_usage_info: Some(MemoryUsageInfo::detect()),
        storage_info: Some(StorageInfo::detect()),
        network_monitoring: Some(NetworkMonitoring::detect()),
        kernel_modules: Some(KernelModules::detect()),
        package_health: Some(PackageHealth::detect()),
        initramfs_info: Some(InitramfsInfo::detect()),
        security_features: Some(SecurityFeatures::detect()),
        system_health: Some(SystemHealth::detect()),
        orphaned_packages: Some(OrphanedPackages::detect()),
        cpu_throttling: Some(CpuThrottling::detect()),
        gpu_throttling: Some(GpuThrottling::detect()),
        gpu_compute: Some(GpuComputeCapabilities::detect()),
        voltage_monitoring: Some(VoltageMonitoring::detect()),
        is_nvidia: detect_nvidia(),
        nvidia_driver_version: if detect_nvidia() {
            get_nvidia_driver_version()
        } else {
            None
        },
        has_wayland_nvidia_support: check_wayland_nvidia_support(),
        is_intel_gpu: detect_intel_gpu(),
        is_amd_gpu: detect_amd_gpu(),
        amd_driver_version: if detect_amd_gpu() {
            get_amd_driver_version()
        } else {
            None
        },

        // Enhanced GPU Telemetry (beta.43+)
        gpu_model: get_gpu_model(),
        gpu_vram_mb: get_gpu_vram_mb(),
        vulkan_support: check_vulkan_support(),
        nvidia_cuda_support: check_nvidia_cuda_support(),

        // Performance Score - deprecated in 1.0 (no heuristics)
        performance_score: 0,
        resource_tier: String::from("unknown"),

        // User Behavior (basic for now)
        frequently_used_commands: analyze_command_history().await,
        dev_tools_detected: detect_dev_tools(),
        media_usage: analyze_media_usage().await,
        common_file_types: detect_common_file_types().await,

        // Boot Performance
        boot_time_seconds: get_boot_time(),
        slow_services: get_slow_services(),
        failed_services: get_failed_services(),

        // Package Management
        aur_packages: count_aur_packages(),
        aur_helper: detect_aur_helper(),
        package_cache_size_gb: get_package_cache_size(),
        last_system_upgrade: get_last_upgrade_time(),

        // Kernel & Boot Parameters
        kernel_parameters: get_kernel_parameters(),

        // Advanced Telemetry
        recently_installed_packages: get_recently_installed_packages(),
        active_services: get_active_services(),
        enabled_services: get_enabled_services(),
        disk_usage_trend: analyze_disk_usage(),
        session_info: collect_session_info(),
        development_environment: {
            let dev_env = analyze_development_environment().await;
            dev_env
        },
        gaming_profile: analyze_gaming_profile(),
        network_profile: analyze_network_profile(),
        system_age_days: get_system_age_days(),
        user_preferences: {
            let dev_tools = detect_dev_tools();
            infer_user_preferences(&dev_tools, installed_packages)
        },

        // Enhanced Telemetry (beta.35+)
        hardware_monitoring: collect_hardware_monitoring(&sys),
        disk_health: collect_disk_health(),
        system_health_metrics: collect_system_health_metrics(),
        performance_metrics: collect_performance_metrics(&sys),
        predictive_insights: generate_predictive_insights(),

        // Extended Telemetry (beta.43+)
        microcode_status: detect_microcode_status(&get_cpu_model(&sys)),
        battery_info: collect_battery_info(),
        backup_systems: detect_backup_systems(),
        bluetooth_status: detect_bluetooth_status(),
        ssd_info: collect_ssd_info(),
        swap_config: analyze_swap_configuration(),
        locale_info: collect_locale_info(),
        pacman_hooks: detect_pacman_hooks(),

        // Audio System (beta.43+)
        audio_system: detect_audio_system(),
        audio_server_running: check_audio_server_running(),
        pipewire_session_manager: detect_pipewire_session_manager(),
    })
}

fn get_hostname() -> Result<String> {
    // Try hostname command first
    if let Ok(output) = Command::new("hostname").output() {
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }
    }

    // Fallback: read /etc/hostname
    if let Ok(hostname) = std::fs::read_to_string("/etc/hostname") {
        return Ok(hostname.trim().to_string());
    }

    // Last resort
    Ok("unknown".to_string())
}

fn get_kernel_version() -> Result<String> {
    let output = Command::new("uname").arg("-r").output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn get_cpu_model(sys: &System) -> String {
    sys.cpus()
        .first()
        .map(|cpu| cpu.brand().to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

fn detect_gpu() -> Option<String> {
    // Try lspci to detect GPU - MUST check line by line to avoid false positives!
    // (e.g., Intel chipset on Nvidia GPU systems would incorrectly detect as Intel GPU)
    let output = Command::new("lspci").output().ok()?;
    let lspci_output = String::from_utf8_lossy(&output.stdout);

    // Check each line for GPU-specific devices only
    for line in lspci_output.lines() {
        let lower = line.to_lowercase();

        // Only check GPU/display controller lines (not chipset/audio/etc)
        if lower.contains("vga") || lower.contains("display") || lower.contains("3d") {
            // Priority: Nvidia > AMD > Intel (dedicated GPUs take precedence)
            if lower.contains("nvidia") {
                return Some("NVIDIA".to_string());
            } else if lower.contains("amd") || lower.contains("radeon") || lower.contains("ati") {
                return Some("AMD".to_string());
            } else if lower.contains("intel") {
                return Some("Intel".to_string());
            }
        }
    }

    None
}

fn get_storage_devices() -> Result<Vec<StorageDevice>> {
    // Parse df output for mounted filesystems
    let output = Command::new("df")
        .arg("-h")
        .arg("--output=source,fstype,size,used,target")
        .output()?;

    let df_output = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    for line in df_output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            let name = parts[0].to_string();
            let filesystem = parts[1].to_string();
            let size_gb = parse_size(parts[2]);
            let used_gb = parse_size(parts[3]);
            let mount_point = parts[4].to_string();

            // Filter out tmpfs and other virtual filesystems
            if !filesystem.starts_with("tmp") && !name.starts_with("/dev/loop") {
                devices.push(StorageDevice {
                    name,
                    filesystem,
                    size_gb,
                    used_gb,
                    mount_point,
                });
            }
        }
    }

    Ok(devices)
}

fn parse_size(size_str: &str) -> f64 {
    // Parse size string like "100G" or "500M"
    let size_str = size_str.trim_end_matches(|c: char| !c.is_numeric() && c != '.');
    size_str.parse().unwrap_or(0.0)
}

fn count_packages() -> Result<usize> {
    // Count installed packages on Arch Linux
    let output = Command::new("pacman").arg("-Q").output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().count())
}

fn find_orphan_packages() -> Result<Vec<String>> {
    // Find orphaned packages (installed as dependencies but no longer needed)
    let output = Command::new("pacman").arg("-Qdtq").output()?;

    // pacman returns exit code 1 when no orphans found, which is fine
    let orphans = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    Ok(orphans)
}

fn get_network_interfaces() -> Vec<String> {
    // Get network interfaces from ip command
    let output = Command::new("ip").args(&["link", "show"]).output();

    if let Ok(output) = output {
        let ip_output = String::from_utf8_lossy(&output.stdout);
        ip_output
            .lines()
            .filter_map(|line| {
                if line.contains(": <") {
                    let parts: Vec<&str> = line.split(':').collect();
                    parts.get(1).map(|s| s.trim().to_string())
                } else {
                    None
                }
            })
            .collect()
    } else {
        vec![]
    }
}

fn detect_package_groups() -> Vec<String> {
    let mut groups = Vec::new();

    if package_installed("base-devel") {
        groups.push("base-devel".to_string());
    }
    if package_installed("gnome-shell") {
        groups.push("gnome".to_string());
    }
    if package_installed("plasma-desktop") {
        groups.push("kde-plasma".to_string());
    }
    if package_installed("xfce4-session") {
        groups.push("xfce4".to_string());
    }

    groups
}

fn package_installed(name: &str) -> bool {
    Command::new("pacman")
        .args(&["-Q", name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn detect_wifi() -> bool {
    std::fs::read_dir("/sys/class/net")
        .ok()
        .map(|entries| {
            entries.filter_map(|e| e.ok()).any(|entry| {
                let wireless_path = entry.path().join("wireless");
                wireless_path.exists()
            })
        })
        .unwrap_or(false)
}

fn detect_ethernet() -> bool {
    get_network_interfaces()
        .iter()
        .any(|iface| iface.starts_with("en") || iface.starts_with("eth"))
}

fn detect_shell() -> String {
    std::env::var("SHELL")
        .ok()
        .and_then(|s| {
            Path::new(&s)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| "bash".to_string())
}

fn detect_desktop_environment() -> Option<String> {
    if let Ok(de) = std::env::var("XDG_CURRENT_DESKTOP") {
        return Some(de);
    }

    if package_installed("gnome-shell") {
        Some("GNOME".to_string())
    } else if package_installed("plasma-desktop") {
        Some("KDE".to_string())
    } else if package_installed("xfce4-session") {
        Some("XFCE".to_string())
    } else if package_installed("i3-wm") {
        Some("i3".to_string())
    } else {
        None
    }
}

fn detect_display_server() -> Option<String> {
    if let Ok(session) = std::env::var("XDG_SESSION_TYPE") {
        return Some(session);
    }

    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        Some("wayland".to_string())
    } else if std::env::var("DISPLAY").is_ok() {
        Some("x11".to_string())
    } else {
        None
    }
}

/// Detect window manager (separate from desktop environment)
fn detect_window_manager() -> Option<String> {
    // Check environment variable first
    if let Ok(wm) = std::env::var("XDG_CURRENT_DESKTOP") {
        // Map common WM names
        match wm.to_lowercase().as_str() {
            // Wayland compositors
            "hyprland" => return Some("Hyprland".to_string()),
            "sway" => return Some("sway".to_string()),
            "wayfire" => return Some("Wayfire".to_string()),
            "river" => return Some("River".to_string()),
            // Tiling WMs
            "i3" => return Some("i3".to_string()),
            "bspwm" => return Some("bspwm".to_string()),
            "qtile" => return Some("Qtile".to_string()),
            "dwm" => return Some("dwm".to_string()),
            "xmonad" => return Some("XMonad".to_string()),
            "herbstluftwm" => return Some("Herbstluftwm".to_string()),
            "leftwm" => return Some("LeftWM".to_string()),
            "spectrwm" => return Some("Spectrwm".to_string()),
            "ratpoison" => return Some("Ratpoison".to_string()),
            "wmii" => return Some("Wmii".to_string()),
            "stumpwm" => return Some("StumpWM".to_string()),
            // Stacking WMs
            "awesome" => return Some("Awesome".to_string()),
            "openbox" => return Some("Openbox".to_string()),
            "fluxbox" => return Some("Fluxbox".to_string()),
            "blackbox" => return Some("Blackbox".to_string()),
            "icewm" => return Some("IceWM".to_string()),
            "jwm" => return Some("JWM".to_string()),
            "enlightenment" => return Some("Enlightenment".to_string()),
            "fvwm" => return Some("FVWM".to_string()),
            "windowmaker" => return Some("Window Maker".to_string()),
            "pekwm" => return Some("PekWM".to_string()),
            "evilwm" => return Some("EvilWM".to_string()),
            "cwm" => return Some("cwm".to_string()),
            "ctwm" => return Some("CTWM".to_string()),
            "afterstep" => return Some("AfterStep".to_string()),
            "sawfish" => return Some("Sawfish".to_string()),
            "twm" => return Some("twm".to_string()),
            // Desktop environment WMs
            "kwin" => return Some("KWin".to_string()),
            "mutter" => return Some("Mutter".to_string()),
            "marco" => return Some("Marco".to_string()),
            "xfwm4" => return Some("Xfwm".to_string()),
            "muffin" => return Some("Muffin".to_string()),
            "metacity" => return Some("Metacity".to_string()),
            "gala" => return Some("Gala".to_string()),
            "compiz" => return Some("Compiz".to_string()),
            _ => {}
        }
    }

    // Check if Hyprland is running
    if let Ok(output) = Command::new("pgrep").arg("-x").arg("Hyprland").output() {
        if output.status.success() {
            return Some("Hyprland".to_string());
        }
    }

    // Check for other window managers by process
    let wms = vec![
        // Wayland compositors
        ("sway", "sway"),
        ("wayfire", "Wayfire"),
        ("river", "River"),
        // Tiling WMs
        ("i3", "i3"),
        ("bspwm", "bspwm"),
        ("qtile", "qtile"),
        ("dwm", "dwm"),
        ("xmonad-x86_64-linux", "XMonad"),
        ("xmonad", "XMonad"),
        ("herbstluftwm", "Herbstluftwm"),
        ("leftwm", "LeftWM"),
        ("spectrwm", "Spectrwm"),
        ("ratpoison", "Ratpoison"),
        ("wmii", "Wmii"),
        ("stumpwm", "StumpWM"),
        ("notion", "Notion"),
        // Stacking/floating WMs
        ("awesome", "Awesome"),
        ("openbox", "Openbox"),
        ("fluxbox", "Fluxbox"),
        ("blackbox", "Blackbox"),
        ("icewm", "IceWM"),
        ("jwm", "JWM"),
        ("enlightenment", "Enlightenment"),
        ("fvwm", "FVWM"),
        ("fvwm3", "FVWM"),
        ("wmaker", "Window Maker"),
        ("WindowMaker", "Window Maker"),
        ("pekwm", "PekWM"),
        ("evilwm", "EvilWM"),
        ("cwm", "cwm"),
        ("ctwm", "CTWM"),
        ("afterstep", "AfterStep"),
        ("sawfish", "Sawfish"),
        ("twm", "twm"),
        // Desktop environment WMs
        ("kwin_x11", "KWin"),
        ("kwin_wayland", "KWin"),
        ("mutter", "Mutter"),
        ("marco", "Marco"),
        ("xfwm4", "Xfwm"),
        ("muffin", "Muffin"),
        ("metacity", "Metacity"),
        ("gala", "Gala"),
        ("compiz", "Compiz"),
    ];

    for (process, name) in wms {
        if let Ok(output) = Command::new("pgrep").arg("-x").arg(process).output() {
            if output.status.success() {
                return Some(name.to_string());
            }
        }
    }

    // Check by installed packages
    // Wayland compositors
    if package_installed("hyprland") {
        Some("Hyprland".to_string())
    } else if package_installed("sway") {
        Some("sway".to_string())
    } else if package_installed("wayfire") {
        Some("Wayfire".to_string())
    } else if package_installed("river") {
        Some("River".to_string())
    // Tiling WMs
    } else if package_installed("i3-wm") || package_installed("i3-gaps") {
        Some("i3".to_string())
    } else if package_installed("bspwm") {
        Some("bspwm".to_string())
    } else if package_installed("qtile") {
        Some("Qtile".to_string())
    } else if package_installed("dwm") {
        Some("dwm".to_string())
    } else if package_installed("xmonad") {
        Some("XMonad".to_string())
    } else if package_installed("herbstluftwm") {
        Some("Herbstluftwm".to_string())
    } else if package_installed("leftwm") {
        Some("LeftWM".to_string())
    } else if package_installed("spectrwm") {
        Some("Spectrwm".to_string())
    } else if package_installed("ratpoison") {
        Some("Ratpoison".to_string())
    } else if package_installed("wmii") {
        Some("Wmii".to_string())
    } else if package_installed("stumpwm") {
        Some("StumpWM".to_string())
    } else if package_installed("notion") {
        Some("Notion".to_string())
    // Stacking/floating WMs
    } else if package_installed("awesome") {
        Some("Awesome".to_string())
    } else if package_installed("openbox") {
        Some("Openbox".to_string())
    } else if package_installed("fluxbox") {
        Some("Fluxbox".to_string())
    } else if package_installed("blackbox") {
        Some("Blackbox".to_string())
    } else if package_installed("icewm") {
        Some("IceWM".to_string())
    } else if package_installed("jwm") {
        Some("JWM".to_string())
    } else if package_installed("enlightenment") {
        Some("Enlightenment".to_string())
    } else if package_installed("fvwm") || package_installed("fvwm3") {
        Some("FVWM".to_string())
    } else if package_installed("windowmaker") {
        Some("Window Maker".to_string())
    } else if package_installed("pekwm") {
        Some("PekWM".to_string())
    } else if package_installed("evilwm") {
        Some("EvilWM".to_string())
    } else if package_installed("cwm") {
        Some("cwm".to_string())
    } else if package_installed("ctwm") {
        Some("CTWM".to_string())
    } else if package_installed("afterstep") {
        Some("AfterStep".to_string())
    } else if package_installed("sawfish") {
        Some("Sawfish".to_string())
    } else if package_installed("twm") || package_installed("xorg-twm") {
        Some("twm".to_string())
    // Desktop environment WMs
    } else if package_installed("kwin") {
        Some("KWin".to_string())
    } else if package_installed("mutter") {
        Some("Mutter".to_string())
    } else if package_installed("marco") {
        Some("Marco".to_string())
    } else if package_installed("xfwm4") {
        Some("Xfwm".to_string())
    } else if package_installed("muffin") {
        Some("Muffin".to_string())
    } else if package_installed("metacity") {
        Some("Metacity".to_string())
    } else if package_installed("gala") {
        Some("Gala".to_string())
    } else if package_installed("compiz") {
        Some("Compiz".to_string())
    } else {
        None
    }
}

/// Detect compositor
fn detect_compositor() -> Option<String> {
    // Hyprland is both WM and compositor
    if let Ok(output) = Command::new("pgrep").arg("-x").arg("Hyprland").output() {
        if output.status.success() {
            return Some("Hyprland".to_string());
        }
    }

    // Check for standalone compositors
    let compositors = vec![
        ("picom", "picom"),
        ("compton", "compton"),
        ("xcompmgr", "xcompmgr"),
    ];

    for (process, name) in compositors {
        if let Ok(output) = Command::new("pgrep").arg("-x").arg(process).output() {
            if output.status.success() {
                return Some(name.to_string());
            }
        }
    }

    None
}

/// Detect if system has Nvidia GPU
fn detect_nvidia() -> bool {
    // Check lspci for Nvidia GPU - line by line for consistency and accuracy
    if let Ok(output) = Command::new("lspci").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let lower = line.to_lowercase();
            // Only check for Nvidia on actual GPU lines
            if (lower.contains("vga") || lower.contains("display") || lower.contains("3d"))
                && (lower.contains("nvidia"))
            {
                return true;
            }
        }
    }

    // Check if nvidia driver module is loaded
    if let Ok(output) = Command::new("lsmod").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("nvidia") {
            return true;
        }
    }

    // Check for nvidia-smi command
    if let Ok(output) = Command::new("which").arg("nvidia-smi").output() {
        if output.status.success() {
            return true;
        }
    }

    false
}

/// Get Nvidia driver version
fn get_nvidia_driver_version() -> Option<String> {
    if let Ok(output) = Command::new("nvidia-smi")
        .arg("--query-gpu=driver_version")
        .arg("--format=csv,noheader")
        .output()
    {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            return Some(version.trim().to_string());
        }
    }

    // Try alternative method
    if let Ok(output) = Command::new("modinfo").arg("nvidia").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.starts_with("version:") {
                if let Some(version) = line.split_whitespace().nth(1) {
                    return Some(version.to_string());
                }
            }
        }
    }

    None
}

/// Check if Wayland+Nvidia is properly configured
fn check_wayland_nvidia_support() -> bool {
    // Check if using Wayland
    if detect_display_server().as_deref() != Some("wayland") {
        return false; // Not using Wayland
    }

    // Check if Nvidia GPU present
    if !detect_nvidia() {
        return false; // No Nvidia GPU
    }

    // Check for required environment variables
    let required_vars = vec![
        "GBM_BACKEND",
        "__GLX_VENDOR_LIBRARY_NAME",
        "LIBVA_DRIVER_NAME",
    ];

    let mut found_vars = 0;

    // 1. Check window manager config files (Hyprland, Sway, etc.)
    let wm = anna_common::detect_window_manager();
    if wm != anna_common::WindowManager::Unknown {
        let mut parser = anna_common::ConfigParser::new(wm);
        if parser.load().is_ok() {
            for var in &required_vars {
                if parser.has_env_var(var) {
                    found_vars += 1;
                }
            }
        }
    }

    // 2. Check system-wide config files
    let system_config_files = vec!["/etc/environment", "/etc/profile.d/nvidia.sh"];

    for file in &system_config_files {
        if let Ok(content) = std::fs::read_to_string(file) {
            for var in &required_vars {
                if content.contains(var) {
                    found_vars += 1;
                }
            }
        }
    }

    // Consider it configured if at least 2 out of 3 vars are set
    found_vars >= 2
}

/// Detect if system has Intel integrated graphics (beta.41+)
fn detect_intel_gpu() -> bool {
    // Check lspci for Intel GPU - must check line by line to avoid false positives
    // (e.g., Intel chipset/audio on Nvidia GPU systems)
    if let Ok(output) = Command::new("lspci").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let lower = line.to_lowercase();
            // Only check for Intel on actual GPU lines (VGA/Display/3D controller lines)
            if (lower.contains("vga") || lower.contains("display") || lower.contains("3d"))
                && lower.contains("intel")
            {
                return true;
            }
        }
    }

    // Check if i915 kernel module is loaded (Intel driver)
    if let Ok(output) = Command::new("lsmod").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("i915") {
            return true;
        }
    }

    false
}

/// Detect if system has AMD/ATI GPU (beta.41+)
fn detect_amd_gpu() -> bool {
    // Check lspci for AMD/ATI GPU - must check line by line to avoid false positives
    if let Ok(output) = Command::new("lspci").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let lower = line.to_lowercase();
            // Only check for AMD/ATI/Radeon on actual GPU lines
            if (lower.contains("vga") || lower.contains("display") || lower.contains("3d"))
                && (lower.contains("amd") || lower.contains("ati") || lower.contains("radeon"))
            {
                return true;
            }
        }
    }

    // Check if amdgpu or radeon kernel module is loaded
    if let Ok(output) = Command::new("lsmod").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("amdgpu") || stdout.contains("radeon") {
            return true;
        }
    }

    false
}

/// Get AMD driver version (beta.41+)
fn get_amd_driver_version() -> Option<String> {
    // Check which driver is loaded
    if let Ok(output) = Command::new("lsmod").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("amdgpu") {
            return Some("amdgpu (modern)".to_string());
        } else if stdout.contains("radeon") {
            return Some("radeon (legacy)".to_string());
        }
    }

    None
}

async fn analyze_command_history() -> Vec<CommandUsage> {
    let mut command_counts: HashMap<String, usize> = HashMap::new();

    // Try bash history
    if let Ok(history) = tokio::fs::read_to_string("/root/.bash_history").await {
        for line in history.lines().take(1000) {
            if let Some(cmd) = line.split_whitespace().next() {
                *command_counts.entry(cmd.to_string()).or_insert(0) += 1;
            }
        }
    }

    let mut usage: Vec<CommandUsage> = command_counts
        .into_iter()
        .map(|(command, count)| CommandUsage { command, count })
        .collect();

    usage.sort_by(|a, b| b.count.cmp(&a.count));
    usage.truncate(20);

    usage
}

fn detect_dev_tools() -> Vec<String> {
    let tools = vec![
        "git", "docker", "cargo", "python3", "node", "npm", "go", "java", "gcc", "vim", "nvim",
        "code",
    ];

    tools
        .iter()
        .filter(|tool| command_exists(tool))
        .map(|s| s.to_string())
        .collect()
}

fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

async fn analyze_media_usage() -> MediaUsageProfile {
    MediaUsageProfile {
        has_video_files: has_media_files("/root", &[".mp4", ".mkv", ".avi"]).await,
        has_audio_files: has_media_files("/root", &[".mp3", ".flac", ".ogg"]).await,
        has_images: has_media_files("/root", &[".jpg", ".png", ".gif"]).await,
        video_player_installed: package_installed("mpv") || package_installed("vlc"),
        audio_player_installed: package_installed("rhythmbox") || package_installed("clementine"),
        image_viewer_installed: package_installed("eog") || package_installed("feh"),
    }
}

async fn has_media_files(base: &str, extensions: &[&str]) -> bool {
    let media_dirs = vec!["Videos", "Music", "Pictures", "Downloads"];

    for dir_name in media_dirs {
        let path = Path::new(base).join(dir_name);
        if path.exists() {
            if let Ok(mut entries) = tokio::fs::read_dir(&path).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if let Some(ext) = entry.path().extension() {
                        let ext_str = format!(".{}", ext.to_string_lossy());
                        if extensions.iter().any(|e| e.eq_ignore_ascii_case(&ext_str)) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

async fn detect_common_file_types() -> Vec<String> {
    let mut types = Vec::new();

    if has_media_files("/root", &[".py"]).await {
        types.push("python".to_string());
    }
    if has_media_files("/root", &[".rs"]).await {
        types.push("rust".to_string());
    }
    if has_media_files("/root", &[".js", ".ts"]).await {
        types.push("javascript".to_string());
    }
    if has_media_files("/root", &[".go"]).await {
        types.push("go".to_string());
    }

    types
}

/// Enhanced: Analyze process CPU time to understand user behavior
#[allow(dead_code)]
pub async fn analyze_process_cpu_time() -> Vec<ProcessUsage> {
    let mut process_usage = Vec::new();

    // Get list of processes sorted by CPU time
    if let Ok(output) = Command::new("ps").args(&["aux", "--sort=-time"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().skip(1).take(50) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 11 {
                let user = parts[0];
                let cpu_percent = parts[2].parse::<f32>().unwrap_or(0.0);
                let mem_percent = parts[3].parse::<f32>().unwrap_or(0.0);
                let time = parts[9]; // CPU time
                let command = parts[10..].join(" ");

                // Filter out system processes, focus on user processes
                if user != "root" && cpu_percent > 0.1 {
                    process_usage.push(ProcessUsage {
                        command: command.clone(),
                        user: user.to_string(),
                        cpu_percent,
                        mem_percent,
                        cpu_time: time.to_string(),
                    });
                }
            }
        }
    }

    process_usage
}

/// Process usage information for behavior analysis
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ProcessUsage {
    pub command: String,
    pub user: String,
    pub cpu_percent: f32,
    pub mem_percent: f32,
    pub cpu_time: String,
}

/// Enhanced: Deep bash history analysis with frequency, recency, and patterns
#[allow(dead_code)]
pub async fn analyze_bash_history_deep() -> BashHistoryAnalysis {
    let mut analysis = BashHistoryAnalysis::default();

    // Analyze all users' bash/zsh history
    if let Ok(entries) = std::fs::read_dir("/home") {
        for entry in entries.filter_map(|e| e.ok()) {
            let username = entry.file_name().to_string_lossy().to_string();
            let home_dir = entry.path();

            // Try bash history
            let bash_hist = home_dir.join(".bash_history");
            if bash_hist.exists() {
                if let Ok(contents) = tokio::fs::read_to_string(&bash_hist).await {
                    analysis.parse_history(&contents, &username);
                }
            }

            // Try zsh history
            let zsh_hist = home_dir.join(".zsh_history");
            if zsh_hist.exists() {
                if let Ok(contents) = tokio::fs::read_to_string(&zsh_hist).await {
                    analysis.parse_history(&contents, &username);
                }
            }
        }
    }

    // Also check root
    if let Ok(contents) = tokio::fs::read_to_string("/root/.bash_history").await {
        analysis.parse_history(&contents, "root");
    }

    analysis.calculate_scores();
    analysis
}

/// Comprehensive bash history analysis
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct BashHistoryAnalysis {
    pub command_frequency: HashMap<String, usize>,
    pub tool_categories: HashMap<String, Vec<String>>, // category -> tools
    pub workflow_patterns: Vec<WorkflowPattern>,
    pub total_commands: usize,
    pub unique_commands: usize,
}

impl BashHistoryAnalysis {
    #[allow(dead_code)]
    fn parse_history(&mut self, contents: &str, _username: &str) {
        for line in contents.lines() {
            self.total_commands += 1;

            // Handle zsh history format (: timestamp:duration;command)
            let command_line = if line.starts_with(':') {
                line.split(';').nth(1).unwrap_or(line)
            } else {
                line
            };

            if let Some(cmd) = command_line.split_whitespace().next() {
                *self.command_frequency.entry(cmd.to_string()).or_insert(0) += 1;

                // Categorize tools
                self.categorize_tool(cmd);
            }
        }

        self.unique_commands = self.command_frequency.len();
    }

    #[allow(dead_code)]
    fn categorize_tool(&mut self, cmd: &str) {
        let category = match cmd {
            "vim" | "nvim" | "nano" | "emacs" | "code" => "editor",
            "git" | "gh" | "gitlab" => "vcs",
            "docker" | "podman" | "kubectl" => "container",
            "cargo" | "rustc" | "npm" | "yarn" | "python" | "python3" | "pip" | "go" | "gcc"
            | "make" => "development",
            "pacman" | "yay" | "paru" => "package_manager",
            "ssh" | "scp" | "rsync" | "curl" | "wget" => "network",
            "systemctl" | "journalctl" | "dmesg" => "system_admin",
            "grep" | "sed" | "awk" | "find" | "fd" | "rg" => "text_processing",
            "htop" | "top" | "ps" | "free" | "df" => "monitoring",
            _ => return,
        };

        self.tool_categories
            .entry(category.to_string())
            .or_insert_with(Vec::new)
            .push(cmd.to_string());
    }

    #[allow(dead_code)]
    fn calculate_scores(&mut self) {
        // Detect workflow patterns
        if self.command_frequency.get("git").unwrap_or(&0) > &20 {
            self.workflow_patterns.push(WorkflowPattern {
                name: "Version Control Heavy".to_string(),
                confidence: 0.9,
                evidence: format!(
                    "git used {} times",
                    self.command_frequency.get("git").unwrap()
                ),
            });
        }

        if self.command_frequency.get("docker").unwrap_or(&0) > &10 {
            self.workflow_patterns.push(WorkflowPattern {
                name: "Container Development".to_string(),
                confidence: 0.85,
                evidence: format!(
                    "docker used {} times",
                    self.command_frequency.get("docker").unwrap()
                ),
            });
        }

        let dev_tools = ["cargo", "npm", "python", "go", "gcc", "make"];
        let dev_count: usize = dev_tools
            .iter()
            .map(|t| self.command_frequency.get(*t).unwrap_or(&0))
            .sum();

        if dev_count > 30 {
            self.workflow_patterns.push(WorkflowPattern {
                name: "Software Development".to_string(),
                confidence: 0.95,
                evidence: format!("Development tools used {} times", dev_count),
            });
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WorkflowPattern {
    pub name: String,
    pub confidence: f32, // 0.0 to 1.0
    pub evidence: String,
}

/// Deep system configuration analysis - sysadmin perspective
#[allow(dead_code)]
pub async fn analyze_system_configuration() -> SystemConfigAnalysis {
    let mut analysis = SystemConfigAnalysis::default();

    // Analyze bootloader
    analysis.bootloader = detect_bootloader();

    // Analyze init system (should be systemd on Arch)
    analysis.init_system = detect_init_system();

    // Analyze failed services
    analysis.failed_services = get_failed_services();

    // Analyze security: firewall status
    analysis.firewall_active = check_firewall_active();

    // Analyze SELinux/AppArmor
    analysis.mac_system = detect_mac_system();

    // Check for swap
    analysis.swap_info = analyze_swap();

    // Check systemd boot time (store as String for the old struct)
    analysis.boot_time = get_boot_time()
        .map(|t| format!("{:.2}s", t))
        .unwrap_or_else(|| "Unknown".to_string());

    // Analyze disk I/O scheduler
    analysis.io_schedulers = get_io_schedulers();

    // Check kernel parameters
    analysis.important_kernel_params = get_important_kernel_params();

    analysis
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct SystemConfigAnalysis {
    pub bootloader: String,
    pub init_system: String,
    pub failed_services: Vec<String>,
    pub firewall_active: bool,
    pub mac_system: Option<String>, // SELinux, AppArmor, etc.
    pub swap_info: SwapInfo,
    pub boot_time: String,
    pub io_schedulers: HashMap<String, String>, // device -> scheduler
    pub important_kernel_params: HashMap<String, String>,
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct SwapInfo {
    pub enabled: bool,
    pub total_mb: u64,
    pub used_mb: u64,
    pub swappiness: u32,
    pub zswap_enabled: bool,
}

#[allow(dead_code)]
fn detect_bootloader() -> String {
    if Path::new("/boot/grub").exists() {
        "GRUB".to_string()
    } else if Path::new("/boot/loader/entries").exists() {
        "systemd-boot".to_string()
    } else if Path::new("/boot/refind_linux.conf").exists() {
        "rEFInd".to_string()
    } else {
        "Unknown".to_string()
    }
}

#[allow(dead_code)]
fn detect_init_system() -> String {
    if Path::new("/run/systemd/system").exists() {
        "systemd".to_string()
    } else {
        "Unknown".to_string()
    }
}

#[allow(dead_code)]
fn check_firewall_active() -> bool {
    Command::new("systemctl")
        .args(&["is-active", "ufw"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
        || Command::new("systemctl")
            .args(&["is-active", "firewalld"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
}

#[allow(dead_code)]
fn detect_mac_system() -> Option<String> {
    if Path::new("/sys/fs/selinux").exists() {
        Some("SELinux".to_string())
    } else if Path::new("/sys/kernel/security/apparmor").exists() {
        Some("AppArmor".to_string())
    } else {
        None
    }
}

#[allow(dead_code)]
fn analyze_swap() -> SwapInfo {
    let mut info = SwapInfo::default();

    // Check /proc/swaps
    if let Ok(swaps) = std::fs::read_to_string("/proc/swaps") {
        info.enabled = swaps.lines().count() > 1; // Header + entries

        for line in swaps.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                info.total_mb += parts[2].parse::<u64>().unwrap_or(0) / 1024;
                info.used_mb += parts[3].parse::<u64>().unwrap_or(0) / 1024;
            }
        }
    }

    // Check swappiness
    if let Ok(swappiness) = std::fs::read_to_string("/proc/sys/vm/swappiness") {
        info.swappiness = swappiness.trim().parse().unwrap_or(60);
    }

    // Check zswap
    if let Ok(enabled) = std::fs::read_to_string("/sys/module/zswap/parameters/enabled") {
        info.zswap_enabled = enabled.trim() == "Y";
    }

    info
}

#[allow(dead_code)]
fn get_io_schedulers() -> HashMap<String, String> {
    let mut schedulers = HashMap::new();

    if let Ok(entries) = std::fs::read_dir("/sys/block") {
        for entry in entries.filter_map(|e| e.ok()) {
            let device = entry.file_name().to_string_lossy().to_string();

            // Skip loop devices and partitions
            if device.starts_with("loop")
                || device
                    .chars()
                    .last()
                    .map(|c| c.is_numeric())
                    .unwrap_or(false)
            {
                continue;
            }

            let scheduler_path = entry.path().join("queue/scheduler");
            if let Ok(scheduler) = std::fs::read_to_string(scheduler_path) {
                // Extract current scheduler (marked with [brackets])
                if let Some(current) = scheduler
                    .split_whitespace()
                    .find(|s| s.starts_with('[') && s.ends_with(']'))
                {
                    schedulers.insert(
                        device,
                        current.trim_matches(|c| c == '[' || c == ']').to_string(),
                    );
                }
            }
        }
    }

    schedulers
}

#[allow(dead_code)]
fn get_important_kernel_params() -> HashMap<String, String> {
    let mut params = HashMap::new();

    // Read kernel command line
    if let Ok(cmdline) = std::fs::read_to_string("/proc/cmdline") {
        params.insert("cmdline".to_string(), cmdline.trim().to_string());
    }

    // Check important sysctl values
    let important_sysctls = vec![
        "/proc/sys/kernel/printk",
        "/proc/sys/vm/swappiness",
        "/proc/sys/net/ipv4/ip_forward",
    ];

    for path in important_sysctls {
        if let Ok(value) = std::fs::read_to_string(path) {
            let key = Path::new(path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();
            params.insert(key, value.trim().to_string());
        }
    }

    params
}

/// Get boot time in seconds using systemd-analyze
fn get_boot_time() -> Option<f64> {
    let output = Command::new("systemd-analyze").arg("time").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    // Parse output like: "Startup finished in 2.153s (kernel) + 15.234s (userspace) = 17.387s"
    for line in text.lines() {
        if line.contains("=") {
            if let Some(total_part) = line.split('=').nth(1) {
                let time_str = total_part.trim().replace("s", "");
                if let Ok(seconds) = time_str.parse::<f64>() {
                    return Some(seconds);
                }
            }
        }
    }

    None
}

/// Get services that take longer than 5 seconds to start
fn get_slow_services() -> Vec<SystemdService> {
    let mut services = Vec::new();

    let output = match Command::new("systemd-analyze").arg("blame").output() {
        Ok(o) if o.status.success() => o,
        _ => return services,
    };

    let text = String::from_utf8_lossy(&output.stdout);

    for line in text.lines().take(20) {
        // Only check top 20 slowest
        // Parse lines like: "15.234s NetworkManager.service"
        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        if parts.len() >= 2 {
            let time_str = parts[0].replace("ms", "").replace("s", "");
            if let Ok(mut time) = time_str.parse::<f64>() {
                // Convert ms to seconds if needed
                if parts[0].contains("ms") {
                    time /= 1000.0;
                }

                if time >= 5.0 {
                    services.push(SystemdService {
                        name: parts[1].to_string(),
                        time_seconds: time,
                    });
                }
            }
        }
    }

    services
}

/// Get list of failed systemd services
fn get_failed_services() -> Vec<String> {
    let mut failed = Vec::new();

    let output = match Command::new("systemctl")
        .args(&["--failed", "--no-pager", "--no-legend"])
        .output()
    {
        Ok(o) => o,
        _ => return failed,
    };

    let text = String::from_utf8_lossy(&output.stdout);

    for line in text.lines() {
        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        if !parts.is_empty() {
            failed.push(parts[0].to_string());
        }
    }

    failed
}

/// Count AUR packages by checking /var/lib/pacman/local for packages not in official repos
fn count_aur_packages() -> usize {
    // Quick approximation: check for common AUR helpers first
    let aur_list = Command::new("pacman")
        .args(&["-Qm"]) // List foreign packages (not in sync database)
        .output();

    if let Ok(output) = aur_list {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).lines().count();
        }
    }

    0
}

/// Detect which AUR helper is installed
fn detect_aur_helper() -> Option<String> {
    let helpers = vec!["yay", "paru", "aurutils", "pikaur", "aura", "trizen"];

    for helper in helpers {
        if Command::new("which")
            .arg(helper)
            .output()
            .ok()?
            .status
            .success()
        {
            return Some(helper.to_string());
        }
    }

    None
}

/// Get package cache size in GB
fn get_package_cache_size() -> f64 {
    let output = Command::new("du")
        .args(&["-sb", "/var/cache/pacman/pkg"])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            if let Some(size_str) = text.split_whitespace().next() {
                if let Ok(bytes) = size_str.parse::<u64>() {
                    return bytes as f64 / 1024.0 / 1024.0 / 1024.0; // Convert to GB
                }
            }
        }
    }

    0.0
}

/// Get last system upgrade time from pacman log
fn get_last_upgrade_time() -> Option<chrono::DateTime<chrono::Utc>> {
    let log_path = "/var/log/pacman.log";
    let contents = std::fs::read_to_string(log_path).ok()?;

    // Find the most recent "starting full system upgrade" or "upgraded" entry
    for line in contents.lines().rev() {
        if line.contains("starting full system upgrade") || line.contains("upgraded") {
            // Parse timestamp like: [2025-01-04T17:23:45+0000]
            if let Some(timestamp_str) = line.split('[').nth(1) {
                if let Some(timestamp) = timestamp_str.split(']').next() {
                    // Parse ISO 8601 timestamp
                    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
                        return Some(dt.with_timezone(&chrono::Utc));
                    }
                }
            }
        }
    }

    None
}

/// Get kernel parameters from /proc/cmdline
fn get_kernel_parameters() -> Vec<String> {
    if let Ok(cmdline) = std::fs::read_to_string("/proc/cmdline") {
        return cmdline
            .trim()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
    }

    Vec::new()
}
// New comprehensive telemetry functions

use anna_common::{
    DevelopmentProfile, DirectorySize, DiskUsageTrend, GamingProfile, LanguageUsage,
    NetworkProfile, PackageInstallation, SessionInfo, UserPreferences,
};

/// Get recently installed packages (last 30 days)
fn get_recently_installed_packages() -> Vec<PackageInstallation> {
    let mut packages = Vec::new();

    // Parse /var/log/pacman.log for installations
    if let Ok(log_content) = std::fs::read_to_string("/var/log/pacman.log") {
        let _thirty_days_ago = chrono::Utc::now() - chrono::Duration::days(30);

        for line in log_content.lines() {
            if line.contains("installed") {
                // Parse format: [2025-01-04T12:34:56-0800] [ALPM] installed package (version)
                if let Some(pkg_info) = line.split("installed ").nth(1) {
                    let pkg_name = pkg_info.split_whitespace().next().unwrap_or("").to_string();

                    // Try to parse timestamp
                    if let Some(timestamp_str) = line.split('[').nth(1) {
                        if let Some(_ts) = timestamp_str.split(']').next() {
                            // Simple approach: just add recent packages
                            packages.push(PackageInstallation {
                                name: pkg_name,
                                installed_at: chrono::Utc::now(), // Simplified for now
                                from_aur: false,                  // We'll detect this separately
                            });
                        }
                    }
                }
            }
        }
    }

    // Limit to last 100 for performance
    packages.truncate(100);
    packages
}

/// Get currently active systemd services
fn get_active_services() -> Vec<String> {
    let output = Command::new("systemctl")
        .args(&[
            "list-units",
            "--type=service",
            "--state=running",
            "--no-pager",
            "--no-legend",
        ])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return stdout
            .lines()
            .filter_map(|line| line.split_whitespace().next().map(|s| s.to_string()))
            .collect();
    }

    Vec::new()
}

/// Get services enabled on boot
fn get_enabled_services() -> Vec<String> {
    let output = Command::new("systemctl")
        .args(&[
            "list-unit-files",
            "--type=service",
            "--state=enabled",
            "--no-pager",
            "--no-legend",
        ])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return stdout
            .lines()
            .filter_map(|line| line.split_whitespace().next().map(|s| s.to_string()))
            .collect();
    }

    Vec::new()
}

/// Analyze disk usage trends
fn analyze_disk_usage() -> DiskUsageTrend {
    // Get total disk usage
    let mut total_gb = 0.0;
    let mut used_gb = 0.0;

    if let Ok(output) = Command::new("df").args(&["-B1", "/"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = stdout.lines().nth(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                total_gb = parts[1].parse::<f64>().unwrap_or(0.0) / 1024.0 / 1024.0 / 1024.0;
                used_gb = parts[2].parse::<f64>().unwrap_or(0.0) / 1024.0 / 1024.0 / 1024.0;
            }
        }
    }

    // Get largest directories
    let largest_dirs = get_largest_directories();

    // Get cache size
    let cache_size_gb = get_directory_size_gb("/var/cache");

    // Get log size
    let log_size_gb = get_directory_size_gb("/var/log");

    DiskUsageTrend {
        total_gb,
        used_gb,
        largest_directories: largest_dirs,
        cache_size_gb,
        log_size_gb,
    }
}

/// Get largest directories in home
fn get_largest_directories() -> Vec<DirectorySize> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
    let mut dirs = Vec::new();

    // Use du to find large directories (only first level for performance)
    if let Ok(output) = Command::new("du")
        .args(&["-sh", "--threshold=100M", &home])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().take(10) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let size_str = parts[0];
                let path = parts[1].to_string();

                // Convert size to GB (handles M, G suffixes)
                let size_gb = parse_size_to_gb(size_str);

                dirs.push(DirectorySize { path, size_gb });
            }
        }
    }

    dirs
}

/// Helper to parse size strings like "1.5G" or "500M" to GB
fn parse_size_to_gb(size_str: &str) -> f64 {
    let size_str = size_str.trim();
    if size_str.ends_with('G') {
        size_str.trim_end_matches('G').parse().unwrap_or(0.0)
    } else if size_str.ends_with('M') {
        size_str.trim_end_matches('M').parse::<f64>().unwrap_or(0.0) / 1024.0
    } else {
        0.0
    }
}

/// Get directory size in GB
fn get_directory_size_gb(path: &str) -> f64 {
    if let Ok(output) = Command::new("du").args(&["-sb", path]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = stdout.lines().next() {
            if let Some(size_str) = line.split_whitespace().next() {
                if let Ok(bytes) = size_str.parse::<f64>() {
                    return bytes / 1024.0 / 1024.0 / 1024.0;
                }
            }
        }
    }
    0.0
}

/// Collect session information
fn collect_session_info() -> SessionInfo {
    let current_user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());

    // Count logins from wtmp (last 30 days)
    let login_count = count_recent_logins(&current_user);

    // Check for multiple users
    let multiple_users = has_multiple_users();

    SessionInfo {
        current_user,
        login_count_last_30_days: login_count,
        average_session_hours: 4.0, // Placeholder - needs more complex tracking
        last_login: None,           // Placeholder
        multiple_users,
    }
}

/// Count recent logins for user
fn count_recent_logins(username: &str) -> usize {
    if let Ok(output) = Command::new("last")
        .args(&[username, "-s", "-30days"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return stdout.lines().filter(|line| !line.is_empty()).count();
    }
    0
}

/// Check if system has multiple user accounts
fn has_multiple_users() -> bool {
    if let Ok(passwd) = std::fs::read_to_string("/etc/passwd") {
        let user_count = passwd
            .lines()
            .filter(|line| {
                // Count only real users (UID >= 1000)
                if let Some(uid_str) = line.split(':').nth(2) {
                    if let Ok(uid) = uid_str.parse::<u32>() {
                        return uid >= 1000 && uid < 60000;
                    }
                }
                false
            })
            .count();
        return user_count > 1;
    }
    false
}

/// Analyze development environment asynchronously
async fn analyze_development_environment() -> DevelopmentProfile {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());

    // Detect languages
    let languages = detect_programming_languages(&home).await;

    // Detect IDEs
    let ides_installed = detect_ides();

    // Count git repos
    let git_repos_count = count_git_repos(&home).await;

    // Detect containerization
    let uses_containers = Command::new("which")
        .arg("docker")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
        || Command::new("which")
            .arg("podman")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

    // Detect virtualization
    let uses_virtualization = Command::new("which")
        .arg("qemu-system-x86_64")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
        || Command::new("which")
            .arg("virtualbox")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

    DevelopmentProfile {
        languages,
        ides_installed,
        active_projects: Vec::new(), // Placeholder - complex to detect
        uses_containers,
        uses_virtualization,
        git_repos_count,
    }
}

/// Detect programming languages used
async fn detect_programming_languages(home_dir: &str) -> Vec<LanguageUsage> {
    let mut languages = Vec::new();

    // Python
    let py_count = count_files_by_extension(home_dir, "py").await;
    if py_count > 0 {
        languages.push(LanguageUsage {
            language: "Python".to_string(),
            project_count: 0, // Placeholder
            file_count: py_count,
            has_lsp: check_package_installed("python-lsp-server"),
        });
    }

    // Rust
    let rs_count = count_files_by_extension(home_dir, "rs").await;
    if rs_count > 0 {
        languages.push(LanguageUsage {
            language: "Rust".to_string(),
            project_count: 0,
            file_count: rs_count,
            has_lsp: check_package_installed("rust-analyzer"),
        });
    }

    // JavaScript/TypeScript
    let js_count = count_files_by_extension(home_dir, "js").await
        + count_files_by_extension(home_dir, "ts").await;
    if js_count > 0 {
        languages.push(LanguageUsage {
            language: "JavaScript/TypeScript".to_string(),
            project_count: 0,
            file_count: js_count,
            has_lsp: check_package_installed("typescript-language-server"),
        });
    }

    // Go
    let go_count = count_files_by_extension(home_dir, "go").await;
    if go_count > 0 {
        languages.push(LanguageUsage {
            language: "Go".to_string(),
            project_count: 0,
            file_count: go_count,
            has_lsp: check_package_installed("gopls"),
        });
    }

    languages
}

/// Count files by extension (limited search for performance)
async fn count_files_by_extension(base_dir: &str, extension: &str) -> usize {
    // Use find command with limits for performance
    if let Ok(output) = Command::new("find")
        .args(&[
            base_dir,
            "-maxdepth",
            "4",
            "-name",
            &format!("*.{}", extension),
            "-type",
            "f",
        ])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return stdout.lines().count();
    }
    0
}

/// Check if package is installed
fn check_package_installed(package: &str) -> bool {
    Command::new("pacman")
        .args(&["-Q", package])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Detect installed IDEs
fn detect_ides() -> Vec<String> {
    let mut ides = Vec::new();

    let ide_list = vec![
        ("code", "VSCode"),
        ("vim", "Vim"),
        ("nvim", "Neovim"),
        ("emacs", "Emacs"),
        ("idea", "IntelliJ IDEA"),
        ("pycharm", "PyCharm"),
        ("clion", "CLion"),
    ];

    for (cmd, name) in ide_list {
        if Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            ides.push(name.to_string());
        }
    }

    ides
}

/// Count git repositories
async fn count_git_repos(home_dir: &str) -> usize {
    // Find .git directories
    if let Ok(output) = Command::new("find")
        .args(&[home_dir, "-maxdepth", "4", "-type", "d", "-name", ".git"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return stdout.lines().count();
    }
    0
}

/// Analyze gaming profile
fn analyze_gaming_profile() -> GamingProfile {
    GamingProfile {
        steam_installed: check_package_installed("steam"),
        lutris_installed: check_package_installed("lutris"),
        wine_installed: check_package_installed("wine"),
        proton_ge_installed: check_package_installed("proton-ge-custom"),
        mangohud_installed: check_package_installed("mangohud"),
        game_count: 0, // Placeholder - would need to scan Steam library
        uses_gamepad: check_gamepad_drivers(),
    }
}

/// Check if gamepad drivers are installed
fn check_gamepad_drivers() -> bool {
    check_package_installed("xpadneo")
        || check_package_installed("xpad")
        || check_package_installed("hid-nintendo")
}

/// Analyze network profile
fn analyze_network_profile() -> NetworkProfile {
    NetworkProfile {
        vpn_configured: check_vpn_configured(),
        firewall_active: check_firewall_active(),
        ssh_server_running: check_service_running("sshd"),
        has_ssh_client_keys: check_ssh_client_keys(),
        has_static_ip: false, // Placeholder - complex to detect reliably
        dns_configuration: detect_dns_config(),
        uses_network_share: check_network_shares(),
    }
}

/// Check if VPN is configured
fn check_vpn_configured() -> bool {
    check_package_installed("wireguard-tools")
        || check_package_installed("openvpn")
        || std::path::Path::new("/etc/wireguard").exists()
}

/// Check if service is running
fn check_service_running(service: &str) -> bool {
    Command::new("systemctl")
        .args(&["is-active", service])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Detect DNS configuration
fn detect_dns_config() -> String {
    if check_service_running("systemd-resolved") {
        "systemd-resolved".to_string()
    } else if check_package_installed("dnsmasq") {
        "dnsmasq".to_string()
    } else {
        "default".to_string()
    }
}

/// Check for network shares
fn check_network_shares() -> bool {
    if let Ok(mounts) = std::fs::read_to_string("/proc/mounts") {
        return mounts.contains("nfs") || mounts.contains("cifs");
    }
    false
}

/// Check if user has SSH client keys in ~/.ssh/
fn check_ssh_client_keys() -> bool {
    if let Ok(home) = std::env::var("HOME") {
        let ssh_dir = std::path::PathBuf::from(format!("{}/.ssh", home));
        if !ssh_dir.exists() {
            return false;
        }

        // Check for common SSH key types
        let key_files = [
            "id_ed25519", // Ed25519 (modern, recommended)
            "id_rsa",     // RSA (common)
            "id_ecdsa",   // ECDSA (older but still used)
            "id_dsa",     // DSA (very old, deprecated)
        ];

        for key_file in &key_files {
            let key_path = ssh_dir.join(key_file);
            if key_path.exists() {
                return true;
            }
        }
    }
    false
}

/// Get system age in days
fn get_system_age_days() -> u64 {
    // Check installation timestamp from /var/log/pacman.log
    if let Ok(metadata) = std::fs::metadata("/var/log/pacman.log") {
        if let Ok(created) = metadata.created() {
            if let Ok(duration) = created.elapsed() {
                return duration.as_secs() / 86400;
            }
        }
    }

    // Fallback: check root filesystem age
    if let Ok(metadata) = std::fs::metadata("/") {
        if let Ok(created) = metadata.created() {
            if let Ok(duration) = created.elapsed() {
                return duration.as_secs() / 86400;
            }
        }
    }

    0
}

/// Infer user preferences from system state
fn infer_user_preferences(dev_tools: &[String], package_count: usize) -> UserPreferences {
    // Check for CLI tools vs GUI tools
    let cli_tools = vec!["vim", "nvim", "emacs", "tmux", "screen", "htop", "btop"];
    let has_cli_tools = cli_tools
        .iter()
        .any(|tool| dev_tools.contains(&tool.to_string()));

    // Check for beautification tools
    let beauty_tools = vec!["starship", "eza", "bat", "fd", "ripgrep", "fzf"];
    let has_beauty_tools = beauty_tools
        .iter()
        .any(|tool| check_package_installed(tool));

    // Detect laptop
    let uses_laptop = std::path::Path::new("/sys/class/power_supply/BAT0").exists()
        || std::path::Path::new("/sys/class/power_supply/BAT1").exists();

    UserPreferences {
        prefers_cli_over_gui: has_cli_tools,
        is_power_user: dev_tools.len() > 10,
        values_aesthetics: has_beauty_tools,
        is_gamer: check_package_installed("steam") || check_package_installed("lutris"),
        is_developer: !dev_tools.is_empty(),
        is_content_creator: check_package_installed("obs-studio")
            || check_package_installed("kdenlive")
            || check_package_installed("gimp"),
        uses_laptop,
        prefers_minimalism: package_count < 500,
    }
}

/// Collect hardware monitoring data (beta.35+)
fn collect_hardware_monitoring(sys: &System) -> anna_common::HardwareMonitoring {
    use anna_common::HardwareMonitoring;

    // Get CPU temperature
    let cpu_temp = get_cpu_temperature();

    // Get load averages
    let load = get_load_averages();

    // Get memory info
    let memory_used_gb = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let memory_available_gb = sys.available_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let swap_used_gb = sys.used_swap() as f64 / 1024.0 / 1024.0 / 1024.0;
    let swap_total_gb = sys.total_swap() as f64 / 1024.0 / 1024.0 / 1024.0;

    // Get battery health
    let battery_health = get_battery_health();

    HardwareMonitoring {
        cpu_temperature_celsius: cpu_temp,
        cpu_load_1min: load.0,
        cpu_load_5min: load.1,
        cpu_load_15min: load.2,
        memory_used_gb,
        memory_available_gb,
        swap_used_gb,
        swap_total_gb,
        battery_health,
    }
}

/// Get CPU temperature from sensors
fn get_cpu_temperature() -> Option<f64> {
    let mut cpu_temps = Vec::new();

    // Try using sensors command (most reliable)
    if let Ok(output) = Command::new("sensors").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Parse temperature from sensors output
            // Look for lines like: "Core 0:        +45.0°C" or "Tctl:         +50.0°C"
            for line in stdout.lines() {
                // Only look at CPU-related sensors, skip GPU, disk, etc.
                if line.contains("Core")
                    || line.contains("Tctl")
                    || line.contains("Tccd")
                    || (line.contains("CPU") && !line.contains("fan"))
                {
                    if let Some(temp_str) = line.split('+').nth(1) {
                        if let Some(temp) = temp_str.split('°').next() {
                            if let Ok(temp_val) = temp.trim().parse::<f64>() {
                                // Sanity check: CPU temps should be between 0-120°C
                                if temp_val > 0.0 && temp_val < 120.0 {
                                    cpu_temps.push(temp_val);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // If we got CPU temps from sensors, return the maximum
    if !cpu_temps.is_empty() {
        return cpu_temps
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max)
            .into();
    }

    // Fallback: try reading from /sys/class/thermal (less reliable, might get GPU/other sensors)
    if let Ok(thermal_zones) = std::fs::read_dir("/sys/class/thermal") {
        for entry in thermal_zones.flatten() {
            let path = entry.path();
            if path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("thermal_zone")
            {
                // Check the type to ensure it's a CPU sensor
                let type_path = path.join("type");
                if let Ok(sensor_type) = std::fs::read_to_string(&type_path) {
                    let sensor_type = sensor_type.trim().to_lowercase();
                    // Only read if it's explicitly a CPU sensor
                    if sensor_type.contains("cpu")
                        || sensor_type.contains("x86_pkg_temp")
                        || sensor_type.contains("coretemp")
                    {
                        let temp_path = path.join("temp");
                        if let Ok(temp_str) = std::fs::read_to_string(&temp_path) {
                            if let Ok(temp_millidegrees) = temp_str.trim().parse::<i64>() {
                                let temp = temp_millidegrees as f64 / 1000.0;
                                if temp > 0.0 && temp < 120.0 {
                                    cpu_temps.push(temp);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Return maximum CPU temp if we found any
    if !cpu_temps.is_empty() {
        Some(cpu_temps.iter().copied().fold(f64::NEG_INFINITY, f64::max))
    } else {
        None
    }
}

/// Get system load averages
fn get_load_averages() -> (Option<f64>, Option<f64>, Option<f64>) {
    if let Ok(loadavg) = std::fs::read_to_string("/proc/loadavg") {
        let parts: Vec<&str> = loadavg.split_whitespace().collect();
        if parts.len() >= 3 {
            return (
                parts[0].parse().ok(),
                parts[1].parse().ok(),
                parts[2].parse().ok(),
            );
        }
    }
    (None, None, None)
}

/// Get battery health information
fn get_battery_health() -> Option<anna_common::BatteryHealth> {
    use anna_common::BatteryHealth;

    // Check for battery presence
    let bat_path = std::path::Path::new("/sys/class/power_supply/BAT0");
    let bat1_path = std::path::Path::new("/sys/class/power_supply/BAT1");

    let battery_dir = if bat_path.exists() {
        bat_path
    } else if bat1_path.exists() {
        bat1_path
    } else {
        return None;
    };

    // Read battery info
    let capacity = std::fs::read_to_string(battery_dir.join("capacity"))
        .ok()?
        .trim()
        .parse::<u8>()
        .ok()?;

    let status = std::fs::read_to_string(battery_dir.join("status"))
        .ok()?
        .trim()
        .to_string();

    // Try to get design capacity and current full capacity for health percentage
    let health_percentage = if let (Ok(energy_full), Ok(energy_full_design)) = (
        std::fs::read_to_string(battery_dir.join("energy_full")),
        std::fs::read_to_string(battery_dir.join("energy_full_design")),
    ) {
        if let (Ok(full), Ok(design)) = (
            energy_full.trim().parse::<f64>(),
            energy_full_design.trim().parse::<f64>(),
        ) {
            Some(((full / design) * 100.0) as u8)
        } else {
            None
        }
    } else {
        None
    };

    // Try to get cycle count
    let cycles = std::fs::read_to_string(battery_dir.join("cycle_count"))
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok());

    Some(BatteryHealth {
        percentage: capacity,
        status,
        health_percentage,
        cycles,
        is_critical: capacity < 20,
    })
}

/// Collect disk health information from SMART data
fn collect_disk_health() -> Vec<anna_common::DiskHealthInfo> {
    let mut disk_health = Vec::new();

    // Get list of block devices
    if let Ok(output) = Command::new("lsblk")
        .args(&["-d", "-n", "-o", "NAME,TYPE"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[1] == "disk" {
                let device = format!("/dev/{}", parts[0]);
                if let Some(health) = get_smart_data(&device) {
                    disk_health.push(health);
                }
            }
        }
    }

    disk_health
}

/// Get SMART data for a disk
fn get_smart_data(device: &str) -> Option<anna_common::DiskHealthInfo> {
    use anna_common::DiskHealthInfo;

    // Try using smartctl (requires smartmontools package)
    let output = Command::new("smartctl")
        .args(&["-H", "-A", device])
        .output()
        .ok()?;

    if !output.status.success() {
        return Some(DiskHealthInfo {
            device: device.to_string(),
            health_status: "UNKNOWN".to_string(),
            temperature_celsius: None,
            power_on_hours: None,
            wear_leveling: None,
            reallocated_sectors: None,
            pending_sectors: None,
            has_errors: false,
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse health status
    let health_status = if stdout.contains("PASSED") {
        "PASSED".to_string()
    } else if stdout.contains("FAILING") {
        "FAILING".to_string()
    } else {
        "UNKNOWN".to_string()
    };

    // Parse temperature
    let temperature_celsius = stdout
        .lines()
        .find(|l| l.contains("Temperature"))
        .and_then(|l| {
            l.split_whitespace()
                .find(|s| s.parse::<u8>().is_ok())
                .and_then(|s| s.parse::<u8>().ok())
        });

    // Parse power-on hours
    let power_on_hours = stdout
        .lines()
        .find(|l| l.contains("Power_On_Hours"))
        .and_then(|l| {
            l.split_whitespace()
                .nth(9)
                .and_then(|s| s.parse::<u64>().ok())
        });

    // Parse wear leveling (for SSDs)
    let wear_leveling = stdout
        .lines()
        .find(|l| l.contains("Wear_Leveling_Count"))
        .and_then(|l| {
            l.split_whitespace()
                .nth(3)
                .and_then(|s| s.parse::<u8>().ok())
        });

    // Parse reallocated sectors
    let reallocated_sectors = stdout
        .lines()
        .find(|l| l.contains("Reallocated_Sector"))
        .and_then(|l| {
            l.split_whitespace()
                .nth(9)
                .and_then(|s| s.parse::<u64>().ok())
        });

    // Parse pending sectors
    let pending_sectors = stdout
        .lines()
        .find(|l| l.contains("Current_Pending_Sector"))
        .and_then(|l| {
            l.split_whitespace()
                .nth(9)
                .and_then(|s| s.parse::<u64>().ok())
        });

    let has_errors = health_status == "FAILING"
        || reallocated_sectors.unwrap_or(0) > 0
        || pending_sectors.unwrap_or(0) > 0;

    Some(DiskHealthInfo {
        device: device.to_string(),
        health_status,
        temperature_celsius,
        power_on_hours,
        wear_leveling,
        reallocated_sectors,
        pending_sectors,
        has_errors,
    })
}

/// Collect system health metrics from systemd journal
fn collect_system_health_metrics() -> anna_common::SystemHealthMetrics {
    use anna_common::SystemHealthMetrics;

    // Count journal errors and warnings in last 24h
    let (errors_24h, warnings_24h) = count_journal_issues();

    // Get critical events
    let critical_events = get_critical_events();

    // Get degraded services
    let degraded_services = get_degraded_services();

    // Get recent crashes
    let recent_crashes = get_recent_service_crashes();

    // Count OOM events
    let oom_events = count_oom_events();

    // Get kernel errors
    let kernel_errors = get_kernel_errors();

    SystemHealthMetrics {
        journal_errors_last_24h: errors_24h,
        journal_warnings_last_24h: warnings_24h,
        critical_events,
        degraded_services,
        recent_crashes,
        oom_events_last_week: oom_events,
        kernel_errors,
    }
}

/// Count journal errors and warnings in last 24 hours
fn count_journal_issues() -> (usize, usize) {
    let mut errors = 0;
    let mut warnings = 0;

    if let Ok(output) = Command::new("journalctl")
        .args(&["--since", "24 hours ago", "-p", "err", "--no-pager"])
        .output()
    {
        let error_lines = String::from_utf8_lossy(&output.stdout);

        // Filter out known false positives (TLP and other common benign errors)
        errors = error_lines
            .lines()
            .filter(|line| !is_whitelisted_error(line))
            .count();
    }

    if let Ok(output) = Command::new("journalctl")
        .args(&["--since", "24 hours ago", "-p", "warning", "--no-pager"])
        .output()
    {
        warnings = String::from_utf8_lossy(&output.stdout).lines().count();
    }

    (errors, warnings)
}

/// Check if an error message is whitelisted (known false positive)
fn is_whitelisted_error(line: &str) -> bool {
    let line_lower = line.to_lowercase();

    // TLP common false positives
    if line_lower.contains("tlp") {
        // TLP logs many things at error level that are informational
        if line_lower.contains("laptop-mode-tools detected")
            || line_lower.contains("deprecated")
            || line_lower.contains("not configured")
            || line_lower.contains("not available")
            || line_lower.contains("not installed")
            || line_lower.contains("could not enable")
        {
            return true;
        }
    }

    // GNOME Shell warnings often logged as errors
    if line_lower.contains("gjs") && line_lower.contains("warning") {
        return true;
    }

    // PulseAudio/Pipewire common benign errors
    if (line_lower.contains("pulseaudio") || line_lower.contains("pipewire"))
        && (line_lower.contains("deprecated") || line_lower.contains("suspend"))
    {
        return true;
    }

    false
}

/// Get critical system events from journal
fn get_critical_events() -> Vec<anna_common::CriticalEvent> {
    use anna_common::CriticalEvent;

    let mut events = Vec::new();

    if let Ok(output) = Command::new("journalctl")
        .args(&[
            "--since",
            "7 days ago",
            "-p",
            "crit",
            "--no-pager",
            "-o",
            "json",
        ])
        .output()
    {
        // Parse JSON output - simplified
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().take(10) {
            // Basic parsing - in production we'd use serde_json properly
            if line.contains("MESSAGE") {
                events.push(CriticalEvent {
                    timestamp: Utc::now(),
                    message: line.to_string(),
                    unit: None,
                    severity: "critical".to_string(),
                });
            }
        }
    }

    events
}

/// Get services in degraded state
fn get_degraded_services() -> Vec<String> {
    let mut degraded = Vec::new();

    if let Ok(output) = Command::new("systemctl")
        .args(&["list-units", "--state=degraded", "--no-pager"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().skip(1) {
            if let Some(service_name) = line.split_whitespace().next() {
                degraded.push(service_name.to_string());
            }
        }
    }

    degraded
}

/// Get recent service crashes
fn get_recent_service_crashes() -> Vec<anna_common::ServiceCrash> {
    use anna_common::ServiceCrash;

    let mut crashes = Vec::new();

    // Look for service failures in journal
    if let Ok(output) = Command::new("journalctl")
        .args(&[
            "--since",
            "7 days ago",
            "-u",
            "*.service",
            "-p",
            "err",
            "--no-pager",
        ])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().take(20) {
            if line.contains("Failed") || line.contains("crashed") {
                crashes.push(ServiceCrash {
                    service_name: "unknown".to_string(),
                    timestamp: Utc::now(),
                    exit_code: None,
                    signal: None,
                });
            }
        }
    }

    crashes
}

/// Count OOM (Out of Memory) events in last week
fn count_oom_events() -> usize {
    if let Ok(output) = Command::new("journalctl")
        .args(&["--since", "7 days ago", "-k", "--no-pager"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return stdout
            .lines()
            .filter(|l| l.contains("Out of memory"))
            .count();
    }
    0
}

/// Get recent kernel errors
fn get_kernel_errors() -> Vec<String> {
    let mut errors = Vec::new();

    if let Ok(output) = Command::new("journalctl")
        .args(&["-k", "-p", "err", "--since", "24 hours ago", "--no-pager"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().take(10) {
            errors.push(line.to_string());
        }
    }

    errors
}

/// Collect performance metrics
fn collect_performance_metrics(sys: &System) -> anna_common::PerformanceMetrics {
    use anna_common::{PerformanceMetrics, ProcessInfo};

    // Calculate CPU usage average
    let cpu_usage_avg = sys
        .cpus()
        .iter()
        .map(|cpu| cpu.cpu_usage() as f64)
        .sum::<f64>()
        / sys.cpus().len() as f64;

    // Calculate memory usage percentage
    let memory_usage_percent = (sys.used_memory() as f64 / sys.total_memory() as f64) * 100.0;

    // Get disk I/O stats (simplified - would need /proc/diskstats parsing)
    let (disk_read, disk_write) = get_disk_io_stats();

    // Get network stats (simplified)
    let (net_rx, net_tx) = get_network_stats();

    // Get network quality metrics (latency and packet loss)
    let (latency, packet_loss) = get_network_quality_stats();

    // Get top CPU processes
    let mut high_cpu_processes: Vec<ProcessInfo> = sys
        .processes()
        .values()
        .map(|p| ProcessInfo {
            name: p.name().to_string(),
            pid: p.pid().as_u32(),
            cpu_percent: p.cpu_usage() as f64,
            memory_mb: p.memory() as f64 / 1024.0 / 1024.0,
        })
        .collect();
    high_cpu_processes.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap());
    high_cpu_processes.truncate(5);

    // Get top memory processes
    let mut high_memory_processes: Vec<ProcessInfo> = sys
        .processes()
        .values()
        .map(|p| ProcessInfo {
            name: p.name().to_string(),
            pid: p.pid().as_u32(),
            cpu_percent: p.cpu_usage() as f64,
            memory_mb: p.memory() as f64 / 1024.0 / 1024.0,
        })
        .collect();
    high_memory_processes.sort_by(|a, b| b.memory_mb.partial_cmp(&a.memory_mb).unwrap());
    high_memory_processes.truncate(5);

    PerformanceMetrics {
        cpu_usage_avg_percent: cpu_usage_avg,
        memory_usage_avg_percent: memory_usage_percent,
        disk_io_read_mb_s: disk_read,
        disk_io_write_mb_s: disk_write,
        network_rx_mb_s: net_rx,
        network_tx_mb_s: net_tx,
        high_cpu_processes,
        high_memory_processes,
        average_latency_ms: latency,
        packet_loss_percent: packet_loss,
    }
}

/// Get disk I/O statistics from /proc/diskstats
fn get_disk_io_stats() -> (f64, f64) {
    use std::fs;

    // Read /proc/diskstats
    let diskstats = match fs::read_to_string("/proc/diskstats") {
        Ok(content) => content,
        Err(_) => return (0.0, 0.0),
    };

    let mut total_read_sectors = 0u64;
    let mut total_write_sectors = 0u64;

    for line in diskstats.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 14 {
            continue;
        }

        // Skip loop devices and partitions (we only want main disks)
        let device_name = fields[2];
        if device_name.starts_with("loop") || device_name.starts_with("ram") {
            continue;
        }

        // Skip partitions - only count whole disks (sda, nvme0n1, etc, not sda1, nvme0n1p1)
        if device_name
            .chars()
            .last()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
            && !device_name.contains("nvme")
        {
            continue;
        }
        if device_name.contains("nvme") && device_name.contains('p') {
            continue;
        }

        // Field 5: sectors read (512 bytes each)
        if let Ok(sectors_read) = fields[5].parse::<u64>() {
            total_read_sectors += sectors_read;
        }

        // Field 9: sectors written
        if let Ok(sectors_written) = fields[9].parse::<u64>() {
            total_write_sectors += sectors_written;
        }
    }

    // Convert sectors to MB (512 bytes per sector)
    // Note: These are cumulative values since boot, not rates
    // For rates, we'd need to track previous values and time delta
    let read_mb = (total_read_sectors as f64 * 512.0) / 1024.0 / 1024.0;
    let write_mb = (total_write_sectors as f64 * 512.0) / 1024.0 / 1024.0;

    // For now, return cumulative totals divided by uptime to get rough average
    // This gives MB/s average over system uptime
    if let Ok(uptime_str) = fs::read_to_string("/proc/uptime") {
        if let Some(uptime_secs) = uptime_str.split_whitespace().next() {
            if let Ok(uptime) = uptime_secs.parse::<f64>() {
                if uptime > 0.0 {
                    return (read_mb / uptime, write_mb / uptime);
                }
            }
        }
    }

    (0.0, 0.0)
}

/// Get network statistics from /proc/net/dev
fn get_network_stats() -> (f64, f64) {
    use std::fs;

    // Read /proc/net/dev
    let netdev = match fs::read_to_string("/proc/net/dev") {
        Ok(content) => content,
        Err(_) => return (0.0, 0.0),
    };

    let mut total_rx_bytes = 0u64;
    let mut total_tx_bytes = 0u64;

    for line in netdev.lines().skip(2) {
        // Skip header lines
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 {
            continue;
        }

        // Skip loopback interface
        if parts[0].starts_with("lo:") {
            continue;
        }

        // Bytes received (field 1 after interface name)
        if let Ok(rx_bytes) = parts[1].parse::<u64>() {
            total_rx_bytes += rx_bytes;
        }

        // Bytes transmitted (field 9 after interface name)
        if let Ok(tx_bytes) = parts[9].parse::<u64>() {
            total_tx_bytes += tx_bytes;
        }
    }

    // Convert to MB
    let rx_mb = total_rx_bytes as f64 / 1024.0 / 1024.0;
    let tx_mb = total_tx_bytes as f64 / 1024.0 / 1024.0;

    // Calculate MB/s average over uptime
    if let Ok(uptime_str) = fs::read_to_string("/proc/uptime") {
        if let Some(uptime_secs) = uptime_str.split_whitespace().next() {
            if let Ok(uptime) = uptime_secs.parse::<f64>() {
                if uptime > 0.0 {
                    return (rx_mb / uptime, tx_mb / uptime);
                }
            }
        }
    }

    (0.0, 0.0)
}

/// Get network quality metrics (latency and packet loss)
fn get_network_quality_stats() -> (Option<f64>, Option<f64>) {
    use std::process::Command;

    // Try to ping Google DNS (8.8.8.8) or fallback to gateway
    let ping_target = "8.8.8.8";
    let ping_count = "4"; // 4 pings for statistical relevance

    // Execute ping command with timeout
    let output = Command::new("ping")
        .args(&["-c", ping_count, "-W", "2", ping_target]) // 2 second timeout
        .output();

    if let Ok(result) = output {
        if result.status.success() {
            let stdout = String::from_utf8_lossy(&result.stdout);

            // Parse latency from ping output
            // Example line: "rtt min/avg/max/mdev = 12.345/15.678/20.123/2.456 ms"
            let latency = if let Some(rtt_line) =
                stdout.lines().find(|l| l.contains("rtt min/avg/max/mdev"))
            {
                // Extract average latency (second value)
                if let Some(values) = rtt_line.split('=').nth(1) {
                    let parts: Vec<&str> = values.trim().split('/').collect();
                    if parts.len() >= 2 {
                        // Parse the average (second value)
                        parts[1].trim().parse::<f64>().ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            // Parse packet loss from ping output
            // Example line: "4 packets transmitted, 4 received, 0% packet loss, time 3005ms"
            let packet_loss =
                if let Some(loss_line) = stdout.lines().find(|l| l.contains("packet loss")) {
                    // Extract packet loss percentage
                    if let Some(percent_part) = loss_line.split(',').nth(2) {
                        // Extract number before '%'
                        if let Some(loss_str) = percent_part.trim().split('%').next() {
                            loss_str.trim().parse::<f64>().ok()
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

            return (latency, packet_loss);
        }
    }

    // If ping fails, return None for both metrics
    (None, None)
}

/// Generate predictive insights based on collected data
fn generate_predictive_insights() -> anna_common::PredictiveInsights {
    use anna_common::{BootTimeTrend, PredictiveInsights, TemperatureTrend, TrendDirection};

    // Disk space prediction (simplified - would need historical data)
    let disk_prediction = predict_disk_usage();

    // Temperature trend (simplified)
    let temperature_trend = TemperatureTrend {
        cpu_trend: TrendDirection::Stable,
        is_concerning: false,
        average_temp_celsius: None,
        max_temp_celsius: None,
    };

    // Service reliability (simplified - would need historical data)
    let service_reliability = Vec::new();

    // Boot time trend (simplified)
    let boot_time_trend = BootTimeTrend {
        current_seconds: get_boot_time(),
        trend: TrendDirection::Stable,
        is_degrading: false,
    };

    // Memory pressure risk
    let memory_pressure_risk = assess_memory_pressure();

    PredictiveInsights {
        disk_full_prediction: disk_prediction,
        temperature_trend,
        service_reliability,
        boot_time_trend,
        memory_pressure_risk,
    }
}

/// Predict when disk will be full (simplified - needs historical data)
fn predict_disk_usage() -> Option<anna_common::DiskPrediction> {
    // Would need to track disk usage over time to make predictions
    // For now, just check if root is over 90% full
    if let Ok(output) = Command::new("df").args(&["-h", "/"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = stdout.lines().nth(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                if let Some(percent_str) = parts[4].strip_suffix('%') {
                    if let Ok(percent) = percent_str.parse::<u8>() {
                        if percent > 90 {
                            return Some(anna_common::DiskPrediction {
                                mount_point: "/".to_string(),
                                days_until_full: Some(30), // Placeholder
                                current_growth_gb_per_day: 0.1, // Placeholder
                            });
                        }
                    }
                }
            }
        }
    }
    None
}

/// Assess memory pressure risk
fn assess_memory_pressure() -> anna_common::RiskLevel {
    use anna_common::RiskLevel;

    if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        let mut total_kb = 0;
        let mut available_kb = 0;

        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                total_kb = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
            } else if line.starts_with("MemAvailable:") {
                available_kb = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
            }
        }

        if total_kb > 0 {
            let available_percent = (available_kb as f64 / total_kb as f64) * 100.0;
            if available_percent < 10.0 {
                return RiskLevel::High;
            } else if available_percent < 25.0 {
                return RiskLevel::Medium;
            }
        }
    }

    RiskLevel::Low
}

/// Detect CPU microcode status (beta.43+)
fn detect_microcode_status(cpu_model: &str) -> MicrocodeStatus {
    let vendor = if cpu_model.contains("Intel") {
        "Intel"
    } else if cpu_model.contains("AMD") {
        "AMD"
    } else {
        "Unknown"
    };

    let package_name = if vendor == "Intel" {
        "intel-ucode"
    } else if vendor == "AMD" {
        "amd-ucode"
    } else {
        return MicrocodeStatus::default();
    };

    let microcode_installed = package_installed(package_name);

    // Get current microcode version
    let current_version = if let Ok(output) = Command::new("grep")
        .args(&["microcode", "/proc/cpuinfo"])
        .output()
    {
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .and_then(|line| line.split(':').nth(1))
            .map(|v| v.trim().to_string())
    } else {
        None
    };

    MicrocodeStatus {
        microcode_installed,
        vendor: vendor.to_string(),
        current_version,
        needs_update: !microcode_installed, // Simple heuristic
    }
}

/// Collect battery information for laptops (beta.43+)
fn collect_battery_info() -> Option<BatteryInfo> {
    // Check if /sys/class/power_supply/BAT* exists
    let battery_paths: Vec<_> = std::fs::read_dir("/sys/class/power_supply")
        .ok()?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("BAT"))
        .collect();

    if battery_paths.is_empty() {
        return None;
    }

    let battery_path = battery_paths.first()?.path();

    // Read battery status
    let read_value = |file: &str| -> Option<String> {
        std::fs::read_to_string(battery_path.join(file))
            .ok()
            .map(|s| s.trim().to_string())
    };

    let status = read_value("status").unwrap_or_else(|| "Unknown".to_string());
    let capacity_percent = read_value("capacity").and_then(|s| s.parse::<f64>().ok());

    // Calculate health from energy_full vs energy_full_design
    let health_percent = if let (Some(full), Some(design)) = (
        read_value("energy_full").and_then(|s| s.parse::<f64>().ok()),
        read_value("energy_full_design").and_then(|s| s.parse::<f64>().ok()),
    ) {
        Some((full / design) * 100.0)
    } else {
        None
    };

    Some(BatteryInfo {
        present: true,
        capacity_percent,
        health_percent,
        status,
        time_to_empty: None, // Would need to calculate from power_now
        time_to_full: None,
        cycle_count: read_value("cycle_count").and_then(|s| s.parse::<u32>().ok()),
    })
}

/// Detect installed backup systems (beta.43+)
fn detect_backup_systems() -> Vec<String> {
    let backup_tools = vec![
        "timeshift",
        "rsync",
        "borg",
        "restic",
        "duplicity",
        "rclone",
        "deja-dup",
        "backintime",
    ];

    backup_tools
        .into_iter()
        .filter(|&tool| command_exists(tool))
        .map(|s| s.to_string())
        .collect()
}

/// Detect Bluetooth status (beta.43+)
fn detect_bluetooth_status() -> BluetoothStatus {
    // Check if bluetooth hardware exists
    let available = Path::new("/sys/class/bluetooth").exists();

    // Check if bluetooth service is running
    let enabled = if let Ok(output) = Command::new("systemctl")
        .args(&["is-active", "bluetooth"])
        .output()
    {
        output.status.success()
    } else {
        false
    };

    // Get connected devices using bluetoothctl
    let connected_devices = if enabled {
        if let Ok(output) = Command::new("bluetoothctl")
            .arg("devices")
            .arg("Connected")
            .output()
        {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(|line| {
                    // Parse "Device XX:XX:XX:XX:XX:XX DeviceName"
                    line.split_whitespace()
                        .skip(2)
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    BluetoothStatus {
        available,
        enabled,
        connected_devices,
    }
}

/// Collect SSD-specific information (beta.43+)
fn collect_ssd_info() -> Vec<SSDInfo> {
    let mut ssd_info = Vec::new();

    // Get list of block devices
    if let Ok(output) = Command::new("lsblk")
        .args(&["-d", "-o", "NAME,ROTA"])
        .output()
    {
        let output_str = String::from_utf8_lossy(&output.stdout);

        for line in output_str.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[1] == "0" {
                // ROTA=0 means SSD
                let device = format!("/dev/{}", parts[0]);

                // Get model name
                let model =
                    if let Ok(output) = Command::new("smartctl").args(&["-i", &device]).output() {
                        String::from_utf8_lossy(&output.stdout)
                            .lines()
                            .find(|line| {
                                line.contains("Device Model:") || line.contains("Model Number:")
                            })
                            .and_then(|line| line.split(':').nth(1))
                            .map(|s| s.trim().to_string())
                            .unwrap_or_else(|| "Unknown".to_string())
                    } else {
                        "Unknown".to_string()
                    };

                // Check if TRIM is enabled
                let trim_enabled = check_trim_enabled(&device);

                ssd_info.push(SSDInfo {
                    device,
                    model,
                    trim_enabled,
                    wear_leveling_count: None, // Would need SMART data parsing
                    total_bytes_written: None,
                    health_percent: None,
                });
            }
        }
    }

    ssd_info
}

/// Check if TRIM is enabled for a device
fn check_trim_enabled(device: &str) -> bool {
    // Check fstab for discard option
    if let Ok(fstab) = std::fs::read_to_string("/etc/fstab") {
        for line in fstab.lines() {
            if line.contains(device) && line.contains("discard") {
                return true;
            }
        }
    }

    // Check if fstrim.timer is enabled
    if let Ok(output) = Command::new("systemctl")
        .args(&["is-enabled", "fstrim.timer"])
        .output()
    {
        return output.status.success();
    }

    false
}

/// Analyze swap configuration (beta.43+)
fn analyze_swap_configuration() -> SwapConfiguration {
    let mut config = SwapConfiguration::default();

    // Check if swap is enabled
    if let Ok(output) = Command::new("swapon").arg("--show").output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.lines().collect();

        if lines.len() > 1 {
            config.swap_enabled = true;

            // Parse swap type and size
            for line in lines.iter().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let swap_device = parts[0];
                    config.swap_type = if swap_device.contains("zram") {
                        "zram".to_string()
                    } else if swap_device.contains("/swapfile") {
                        "file".to_string()
                    } else {
                        "partition".to_string()
                    };

                    // Parse size (e.g., "2G", "512M")
                    if let Some(size_str) = parts.get(2) {
                        config.swap_size_gb = parse_size_to_gb(size_str);
                    }
                }
            }
        }
    }

    // Get swappiness value
    if let Ok(swappiness_str) = std::fs::read_to_string("/proc/sys/vm/swappiness") {
        config.swappiness = swappiness_str.trim().parse::<u32>().unwrap_or(60);
    }

    // Check if zram module is loaded
    config.zram_enabled = Path::new("/dev/zram0").exists();

    // Get swap usage
    if let Ok(output) = Command::new("free").arg("-b").output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        if let Some(swap_line) = output_str.lines().find(|line| line.starts_with("Swap:")) {
            let parts: Vec<&str> = swap_line.split_whitespace().collect();
            if parts.len() >= 3 {
                if let (Ok(total), Ok(used)) = (parts[1].parse::<f64>(), parts[2].parse::<f64>()) {
                    if total > 0.0 {
                        config.swap_usage_percent = (used / total) * 100.0;
                    }
                }
            }
        }
    }

    config
}

/// Collect locale and timezone information (beta.43+)
fn collect_locale_info() -> LocaleInfo {
    let timezone = if let Ok(output) = Command::new("timedatectl")
        .arg("show")
        .arg("-p")
        .arg("Timezone")
        .arg("--value")
        .output()
    {
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    } else {
        "Unknown".to_string()
    };

    let locale = std::env::var("LANG").unwrap_or_else(|_| {
        if let Ok(output) = Command::new("locale").output() {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .find(|line| line.starts_with("LANG="))
                .and_then(|line| line.split('=').nth(1))
                .unwrap_or("en_US.UTF-8")
                .to_string()
        } else {
            "en_US.UTF-8".to_string()
        }
    });

    let keymap = if let Ok(output) = Command::new("localectl").arg("status").output() {
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .find(|line| line.contains("X11 Layout:"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "us".to_string())
    } else {
        "us".to_string()
    };

    let language = locale.split('_').next().unwrap_or("en").to_string();

    LocaleInfo {
        timezone,
        locale,
        keymap,
        language,
    }
}

/// Detect installed pacman hooks (beta.43+)
fn detect_pacman_hooks() -> Vec<String> {
    let hook_dirs = vec!["/etc/pacman.d/hooks", "/usr/share/libalpm/hooks"];

    let mut hooks = Vec::new();

    for dir in hook_dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Some(filename) = entry.file_name().to_str() {
                    if filename.ends_with(".hook") {
                        hooks.push(filename.trim_end_matches(".hook").to_string());
                    }
                }
            }
        }
    }

    hooks
}

/// Detect audio system (beta.43+)
fn detect_audio_system() -> Option<String> {
    // Check for PipeWire (modern audio server)
    if Command::new("systemctl")
        .args(&["--user", "is-active", "pipewire.service"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some("PipeWire".to_string());
    }

    // Check for PipeWire using ps (fallback)
    if Command::new("pgrep")
        .arg("pipewire")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some("PipeWire".to_string());
    }

    // Check for PulseAudio (traditional audio server)
    if Command::new("systemctl")
        .args(&["--user", "is-active", "pulseaudio.service"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some("PulseAudio".to_string());
    }

    // Check for PulseAudio using pulseaudio command
    if Command::new("pgrep")
        .arg("pulseaudio")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some("PulseAudio".to_string());
    }

    // Fallback: check if packages are installed
    if package_installed("pipewire") {
        return Some("PipeWire (not running)".to_string());
    }

    if package_installed("pulseaudio") {
        return Some("PulseAudio (not running)".to_string());
    }

    // Default to ALSA if nothing else detected
    Some("ALSA".to_string())
}

/// Check if audio server is currently running (beta.43+)
fn check_audio_server_running() -> bool {
    // Check PipeWire
    if Command::new("systemctl")
        .args(&["--user", "is-active", "pipewire.service"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return true;
    }

    // Check PulseAudio
    if Command::new("systemctl")
        .args(&["--user", "is-active", "pulseaudio.service"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return true;
    }

    // Check using pgrep as fallback
    Command::new("pgrep")
        .arg("-x")
        .arg("pipewire|pulseaudio")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Detect PipeWire session manager (beta.43+)
fn detect_pipewire_session_manager() -> Option<String> {
    // Only relevant if PipeWire is the audio system
    if let Some(audio) = detect_audio_system() {
        if !audio.contains("PipeWire") {
            return None;
        }
    } else {
        return None;
    }

    // Check for WirePlumber (modern session manager)
    if Command::new("systemctl")
        .args(&["--user", "is-active", "wireplumber.service"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some("WirePlumber".to_string());
    }

    // Check using pgrep
    if Command::new("pgrep")
        .arg("wireplumber")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some("WirePlumber".to_string());
    }

    // Check for pipewire-media-session (legacy)
    if Command::new("systemctl")
        .args(&["--user", "is-active", "pipewire-media-session.service"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some("pipewire-media-session".to_string());
    }

    // Check using pgrep
    if Command::new("pgrep")
        .arg("pipewire-media-session")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some("pipewire-media-session".to_string());
    }

    // Check if packages are installed
    if package_installed("wireplumber") {
        return Some("WirePlumber (not running)".to_string());
    }

    if package_installed("pipewire-media-session") {
        return Some("pipewire-media-session (not running)".to_string());
    }

    None
}

/// Get GPU model name (beta.43+)
fn get_gpu_model() -> Option<String> {
    // Try lspci to get GPU model
    if let Ok(output) = Command::new("lspci").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let lower = line.to_lowercase();
            // Look for VGA, Display, or 3D controller lines
            if lower.contains("vga") || lower.contains("display") || lower.contains("3d controller")
            {
                // Extract GPU info after the device ID
                if let Some(gpu_info) = line.split(':').nth(2) {
                    let model = gpu_info.trim();
                    // Filter out generic entries
                    if !model.is_empty() && !model.contains("unknown") {
                        return Some(model.to_string());
                    }
                }
            }
        }
    }

    None
}

/// Get GPU VRAM size in MB (beta.43+)
fn get_gpu_vram_mb() -> Option<u32> {
    // For NVIDIA, use nvidia-smi
    if detect_nvidia() {
        if let Ok(output) = Command::new("nvidia-smi")
            .arg("--query-gpu=memory.total")
            .arg("--format=csv,noheader,nounits")
            .output()
        {
            if output.status.success() {
                let vram_str = String::from_utf8_lossy(&output.stdout);
                if let Ok(vram) = vram_str.trim().parse::<u32>() {
                    return Some(vram);
                }
            }
        }
    }

    // For AMD, try reading from sysfs
    if detect_amd_gpu() {
        // Try to find AMD GPU sysfs entry
        if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("card"))
                    .unwrap_or(false)
                {
                    let vram_path = path.join("device/mem_info_vram_total");
                    if let Ok(vram_str) = std::fs::read_to_string(&vram_path) {
                        if let Ok(vram_bytes) = vram_str.trim().parse::<u64>() {
                            return Some((vram_bytes / 1024 / 1024) as u32); // Convert bytes to MB
                        }
                    }
                }
            }
        }
    }

    // For Intel, integrated GPUs typically share system RAM (no dedicated VRAM)
    // Could estimate based on aperture size, but not reliable
    None
}

/// Check if Vulkan support is available (beta.43+)
fn check_vulkan_support() -> bool {
    // Check if vulkan-icd-loader is installed
    if package_installed("vulkan-icd-loader") {
        return true;
    }

    // Check for Vulkan library
    if Path::new("/usr/lib/libvulkan.so").exists() || Path::new("/usr/lib64/libvulkan.so").exists()
    {
        return true;
    }

    // Check if vulkaninfo command exists and works
    if let Ok(output) = Command::new("vulkaninfo").arg("--summary").output() {
        if output.status.success() {
            return true;
        }
    }

    false
}

/// Check if NVIDIA CUDA support is available (beta.43+)
fn check_nvidia_cuda_support() -> bool {
    // Only relevant for NVIDIA GPUs
    if !detect_nvidia() {
        return false;
    }

    // Check if CUDA toolkit is installed
    if package_installed("cuda") || package_installed("cuda-toolkit") {
        return true;
    }

    // Check for nvcc compiler
    if let Ok(output) = Command::new("which").arg("nvcc").output() {
        if output.status.success() {
            return true;
        }
    }

    // Check for CUDA library
    if Path::new("/opt/cuda").exists() || Path::new("/usr/local/cuda").exists() {
        return true;
    }

    false
}
