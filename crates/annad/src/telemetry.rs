//! System telemetry collection
//!
//! Gathers hardware, software, and system state information.

use anna_common::{CommandUsage, MediaUsageProfile, StorageDevice, SystemFacts, SystemdService};
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

/// Enhanced: Analyze process CPU time to understand user behavior
#[allow(dead_code)]
pub async fn analyze_process_cpu_time() -> Vec<ProcessUsage> {
    let mut process_usage = Vec::new();
    
    // Get list of processes sorted by CPU time
    if let Ok(output) = Command::new("ps")
        .args(&["aux", "--sort=-time"])
        .output()
    {
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
            "cargo" | "rustc" | "npm" | "yarn" | "python" | "python3" | "pip" | "go" | "gcc" | "make" => "development",
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
                evidence: format!("git used {} times", self.command_frequency.get("git").unwrap()),
            });
        }
        
        if self.command_frequency.get("docker").unwrap_or(&0) > &10 {
            self.workflow_patterns.push(WorkflowPattern {
                name: "Container Development".to_string(),
                confidence: 0.85,
                evidence: format!("docker used {} times", self.command_frequency.get("docker").unwrap()),
            });
        }
        
        let dev_tools = ["cargo", "npm", "python", "go", "gcc", "make"];
        let dev_count: usize = dev_tools.iter()
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
    analysis.boot_time = get_boot_time().map(|t| format!("{:.2}s", t)).unwrap_or_else(|| "Unknown".to_string());
    
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
    ||
    Command::new("systemctl")
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
            if device.starts_with("loop") || device.chars().last().map(|c| c.is_numeric()).unwrap_or(false) {
                continue;
            }
            
            let scheduler_path = entry.path().join("queue/scheduler");
            if let Ok(scheduler) = std::fs::read_to_string(scheduler_path) {
                // Extract current scheduler (marked with [brackets])
                if let Some(current) = scheduler.split_whitespace()
                    .find(|s| s.starts_with('[') && s.ends_with(']'))
                {
                    schedulers.insert(device, current.trim_matches(|c| c == '[' || c == ']').to_string());
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
            let key = Path::new(path).file_name().unwrap().to_string_lossy().to_string();
            params.insert(key, value.trim().to_string());
        }
    }
    
    params
}

/// Get boot time in seconds using systemd-analyze
fn get_boot_time() -> Option<f64> {
    let output = Command::new("systemd-analyze")
        .arg("time")
        .output()
        .ok()?;

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

    for line in text.lines().take(20) {  // Only check top 20 slowest
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
        .args(&["-Qm"])  // List foreign packages (not in sync database)
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
    let helpers = vec![
        "yay",
        "paru",
        "aurutils",
        "pikaur",
        "aura",
        "trizen",
    ];

    for helper in helpers {
        if Command::new("which").arg(helper).output().ok()?.status.success() {
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
                    return bytes as f64 / 1024.0 / 1024.0 / 1024.0;  // Convert to GB
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
