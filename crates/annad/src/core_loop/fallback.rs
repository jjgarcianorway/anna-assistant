//! Fallback command hints for when LLM is unavailable.
//! v0.0.932: Added profile-based command suggestions
//! v0.0.992: Integrated comprehensive monitoring system

use std::process::Command;
use tracing::{debug, info, warn};

use super::cache::{cache_command, get_cached_command};
use super::profile::get_system_profile;

/// v0.0.992: Run the comprehensive monitoring system and save issues
fn run_comprehensive_monitoring() {
    use anna_shared::monitor::{run_checks, IssueStore, MonitorThresholds};

    let thresholds = MonitorThresholds::default();
    let results = run_checks(&thresholds);

    // Save issues to store for display in REPL
    match IssueStore::load() {
        Ok(mut store) => {
            store.update(results);
            if let Err(e) = store.save() {
                warn!("Failed to save monitoring issues: {}", e);
            } else {
                let unnotified = store.get_unnotified().len();
                let critical = store.get_critical().len();
                if critical > 0 {
                    info!("Monitoring: {} critical issues detected", critical);
                } else if unnotified > 0 {
                    debug!("Monitoring: {} new issues detected", unnotified);
                }
            }
        }
        Err(e) => {
            warn!("Failed to load issue store: {}", e);
        }
    }
}

/// Heuristic command hints for when LLM is unavailable (timeout fallback)
pub fn get_fallback_commands(question: &str) -> Vec<&'static str> {
    get_fallback_commands_with_intent(question, None)
}

/// Get fallback commands with optional intent category for smarter suggestions
pub fn get_fallback_commands_with_intent(question: &str, intent: Option<&str>) -> Vec<&'static str> {
    let q = question.to_lowercase();

    // If we have intent, use category-specific commands
    if let Some(category) = intent {
        match category {
            "TROUBLESHOOT" => {
                if q.contains("network") {
                    return vec!["journalctl -u NetworkManager --no-pager -n 30", "ip addr", "systemctl status NetworkManager"];
                }
                if q.contains("audio") || q.contains("sound") {
                    return vec!["journalctl -u pipewire --no-pager -n 30", "pactl info", "wpctl status"];
                }
                if q.contains("boot") || q.contains("startup") {
                    return vec!["journalctl -b -p err --no-pager -n 30", "systemctl --failed"];
                }
                return vec!["journalctl -p err -b --no-pager | tail -30", "systemctl --failed", "dmesg --level=err | tail -20"];
            }
            "HOWTO" => {
                if q.contains("install") {
                    return vec!["pacman -Ss", "checkupdates | head -10"];
                }
                if q.contains("enable") || q.contains("service") {
                    return vec!["systemctl list-unit-files --type=service | head -20"];
                }
            }
            _ => {}
        }
    }

    // System info
    if q.contains("kernel") || q.contains("version") && q.contains("linux") {
        return vec!["uname -r", "uname -a"];
    }
    if q.contains("hostname") || q.contains("host name") {
        return vec!["hostname", "hostnamectl hostname"];
    }
    if q.contains("uptime") || q.contains("running") && q.contains("long") {
        return vec!["uptime -p", "uptime"];
    }
    if q.contains("distribution") || q.contains("distro") || q.contains("os") && !q.contains("process") {
        return vec!["cat /etc/os-release | head -5", "hostnamectl"];
    }
    if q.contains("architecture") || q.contains("arch") && q.contains("system") {
        return vec!["uname -m", "arch"];
    }
    if q.contains("shell") && (q.contains("using") || q.contains("am i") || q.contains("my")) {
        return vec!["echo $SHELL", "basename $SHELL"];
    }
    if q.contains("home") && q.contains("directory") {
        return vec!["echo $HOME", "pwd"];
    }
    if q.contains("current") && (q.contains("directory") || q.contains("folder") || q.contains("cwd")) {
        return vec!["pwd"];
    }
    if q.contains("username") || (q.contains("user") && q.contains("am i")) {
        return vec!["whoami", "id -un"];
    }
    if q.contains("timezone") || q.contains("time zone") {
        return vec!["timedatectl show -p Timezone --value", "cat /etc/timezone 2>/dev/null || timedatectl"];
    }
    if q.contains("locale") {
        return vec!["locale", "echo $LANG"];
    }
    if q.contains("display") && q.contains("server") || q.contains("wayland") || q.contains("xorg") {
        return vec!["echo $XDG_SESSION_TYPE", "loginctl show-session $(loginctl | grep $(whoami) | awk '{print $1}') -p Type --value 2>/dev/null || echo tty"];
    }
    if q.contains("desktop") && q.contains("environment") || q.contains(" de ") {
        return vec!["echo $XDG_CURRENT_DESKTOP", "echo $DESKTOP_SESSION"];
    }
    if q.contains("resolution") || q.contains("screen size") {
        return vec!["xrandr 2>/dev/null | grep '*' | head -1 | awk '{print $1}'", "wlr-randr 2>/dev/null | grep current | head -1"];
    }
    if q.contains("last") && q.contains("boot") {
        return vec!["who -b", "uptime -s"];
    }

    // Hardware
    if q.contains("cpu") || q.contains("processor") {
        return vec!["lscpu | head -20", "cat /proc/cpuinfo | head -30"];
    }
    if q.contains("memory") || q.contains("ram") {
        return vec!["free -h", "cat /proc/meminfo | head -10"];
    }
    if q.contains("gpu") || q.contains("graphics") || q.contains("video") {
        return vec!["lspci | grep -i vga", "lspci | grep -i 3d"];
    }
    if q.contains("disk") || q.contains("storage") || q.contains("space") {
        return vec!["df -h", "lsblk"];
    }
    if q.contains("usb") {
        return vec!["lsusb"];
    }
    if q.contains("network") || q.contains("interface") {
        return vec!["ip addr", "ip link"];
    }
    if q.contains("ip") && q.contains("address") {
        return vec!["ip addr show | grep 'inet '"];
    }

    // Packages
    if q.contains("installed") && q.contains("package") {
        return vec!["pacman -Q | wc -l"];
    }
    if q.contains("update") && (q.contains("package") || q.contains("system")) {
        return vec!["checkupdates | head -20"];
    }
    if q.contains("orphan") {
        return vec!["pacman -Qdt"];
    }

    // Package version checks
    let pkg_patterns = [
        ("git", vec!["which git && git --version", "pacman -Q git 2>/dev/null"]),
        ("neovim", vec!["which nvim && nvim --version | head -1", "pacman -Q neovim 2>/dev/null"]),
        ("nvim", vec!["which nvim && nvim --version | head -1", "pacman -Q neovim 2>/dev/null"]),
        ("docker", vec!["which docker && docker --version", "pacman -Q docker 2>/dev/null"]),
        ("python", vec!["which python && python --version", "pacman -Q python 2>/dev/null"]),
        ("node", vec!["which node && node --version", "pacman -Q nodejs 2>/dev/null"]),
        ("rust", vec!["which rustc && rustc --version", "pacman -Q rust 2>/dev/null"]),
    ];
    for (pkg, cmds) in pkg_patterns {
        if q.contains(pkg) && (q.contains("installed") || q.contains("version") || q.contains("have")) {
            return cmds;
        }
    }

    // Services
    if q.contains("service") && q.contains("fail") {
        return vec!["systemctl --failed"];
    }
    if q.contains("service") && q.contains("running") {
        return vec!["systemctl list-units --type=service --state=running | head -20"];
    }
    if q.contains("service") && q.contains("enabled") {
        return vec!["systemctl list-unit-files --state=enabled | head -20"];
    }

    // Troubleshooting
    if q.contains("error") || q.contains("log") {
        return vec!["journalctl -p err -b --no-pager | tail -30"];
    }
    if q.contains("process") && (q.contains("cpu") || q.contains("top")) {
        return vec!["ps aux --sort=-%cpu | head -10"];
    }
    if q.contains("process") && q.contains("memory") {
        return vec!["ps aux --sort=-%mem | head -10"];
    }

    vec![]
}

/// v0.0.932: Get profile-based command suggestions
/// Returns commands tailored to the detected system configuration
pub fn get_profile_based_commands(question: &str) -> Vec<String> {
    let q = question.to_lowercase();
    let profile = get_system_profile();
    let mut commands = Vec::new();

    // GPU-specific commands
    let has_nvidia = profile.hardware.pci_devices.iter().any(|d| {
        d.vendor.to_lowercase().contains("nvidia")
    });
    let has_amd_gpu = profile.hardware.pci_devices.iter().any(|d| {
        let v = d.vendor.to_lowercase();
        let c = d.class.to_lowercase();
        (v.contains("amd") || v.contains("advanced micro")) &&
        (c.contains("vga") || c.contains("display") || c.contains("3d"))
    });

    if q.contains("gpu") || q.contains("graphics") || q.contains("video") {
        if has_nvidia {
            commands.push("nvidia-smi".to_string());
            commands.push("nvidia-smi -q | head -50".to_string());
        }
        if has_amd_gpu {
            commands.push("radeontop -d - -l 1 2>/dev/null || echo 'radeontop not installed'".to_string());
            commands.push("cat /sys/class/drm/card*/device/gpu_busy_percent 2>/dev/null".to_string());
        }
    }

    // Filesystem-specific commands
    let fs = profile.system.root_filesystem.as_deref().unwrap_or("");
    if q.contains("disk") || q.contains("storage") || q.contains("filesystem") || q.contains("snapshot") {
        match fs {
            "btrfs" => {
                commands.push("btrfs filesystem df /".to_string());
                commands.push("btrfs subvolume list / 2>/dev/null | head -10".to_string());
                if q.contains("snapshot") {
                    commands.push("btrfs subvolume list -s / 2>/dev/null".to_string());
                }
            }
            "zfs" => {
                commands.push("zpool status".to_string());
                commands.push("zfs list".to_string());
            }
            "xfs" => {
                commands.push("xfs_info /".to_string());
            }
            _ => {}
        }
    }

    // Audio system-specific commands
    let audio = profile.system.audio_system.as_deref().unwrap_or("");
    if q.contains("audio") || q.contains("sound") || q.contains("volume") {
        match audio {
            "pipewire" => {
                commands.push("wpctl status".to_string());
                commands.push("pw-top -b -n 1 2>/dev/null | head -20".to_string());
            }
            "pulseaudio" => {
                commands.push("pactl info".to_string());
                commands.push("pactl list sinks short".to_string());
            }
            _ => {
                commands.push("aplay -l".to_string());
            }
        }
    }

    // Desktop environment-specific
    let de = profile.system.desktop.as_deref().unwrap_or("").to_lowercase();
    if q.contains("settings") || q.contains("theme") || q.contains("extension") {
        if de.contains("gnome") {
            commands.push("gnome-extensions list".to_string());
            commands.push("gsettings list-recursively org.gnome.desktop.interface | head -20".to_string());
        } else if de.contains("kde") || de.contains("plasma") {
            commands.push("plasmashell --version".to_string());
            commands.push("kreadconfig5 --file kdeglobals --group General --key ColorScheme".to_string());
        }
    }

    // Display server-specific
    let display = profile.system.display_server.as_deref().unwrap_or("");
    if q.contains("display") || q.contains("monitor") || q.contains("screen") || q.contains("resolution") {
        if display == "wayland" {
            commands.push("wlr-randr 2>/dev/null || echo 'wlr-randr not available'".to_string());
        } else {
            commands.push("xrandr".to_string());
        }
    }

    // AUR helper-specific
    let aur = profile.system.aur_helper.as_deref().unwrap_or("");
    if q.contains("aur") || (q.contains("install") && q.contains("aur")) {
        if !aur.is_empty() && aur != "none" {
            commands.push(format!("{} -Sua --devel", aur)); // Check AUR updates
        }
    }

    commands
}

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

/// v0.0.953: Health check result
#[derive(Clone, Debug)]
pub struct HealthCheckResult {
    pub category: String,
    pub status: HealthStatus,
    pub message: String,
    pub details: Option<String>,
}

/// v0.0.953: Health status levels
#[derive(Clone, Debug, PartialEq)]
pub enum HealthStatus {
    Ok,
    Warning,
    Critical,
}

/// v0.0.953: Cached health status
static HEALTH_CACHE: std::sync::RwLock<Option<Vec<HealthCheckResult>>> = std::sync::RwLock::new(None);

/// v0.0.953: Run proactive health checks and cache results
/// v0.0.992: Integrated with comprehensive monitoring system
/// Called at startup and can be called periodically
pub fn run_health_checks() -> Vec<HealthCheckResult> {
    info!("Running proactive health checks...");

    // v0.0.992: Run comprehensive monitoring and save issues
    run_comprehensive_monitoring();
    let mut results = Vec::new();

    // Check disk space
    if let Ok(output) = Command::new("df").arg("-h").arg("/").output() {
        if output.status.success() {
            let out = String::from_utf8_lossy(&output.stdout);
            // Parse disk usage percentage
            if let Some(line) = out.lines().nth(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    let usage_str = parts[4].trim_end_matches('%');
                    if let Ok(usage) = usage_str.parse::<u32>() {
                        let status = if usage >= 95 {
                            HealthStatus::Critical
                        } else if usage >= 85 {
                            HealthStatus::Warning
                        } else {
                            HealthStatus::Ok
                        };
                        results.push(HealthCheckResult {
                            category: "disk".to_string(),
                            status,
                            message: format!("Root partition {}% used", usage),
                            details: Some(line.to_string()),
                        });
                    }
                }
            }
        }
    }

    // Check memory
    if let Ok(output) = Command::new("free").arg("-m").output() {
        if output.status.success() {
            let out = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = out.lines().nth(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    if let (Ok(total), Ok(used)) = (parts[1].parse::<u64>(), parts[2].parse::<u64>()) {
                        let pct = (used * 100) / total.max(1);
                        let status = if pct >= 95 {
                            HealthStatus::Critical
                        } else if pct >= 85 {
                            HealthStatus::Warning
                        } else {
                            HealthStatus::Ok
                        };
                        results.push(HealthCheckResult {
                            category: "memory".to_string(),
                            status,
                            message: format!("Memory {}% used ({}/{}MB)", pct, used, total),
                            details: None,
                        });
                    }
                }
            }
        }
    }

    // Check failed services
    if let Ok(output) = Command::new("systemctl").args(["--failed", "--no-pager"]).output() {
        if output.status.success() {
            let out = String::from_utf8_lossy(&output.stdout);
            let failed_count = out.lines().filter(|l| l.contains("failed")).count();
            let status = if failed_count > 0 {
                HealthStatus::Warning
            } else {
                HealthStatus::Ok
            };
            results.push(HealthCheckResult {
                category: "services".to_string(),
                status,
                message: if failed_count > 0 {
                    format!("{} failed service(s)", failed_count)
                } else {
                    "All services running".to_string()
                },
                details: if failed_count > 0 { Some(out.to_string()) } else { None },
            });
        }
    }

    // Check system load
    if let Ok(output) = Command::new("cat").arg("/proc/loadavg").output() {
        if output.status.success() {
            let out = String::from_utf8_lossy(&output.stdout);
            if let Some(first) = out.split_whitespace().next() {
                if let Ok(load) = first.parse::<f32>() {
                    // Get CPU count for context
                    let ncpu = num_cpus().unwrap_or(4) as f32;
                    let status = if load > ncpu * 2.0 {
                        HealthStatus::Critical
                    } else if load > ncpu {
                        HealthStatus::Warning
                    } else {
                        HealthStatus::Ok
                    };
                    results.push(HealthCheckResult {
                        category: "load".to_string(),
                        status,
                        message: format!("System load: {:.2}", load),
                        details: None,
                    });
                }
            }
        }
    }

    // Check for recent errors in journal
    if let Ok(output) = Command::new("journalctl")
        .args(["-p", "err", "-b", "--no-pager", "-n", "10"])
        .output()
    {
        if output.status.success() {
            let out = String::from_utf8_lossy(&output.stdout);
            let error_count = out.lines().filter(|l| !l.is_empty()).count();
            let status = if error_count > 5 {
                HealthStatus::Warning
            } else {
                HealthStatus::Ok
            };
            results.push(HealthCheckResult {
                category: "errors".to_string(),
                status,
                message: format!("{} recent error(s) in journal", error_count),
                details: if error_count > 0 { Some(out.lines().take(5).collect::<Vec<_>>().join("\n")) } else { None },
            });
        }
    }

    // Cache results
    if let Ok(mut guard) = HEALTH_CACHE.write() {
        *guard = Some(results.clone());
    }

    let warnings = results.iter().filter(|r| r.status == HealthStatus::Warning).count();
    let criticals = results.iter().filter(|r| r.status == HealthStatus::Critical).count();
    info!("Health checks complete: {} checks, {} warnings, {} critical", results.len(), warnings, criticals);

    results
}

/// v0.0.953: Get cached health check results
pub fn get_cached_health() -> Option<Vec<HealthCheckResult>> {
    if let Ok(guard) = HEALTH_CACHE.read() {
        guard.clone()
    } else {
        None
    }
}

/// v0.0.953: Get summary of system health for instant answers
pub fn get_health_summary() -> String {
    match get_cached_health() {
        Some(results) => {
            let mut summary = Vec::new();
            // v0.3.30: Use plain text instead of emojis
            for r in &results {
                let icon = match r.status {
                    HealthStatus::Ok => "[OK]",
                    HealthStatus::Warning => "[WARN]",
                    HealthStatus::Critical => "[CRIT]",
                };
                summary.push(format!("{} {}: {}", icon, r.category, r.message));
            }
            summary.join("\n")
        }
        None => "Health checks not yet run".to_string(),
    }
}

/// Helper to get CPU count
fn num_cpus() -> Option<u32> {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .map(|s| s.matches("processor").count() as u32)
}
