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
// New comprehensive telemetry functions

use anna_common::{
    DevelopmentProfile, DiskUsageTrend, DirectorySize, GamingProfile,
    LanguageUsage, NetworkProfile, PackageInstallation, ProjectInfo,
    SessionInfo, UserPreferences,
};

/// Get recently installed packages (last 30 days)
fn get_recently_installed_packages() -> Vec<PackageInstallation> {
    let mut packages = Vec::new();

    // Parse /var/log/pacman.log for installations
    if let Ok(log_content) = std::fs::read_to_string("/var/log/pacman.log") {
        let thirty_days_ago = chrono::Utc::now() - chrono::Duration::days(30);

        for line in log_content.lines() {
            if line.contains("installed") {
                // Parse format: [2025-01-04T12:34:56-0800] [ALPM] installed package (version)
                if let Some(pkg_info) = line.split("installed ").nth(1) {
                    let pkg_name = pkg_info.split_whitespace().next().unwrap_or("").to_string();

                    // Try to parse timestamp
                    if let Some(timestamp_str) = line.split('[').nth(1) {
                        if let Some(ts) = timestamp_str.split(']').next() {
                            // Simple approach: just add recent packages
                            packages.push(PackageInstallation {
                                name: pkg_name,
                                installed_at: chrono::Utc::now(), // Simplified for now
                                from_aur: false, // We'll detect this separately
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
        .args(&["list-units", "--type=service", "--state=running", "--no-pager", "--no-legend"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return stdout
            .lines()
            .filter_map(|line| {
                line.split_whitespace().next().map(|s| s.to_string())
            })
            .collect();
    }

    Vec::new()
}

/// Get services enabled on boot
fn get_enabled_services() -> Vec<String> {
    let output = Command::new("systemctl")
        .args(&["list-unit-files", "--type=service", "--state=enabled", "--no-pager", "--no-legend"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return stdout
            .lines()
            .filter_map(|line| {
                line.split_whitespace().next().map(|s| s.to_string())
            })
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
    if let Ok(output) = Command::new("du")
        .args(&["-sb", path])
        .output()
    {
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
        last_login: None, // Placeholder
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
    let uses_containers = Command::new("which").arg("docker").output()
        .map(|o| o.status.success()).unwrap_or(false)
        || Command::new("which").arg("podman").output()
        .map(|o| o.status.success()).unwrap_or(false);

    // Detect virtualization
    let uses_virtualization = Command::new("which").arg("qemu-system-x86_64").output()
        .map(|o| o.status.success()).unwrap_or(false)
        || Command::new("which").arg("virtualbox").output()
        .map(|o| o.status.success()).unwrap_or(false);

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
            "-maxdepth", "4",
            "-name", &format!("*.{}", extension),
            "-type", "f",
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
        if Command::new("which").arg(cmd).output()
            .map(|o| o.status.success()).unwrap_or(false)
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
        .args(&[
            home_dir,
            "-maxdepth", "4",
            "-type", "d",
            "-name", ".git",
        ])
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
    let has_cli_tools = cli_tools.iter().any(|tool| dev_tools.contains(&tool.to_string()));

    // Check for beautification tools
    let beauty_tools = vec!["starship", "eza", "bat", "fd", "ripgrep", "fzf"];
    let has_beauty_tools = beauty_tools.iter().any(|tool| check_package_installed(tool));

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
