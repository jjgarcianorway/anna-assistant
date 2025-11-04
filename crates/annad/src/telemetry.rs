//! System telemetry collection
//!
//! Gathers hardware, software, and system state information.

use anna_common::{CommandUsage, MediaUsageProfile, StorageDevice, SystemFacts};
use anyhow::Result;
use chrono::Utc;
use sysinfo::System;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
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

        // User Environment
        shell: detect_shell(),
        desktop_environment: detect_desktop_environment(),
        display_server: detect_display_server(),

        // User Behavior (basic for now)
        frequently_used_commands: analyze_command_history().await,
        dev_tools_detected: detect_dev_tools(),
        media_usage: analyze_media_usage().await,
        common_file_types: detect_common_file_types().await,
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
    // Try lspci to detect GPU
    let output = Command::new("lspci")
        .output()
        .ok()?;
    let lspci_output = String::from_utf8_lossy(&output.stdout);

    if lspci_output.contains("NVIDIA") {
        Some("NVIDIA".to_string())
    } else if lspci_output.contains("AMD") || lspci_output.contains("Radeon") {
        Some("AMD".to_string())
    } else if lspci_output.contains("Intel") {
        Some("Intel".to_string())
    } else {
        None
    }
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
    let output = Command::new("pacman")
        .arg("-Q")
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().count())
}

fn find_orphan_packages() -> Result<Vec<String>> {
    // Find orphaned packages (installed as dependencies but no longer needed)
    let output = Command::new("pacman")
        .arg("-Qdtq")
        .output()?;

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
    let output = Command::new("ip")
        .args(&["link", "show"])
        .output();

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
        .and_then(|s| Path::new(&s).file_name().map(|f| f.to_string_lossy().to_string()))
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
        "git", "docker", "cargo", "python3", "node", "npm",
        "go", "java", "gcc", "vim", "nvim", "code",
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
