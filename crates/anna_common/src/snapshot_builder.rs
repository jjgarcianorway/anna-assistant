//! Snapshot Builder v7.41.0 - Daemon-Side Snapshot Generation
//!
//! This module is used ONLY by annad to build snapshots.
//! annactl must never import or use this module.
//!
//! The builder:
//! 1. Checks delta fingerprints to decide what needs rebuilding
//! 2. Runs heavyweight scans only when necessary
//! 3. Atomically publishes snapshots for annactl to read

use chrono::Utc;
use std::process::Command;

use crate::grounded::steam::{detect_steam_games, is_steam_installed};
use crate::snapshots::*;

// =============================================================================
// SOFTWARE SNAPSHOT BUILDER
// =============================================================================

/// Build a complete software snapshot
/// This is the expensive operation - called by daemon only
pub fn build_sw_snapshot() -> SwSnapshot {
    let start = std::time::Instant::now();

    // Build all components
    let packages = build_package_snapshot();
    let commands = build_command_snapshot();
    let services = build_service_snapshot();
    let categories = build_categories();
    let config_coverage = build_config_coverage();
    let topology = build_topology();
    let platforms = build_platform_snapshot();

    let scan_duration_ms = start.elapsed().as_millis() as u64;

    SwSnapshot {
        data_version: SW_SNAPSHOT_VERSION,
        generated_at: Utc::now(),
        scan_duration_ms,
        packages,
        commands,
        services,
        categories,
        config_coverage,
        topology,
        platforms,
    }
}

fn build_package_snapshot() -> PackageSnapshot {
    let output = Command::new("pacman").args(["-Q"]).output();

    let mut total = 0;
    let mut explicit = 0;
    let mut aur = 0;

    if let Ok(out) = output {
        if out.status.success() {
            total = String::from_utf8_lossy(&out.stdout).lines().count();
        }
    }

    // Count explicit packages
    if let Ok(out) = Command::new("pacman").args(["-Qe"]).output() {
        if out.status.success() {
            explicit = String::from_utf8_lossy(&out.stdout).lines().count();
        }
    }

    // Count AUR packages
    if let Ok(out) = Command::new("pacman").args(["-Qm"]).output() {
        if out.status.success() {
            aur = String::from_utf8_lossy(&out.stdout).lines().count();
        }
    }

    let dependency = total.saturating_sub(explicit);

    PackageSnapshot {
        total,
        explicit,
        dependency,
        aur,
        highlights: Vec::new(),
    }
}

fn build_command_snapshot() -> CommandSnapshot {
    let path_var = std::env::var("PATH").unwrap_or_default();
    let mut total = 0;

    for dir in path_var.split(':') {
        if dir.is_empty() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(ft) = entry.file_type() {
                    if ft.is_file() || ft.is_symlink() {
                        // Check if executable
                        if let Ok(meta) = entry.metadata() {
                            use std::os::unix::fs::PermissionsExt;
                            if meta.permissions().mode() & 0o111 != 0 {
                                total += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    CommandSnapshot {
        total,
        highlights: Vec::new(),
    }
}

fn build_service_snapshot() -> ServiceSnapshot {
    let mut total = 0;
    let mut running = 0;
    let mut failed = 0;
    let mut enabled = 0;
    let mut failed_services = Vec::new();

    // Count services from systemctl
    if let Ok(out) = Command::new("systemctl")
        .args(["list-units", "--type=service", "--no-pager", "--no-legend"])
        .output()
    {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                total += 1;
                if line.contains("running") {
                    running += 1;
                }
                if line.contains("failed") {
                    failed += 1;
                    // Extract service name
                    if let Some(name) = line.split_whitespace().next() {
                        failed_services.push(name.to_string());
                    }
                }
            }
        }
    }

    // Count enabled services
    if let Ok(out) = Command::new("systemctl")
        .args([
            "list-unit-files",
            "--type=service",
            "--state=enabled",
            "--no-pager",
            "--no-legend",
        ])
        .output()
    {
        if out.status.success() {
            enabled = String::from_utf8_lossy(&out.stdout).lines().count();
        }
    }

    ServiceSnapshot {
        total,
        running,
        failed,
        enabled,
        failed_services,
    }
}

/// Build categories - this is the expensive part
/// We optimize by batching pacman calls
fn build_categories() -> Vec<CategoryEntry> {
    use std::collections::HashMap as StdHashMap;

    // Get all explicit packages with their descriptions in ONE pacman call
    let output = Command::new("pacman").args(["-Qi"]).output();

    let Ok(out) = output else {
        return Vec::new();
    };

    if !out.status.success() {
        return Vec::new();
    }

    // Parse all package info at once
    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut packages_info: Vec<(String, String)> = Vec::new();
    let mut current_name = String::new();
    let mut current_desc = String::new();
    let mut is_explicit = false;

    for line in stdout.lines() {
        if line.starts_with("Name") {
            // Save previous package if explicit
            if !current_name.is_empty() && is_explicit {
                packages_info.push((current_name.clone(), current_desc.clone()));
            }
            if let Some(name) = line.split(':').nth(1) {
                current_name = name.trim().to_string();
            }
            current_desc.clear();
            is_explicit = false;
        } else if line.starts_with("Description") {
            if let Some(desc) = line.split(':').nth(1) {
                current_desc = desc.trim().to_string();
            }
        } else if line.starts_with("Install Reason") {
            if let Some(reason) = line.split(':').nth(1) {
                is_explicit = reason.trim().contains("Explicitly");
            }
        }
    }

    // Don't forget the last package
    if !current_name.is_empty() && is_explicit {
        packages_info.push((current_name, current_desc));
    }

    // Categorize based on description
    let mut category_packages: StdHashMap<String, Vec<String>> = StdHashMap::new();

    for (name, desc) in packages_info {
        let cat = categorize_by_description(&name, &desc);
        category_packages.entry(cat).or_default().push(name);
    }

    // Sort packages in each category
    for packages in category_packages.values_mut() {
        packages.sort_by_key(|n| n.to_lowercase());
    }

    // Build result in standard order
    let order = [
        "Editors",
        "Terminals",
        "Shells",
        "Compositors",
        "Browsers",
        "Multimedia",
        "Development",
        "Network",
        "System",
        "Power",
        "Virtualization",
        "Containers",
        "Games",
        "Tools",
        "Other",
    ];

    let mut result = Vec::new();
    for cat_name in order {
        if let Some(packages) = category_packages.remove(cat_name) {
            if !packages.is_empty() {
                result.push(CategoryEntry {
                    name: cat_name.to_string(),
                    count: packages.len(),
                    packages,
                });
            }
        }
    }

    result
}

/// Fast categorization based on description text
fn categorize_by_description(name: &str, desc: &str) -> String {
    let desc_lower = desc.to_lowercase();
    let name_lower = name.to_lowercase();

    // Editors
    if desc_lower.contains("text editor")
        || desc_lower.contains("editor for")
        || desc_lower.contains("vi improved")
        || name_lower == "vim"
        || name_lower == "nano"
        || name_lower == "emacs"
        || name_lower == "helix"
        || name_lower == "neovim"
        || name_lower == "nvim"
    {
        return "Editors".to_string();
    }

    // Terminals
    if desc_lower.contains("terminal emulator")
        || desc_lower.contains("terminal application")
        || name_lower == "alacritty"
        || name_lower == "kitty"
        || name_lower == "foot"
        || name_lower == "wezterm"
    {
        return "Terminals".to_string();
    }

    // Shells
    if desc_lower.contains("command interpreter")
        || desc_lower.contains("unix shell")
        || name_lower == "bash"
        || name_lower == "zsh"
        || name_lower == "fish"
    {
        return "Shells".to_string();
    }

    // Compositors
    if desc_lower.contains("wayland compositor")
        || desc_lower.contains("window manager")
        || name_lower == "hyprland"
        || name_lower == "sway"
        || name_lower == "i3"
    {
        return "Compositors".to_string();
    }

    // Browsers
    if desc_lower.contains("web browser")
        || desc_lower.contains("browser")
        || name_lower.contains("firefox")
        || name_lower.contains("chrome")
        || name_lower.contains("chromium")
    {
        return "Browsers".to_string();
    }

    // Multimedia
    if desc_lower.contains("media player")
        || desc_lower.contains("video player")
        || desc_lower.contains("audio")
        || desc_lower.contains("music")
        || name_lower == "mpv"
        || name_lower == "vlc"
    {
        return "Multimedia".to_string();
    }

    // Development
    if desc_lower.contains("compiler")
        || desc_lower.contains("programming")
        || desc_lower.contains("development")
        || desc_lower.contains("debugger")
        || desc_lower.contains("sdk")
        || name_lower.contains("git")
        || name_lower.contains("rust")
        || name_lower.contains("python")
    {
        return "Development".to_string();
    }

    // Network
    if desc_lower.contains("network")
        || desc_lower.contains("ssh")
        || desc_lower.contains("wireless")
        || desc_lower.contains("wifi")
    {
        return "Network".to_string();
    }

    // System
    if desc_lower.contains("system")
        || desc_lower.contains("boot")
        || desc_lower.contains("firmware")
        || desc_lower.contains("kernel")
    {
        return "System".to_string();
    }

    // Power
    if desc_lower.contains("power")
        || desc_lower.contains("battery")
        || desc_lower.contains("energy")
        || name_lower == "tlp"
    {
        return "Power".to_string();
    }

    // Virtualization
    if desc_lower.contains("virtual")
        || desc_lower.contains("emulator")
        || name_lower.contains("qemu")
        || name_lower.contains("virt")
    {
        return "Virtualization".to_string();
    }

    // Containers
    if desc_lower.contains("container")
        || name_lower.contains("docker")
        || name_lower.contains("podman")
    {
        return "Containers".to_string();
    }

    // Games
    if desc_lower.contains("game") || name_lower == "steam" || name_lower == "discord" {
        return "Games".to_string();
    }

    // Default to Tools for utilities
    if desc_lower.contains("tool") || desc_lower.contains("utility") || desc_lower.contains("cli") {
        return "Tools".to_string();
    }

    "Tools".to_string()
}

fn build_config_coverage() -> ConfigCoverage {
    use crate::config_locator::discover_config;

    const APPS: &[&str] = &[
        "vim",
        "nvim",
        "emacs",
        "foot",
        "hyprland",
        "sway",
        "mpv",
        "pipewire",
        "networkmanager",
        "wpa_supplicant",
        "tlp",
        "systemd",
        "grub",
        "mkinitcpio",
        "pacman",
        "yay",
    ];

    let mut app_names = Vec::new();

    for &app in APPS {
        let discovery = discover_config(app);
        if !discovery.detected.is_empty() {
            app_names.push(app.to_string());
        }
    }

    ConfigCoverage {
        total_apps: APPS.len(),
        apps_with_config: app_names.len(),
        app_names,
    }
}

fn build_topology() -> TopologySnapshot {
    use crate::topology_map::build_software_topology;

    let topo = build_software_topology();

    TopologySnapshot {
        roles: topo
            .roles
            .iter()
            .map(|r| RoleEntry {
                name: r.name.clone(),
                components: r.components.clone(),
            })
            .collect(),
        service_groups: topo
            .service_groups
            .iter()
            .map(|g| ServiceGroupEntry {
                name: g.name.clone(),
                services: g.services.clone(),
            })
            .collect(),
    }
}

fn build_platform_snapshot() -> PlatformSnapshot {
    if !is_steam_installed() {
        return PlatformSnapshot::default();
    }

    let games = detect_steam_games();
    let total_size: u64 = games.iter().filter_map(|g| g.size_on_disk).sum();

    // Get top 5 games by size
    let mut sorted_games = games.clone();
    sorted_games.sort_by(|a, b| b.size_on_disk.cmp(&a.size_on_disk));

    let top_games: Vec<GameEntry> = sorted_games
        .iter()
        .take(5)
        .map(|g| GameEntry {
            name: g.name.clone(),
            size_bytes: g.size_on_disk.unwrap_or(0),
        })
        .collect();

    PlatformSnapshot {
        steam_installed: true,
        steam_game_count: games.len(),
        steam_total_size_bytes: total_size,
        steam_top_games: top_games,
    }
}

// =============================================================================
// HARDWARE SNAPSHOT BUILDER
// =============================================================================

/// Build hardware snapshot
pub fn build_hw_snapshot() -> HwSnapshot {
    let start = std::time::Instant::now();

    let cpu = build_cpu_snapshot();
    let memory = build_memory_snapshot();
    let storage = build_storage_snapshot();
    let network = build_network_snapshot();
    let gpu = build_gpu_snapshot();

    let scan_duration_ms = start.elapsed().as_millis() as u64;

    HwSnapshot {
        data_version: HW_SNAPSHOT_VERSION,
        generated_at: Utc::now(),
        scan_duration_ms,
        cpu,
        memory,
        storage,
        network,
        gpu,
    }
}

fn build_cpu_snapshot() -> CpuSnapshot {
    let mut model = String::new();
    let mut cores = 0;
    let mut threads = 0;

    if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
        for line in content.lines() {
            if line.starts_with("model name") {
                if let Some(val) = line.split(':').nth(1) {
                    if model.is_empty() {
                        model = val.trim().to_string();
                    }
                }
            }
            if line.starts_with("processor") {
                threads += 1;
            }
            if line.starts_with("cpu cores") {
                if let Some(val) = line.split(':').nth(1) {
                    if let Ok(c) = val.trim().parse() {
                        cores = c;
                    }
                }
            }
        }
    }

    CpuSnapshot {
        model,
        cores,
        threads,
        frequency_mhz: None,
    }
}

fn build_memory_snapshot() -> MemorySnapshot {
    let mut total = 0u64;
    let mut available = 0u64;
    let mut swap_total = 0u64;

    if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(kb) = parse_meminfo_kb(line) {
                    total = kb * 1024;
                }
            } else if line.starts_with("MemAvailable:") {
                if let Some(kb) = parse_meminfo_kb(line) {
                    available = kb * 1024;
                }
            } else if line.starts_with("SwapTotal:") {
                if let Some(kb) = parse_meminfo_kb(line) {
                    swap_total = kb * 1024;
                }
            }
        }
    }

    MemorySnapshot {
        total_bytes: total,
        available_bytes: available,
        swap_total_bytes: swap_total,
    }
}

fn parse_meminfo_kb(line: &str) -> Option<u64> {
    line.split_whitespace().nth(1).and_then(|s| s.parse().ok())
}

fn build_storage_snapshot() -> Vec<StorageEntry> {
    let mut entries = Vec::new();

    // Use lsblk for block devices
    if let Ok(out) = Command::new("lsblk")
        .args(["-b", "-o", "NAME,SIZE,MOUNTPOINT", "-n", "-d"])
        .output()
    {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = parts[0].to_string();
                    let size: u64 = parts[1].parse().unwrap_or(0);
                    let mount = parts.get(2).map(|s| s.to_string());

                    entries.push(StorageEntry {
                        name,
                        model: None,
                        size_bytes: size,
                        mount_point: mount,
                    });
                }
            }
        }
    }

    entries
}

fn build_network_snapshot() -> Vec<NetworkEntry> {
    let mut entries = Vec::new();

    if let Ok(read_dir) = std::fs::read_dir("/sys/class/net") {
        for entry in read_dir.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "lo" {
                continue; // Skip loopback
            }

            let is_wireless = entry.path().join("wireless").exists();

            // Check if up
            let operstate =
                std::fs::read_to_string(entry.path().join("operstate")).unwrap_or_default();
            let is_up = operstate.trim() == "up";

            // Get MAC
            let mac = std::fs::read_to_string(entry.path().join("address"))
                .ok()
                .map(|s| s.trim().to_string());

            entries.push(NetworkEntry {
                name,
                mac,
                ipv4: None,
                is_up,
                is_wireless,
            });
        }
    }

    entries
}

fn build_gpu_snapshot() -> Option<GpuSnapshot> {
    // Try lspci for GPU info
    if let Ok(out) = Command::new("lspci").args(["-mm"]).output() {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.contains("VGA") || line.contains("3D") {
                    // Parse lspci -mm format
                    let parts: Vec<&str> = line.split('"').collect();
                    if parts.len() >= 6 {
                        return Some(GpuSnapshot {
                            name: format!("{} {}", parts[3], parts[5]),
                            driver: None,
                            vram_mb: None,
                        });
                    }
                }
            }
        }
    }
    None
}

// =============================================================================
// DELTA-AWARE REBUILD
// =============================================================================

/// Check if software snapshot needs rebuild based on fingerprints
pub fn sw_needs_rebuild(meta: &SwMeta) -> bool {
    // Check pacman.log
    let current_pacman = get_pacman_log_fingerprint();
    if pacman_log_has_changes(&meta.pacman_log, &current_pacman) {
        return true;
    }

    // Check systemd
    let current_systemd = get_systemd_fingerprint();
    if current_systemd.unit_files_hash != meta.systemd.unit_files_hash
        || current_systemd.etc_mtime != meta.systemd.etc_mtime
        || current_systemd.usr_lib_mtime != meta.systemd.usr_lib_mtime
    {
        return true;
    }

    // Check PATH directories
    let path_var = std::env::var("PATH").unwrap_or_default();
    for dir in path_var.split(':') {
        if dir.is_empty() {
            continue;
        }

        if let Some(current_fp) = get_dir_fingerprint(dir) {
            match meta.path_dirs.get(dir) {
                Some(old_fp) => {
                    if old_fp.mtime != current_fp.mtime
                        || old_fp.file_count != current_fp.file_count
                        || old_fp.names_hash != current_fp.names_hash
                    {
                        return true;
                    }
                }
                None => return true, // New directory
            }
        }
    }

    false
}

/// Build and save software snapshot with meta update
pub fn build_and_save_sw_snapshot() -> std::io::Result<SwSnapshot> {
    let snapshot = build_sw_snapshot();
    snapshot.save()?;

    // Update meta
    let mut meta = SwMeta::default();
    meta.pacman_log = get_pacman_log_fingerprint();
    meta.systemd = get_systemd_fingerprint();
    meta.last_full_scan = Some(Utc::now());

    // Capture PATH fingerprints
    let path_var = std::env::var("PATH").unwrap_or_default();
    for dir in path_var.split(':') {
        if dir.is_empty() {
            continue;
        }
        if let Some(fp) = get_dir_fingerprint(dir) {
            meta.path_dirs.insert(dir.to_string(), fp);
        }
    }

    meta.save()?;

    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_by_description() {
        assert_eq!(categorize_by_description("vim", "Vi IMproved"), "Editors");
        assert_eq!(
            categorize_by_description("foot", "terminal emulator"),
            "Terminals"
        );
        assert_eq!(
            categorize_by_description("firefox", "web browser"),
            "Browsers"
        );
    }
}
