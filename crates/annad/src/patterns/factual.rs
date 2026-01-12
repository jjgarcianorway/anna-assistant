//! Factual query patterns - simple questions with known commands
//!
//! These are common "what is X" questions that can be answered immediately
//! with pre-cached commands, bypassing the LLM command selection pipeline.
//! v0.0.937: Added thermal, process, audio, and logs patterns
//! v0.0.945: Added time/date, environment, and shell patterns
//! v0.1.0: Use word boundary matching to prevent "update" matching "date"

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};
use super::contains_word;

/// Match factual queries that have simple, direct answers
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    // System info queries
    if let Some(u) = match_system_info(q) {
        return Some(u);
    }
    // Storage queries
    if let Some(u) = match_storage(q) {
        return Some(u);
    }
    // Network queries
    if let Some(u) = match_network(q) {
        return Some(u);
    }
    // v0.1.0: Process/load queries checked BEFORE hardware
    // so "cpu usage" matches before "what cpu" (which uses lscpu)
    if let Some(u) = match_processes(q) {
        return Some(u);
    }
    // v0.0.937: Thermal/temperature queries
    if let Some(u) = match_thermal(q) {
        return Some(u);
    }
    // Hardware queries (after process queries to avoid "what cpu" matching usage questions)
    if let Some(u) = match_hardware(q) {
        return Some(u);
    }
    // Package queries
    if let Some(u) = match_packages(q) {
        return Some(u);
    }
    // Service queries
    if let Some(u) = match_services(q) {
        return Some(u);
    }
    // v0.0.937: Audio/sound queries
    if let Some(u) = match_audio(q) {
        return Some(u);
    }
    // v0.0.937: Boot/log queries
    if let Some(u) = match_logs(q) {
        return Some(u);
    }
    // v0.0.945: Time/date queries
    if let Some(u) = match_time(q) {
        return Some(u);
    }
    // v0.0.945: Environment/shell queries
    if let Some(u) = match_environment(q) {
        return Some(u);
    }
    // v0.0.945: User/group queries
    if let Some(u) = match_users(q) {
        return Some(u);
    }
    None
}

/// Pattern with keywords, description, topic, and pre-cached commands
type FactualPattern = (&'static [&'static str], &'static str, &'static str, &'static [&'static str]);

fn match_system_info(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[FactualPattern] = &[
        // Kernel
        (&["kernel", "version"], "kernel version query", "system", &["uname -r"]),
        (&["what", "kernel"], "kernel version query", "system", &["uname -r"]),
        (&["running", "kernel"], "kernel version query", "system", &["uname -r"]),
        (&["which", "kernel"], "kernel version query", "system", &["uname -r"]),
        (&["uname"], "kernel info query", "system", &["uname -a"]),
        // Hostname
        (&["hostname"], "hostname query", "system", &["hostnamectl"]),
        (&["computer", "name"], "hostname query", "system", &["hostnamectl"]),
        (&["machine", "name"], "hostname query", "system", &["hostnamectl"]),
        // Uptime
        (&["uptime"], "system uptime query", "system", &["uptime -p"]),
        (&["how", "long", "running"], "system uptime query", "system", &["uptime -p"]),
        (&["last", "reboot"], "system uptime query", "system", &["uptime -s", "last reboot | head -5"]),
        // Distro
        (&["what", "distro"], "distribution query", "system", &["cat /etc/os-release | head -5"]),
        (&["which", "distro"], "distribution query", "system", &["cat /etc/os-release | head -5"]),
        (&["os", "version"], "OS version query", "system", &["cat /etc/os-release | head -5"]),
        (&["linux", "version"], "distribution query", "system", &["cat /etc/os-release | head -5"]),
        // Users
        (&["who", "logged"], "logged users query", "system", &["who"]),
        (&["current", "user"], "current user query", "system", &["whoami", "id"]),
        (&["whoami"], "current user query", "system", &["whoami", "id"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| contains_word(q, kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.95,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

fn match_storage(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[FactualPattern] = &[
        // Disk usage
        (&["disk", "usage"], "disk usage query", "storage", &["df -h"]),
        (&["disk", "space"], "disk space query", "storage", &["df -h"]),
        (&["how", "much", "disk"], "disk usage query", "storage", &["df -h"]),
        (&["storage", "usage"], "disk usage query", "storage", &["df -h"]),
        (&["free", "space"], "free disk space query", "storage", &["df -h"]),
        (&["disk", "full"], "disk space query", "storage", &["df -h", "du -sh /* 2>/dev/null | sort -hr | head -10"]),
        // Partitions
        (&["partition"], "partition info query", "storage", &["lsblk -f"]),
        (&["mount"], "mounted filesystems query", "storage", &["mount | grep -E '^/dev'"]),
        (&["what", "mounted"], "mounted filesystems query", "storage", &["mount | grep -E '^/dev'"]),
        (&["filesystem"], "filesystem info query", "storage", &["df -Th"]),
        // Block devices
        (&["list", "disk"], "disk list query", "storage", &["lsblk"]),
        (&["lsblk"], "block device query", "storage", &["lsblk -f"]),
        (&["what", "drive"], "drive info query", "storage", &["lsblk"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| contains_word(q, kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.95,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

fn match_network(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[FactualPattern] = &[
        // IP address
        (&["ip", "address"], "IP address query", "network", &["ip -4 addr show | grep inet | grep -v 127.0.0.1"]),
        (&["my", "ip"], "IP address query", "network", &["ip -4 addr show | grep inet | grep -v 127.0.0.1"]),
        (&["show", "ip"], "IP address query", "network", &["ip -4 addr show | grep inet | grep -v 127.0.0.1"]),
        (&["what", "ip"], "IP address query", "network", &["ip -4 addr show | grep inet | grep -v 127.0.0.1"]),
        // Network interfaces
        (&["network", "interface"], "network interfaces query", "network", &["ip link show"]),
        (&["list", "interface"], "network interfaces query", "network", &["ip link show"]),
        // DNS
        (&["dns", "server"], "DNS server query", "network", &["resolvectl status | head -20"]),
        (&["nameserver"], "DNS server query", "network", &["cat /etc/resolv.conf"]),
        // Gateway
        (&["gateway"], "gateway query", "network", &["ip route | grep default"]),
        (&["default", "route"], "default route query", "network", &["ip route | grep default"]),
        // Connection status
        (&["network", "status"], "network status query", "network", &["nmcli general status"]),
        (&["connected", "network"], "network connection query", "network", &["nmcli connection show --active"]),
        // Ports
        (&["listening", "port"], "listening ports query", "network", &["ss -tlnp 2>/dev/null | head -20"]),
        (&["open", "port"], "open ports query", "network", &["ss -tlnp 2>/dev/null | head -20"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| contains_word(q, kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.95,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

fn match_hardware(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[FactualPattern] = &[
        // GPU
        (&["what", "gpu"], "GPU info query", "hardware", &["lspci | grep -i vga", "lspci | grep -i 3d"]),
        (&["which", "gpu"], "GPU info query", "hardware", &["lspci | grep -i vga", "lspci | grep -i 3d"]),
        (&["graphics", "card"], "GPU info query", "hardware", &["lspci | grep -i vga"]),
        (&["video", "card"], "GPU info query", "hardware", &["lspci | grep -i vga"]),
        (&["nvidia"], "NVIDIA GPU query", "hardware", &["nvidia-smi 2>/dev/null || lspci | grep -i nvidia"]),
        (&["amd", "gpu"], "AMD GPU query", "hardware", &["lspci | grep -i amd | grep -i vga"]),
        // CPU
        (&["what", "cpu"], "CPU info query", "hardware", &["lscpu | head -15"]),
        (&["which", "cpu"], "CPU info query", "hardware", &["lscpu | head -15"]),
        (&["processor"], "CPU info query", "hardware", &["lscpu | head -15"]),
        (&["how", "many", "core"], "CPU cores query", "hardware", &["nproc", "lscpu | grep -E '^CPU\\(s\\)|Core'"]),
        (&["cpu", "model"], "CPU model query", "hardware", &["lscpu | grep 'Model name'"]),
        // RAM
        (&["how", "much", "ram"], "RAM info query", "hardware", &["free -h"]),
        (&["total", "memory"], "total memory query", "hardware", &["free -h"]),
        (&["memory", "size"], "memory size query", "hardware", &["free -h"]),
        (&["ram", "size"], "RAM size query", "hardware", &["free -h"]),
        (&["how", "much", "memory"], "memory info query", "hardware", &["free -h"]),
        // General hardware
        (&["hardware", "info"], "hardware info query", "hardware", &["lscpu | head -10", "free -h", "lsblk"]),
        (&["system", "spec"], "system specs query", "hardware", &["lscpu | head -10", "free -h", "lspci | grep -i vga"]),
        // USB
        (&["usb", "device"], "USB devices query", "hardware", &["lsusb"]),
        (&["list", "usb"], "USB devices query", "hardware", &["lsusb"]),
        // PCI
        (&["pci", "device"], "PCI devices query", "hardware", &["lspci"]),
        // Battery
        (&["battery", "status"], "battery status query", "hardware", &["upower -i /org/freedesktop/UPower/devices/battery_BAT0 2>/dev/null || cat /sys/class/power_supply/BAT*/capacity 2>/dev/null"]),
        (&["battery", "level"], "battery level query", "hardware", &["cat /sys/class/power_supply/BAT*/capacity 2>/dev/null"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| contains_word(q, kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.95,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

fn match_packages(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[FactualPattern] = &[
        // Installed packages
        (&["installed", "package"], "installed packages query", "packages", &["pacman -Q | wc -l", "pacman -Qe | head -20"]),
        (&["list", "package"], "package list query", "packages", &["pacman -Qe | head -30"]),
        (&["how", "many", "package"], "package count query", "packages", &["pacman -Q | wc -l"]),
        // Specific package check
        (&["is", "installed"], "package installation check", "packages", &["pacman -Qs"]),
        // Updates - v0.1.0: show actual updates, not just counts
        (&["available", "update"], "available updates query", "packages", &["checkupdates 2>/dev/null | head -30 || pacman -Qu 2>/dev/null | head -30"]),
        (&["available", "updates"], "available updates query", "packages", &["checkupdates 2>/dev/null | head -30 || pacman -Qu 2>/dev/null | head -30"]),
        (&["pending", "update"], "pending updates query", "packages",
            &["echo 'Pending updates:' && checkupdates 2>/dev/null | head -30 || pacman -Qu 2>/dev/null | head -30 || echo 'No updates pending'"]),
        (&["pending", "updates"], "pending updates query", "packages",
            &["echo 'Pending updates:' && checkupdates 2>/dev/null | head -30 || pacman -Qu 2>/dev/null | head -30 || echo 'No updates pending'"]),
        (&["any", "updates"], "check for updates", "packages",
            &["checkupdates 2>/dev/null | head -20 || pacman -Qu 2>/dev/null | head -20 || echo 'System is up to date'"]),
        (&["updates", "available"], "check available updates", "packages",
            &["checkupdates 2>/dev/null | head -30 || pacman -Qu 2>/dev/null | head -30 || echo 'No updates available'"]),
        // Orphans
        (&["orphan", "package"], "orphan packages query", "packages", &["pacman -Qtdq 2>/dev/null || echo 'No orphans found'"]),
        // Recently installed
        (&["recently", "installed"], "recent packages query", "packages", &["grep -E 'installed|upgraded' /var/log/pacman.log | tail -20"]),
        (&["last", "installed"], "recent packages query", "packages", &["grep 'installed' /var/log/pacman.log | tail -10"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| contains_word(q, kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.95,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

fn match_services(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[FactualPattern] = &[
        // Failed services (both singular and plural)
        (&["failed", "services"], "failed services query", "services", &["systemctl --failed"]),
        (&["failed", "service"], "failed services query", "services", &["systemctl --failed"]),
        (&["service", "status"], "service status query", "services", &["systemctl status"]),
        // Running services (both singular and plural)
        (&["running", "services"], "running services query", "services", &["systemctl list-units --type=service --state=running | head -20"]),
        (&["running", "service"], "running services query", "services", &["systemctl list-units --type=service --state=running | head -20"]),
        (&["active", "services"], "active services query", "services", &["systemctl list-units --type=service --state=active | head -20"]),
        (&["active", "service"], "active services query", "services", &["systemctl list-units --type=service --state=active | head -20"]),
        // List services (both singular and plural)
        (&["list", "services"], "service list query", "services", &["systemctl list-unit-files --type=service | head -30"]),
        (&["list", "service"], "service list query", "services", &["systemctl list-unit-files --type=service | head -30"]),
        // Timers
        (&["systemd", "timer"], "systemd timers query", "services", &["systemctl list-timers"]),
        (&["systemd", "timers"], "systemd timers query", "services", &["systemctl list-timers"]),
        (&["list", "timer"], "timer list query", "services", &["systemctl list-timers"]),
        (&["list", "timers"], "timer list query", "services", &["systemctl list-timers"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| contains_word(q, kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.95,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

/// v0.0.937: Temperature and thermal queries
fn match_thermal(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[FactualPattern] = &[
        // CPU temperature
        (&["cpu", "temp"], "CPU temperature query", "thermal", &["sensors 2>/dev/null | grep -E 'Core|Tctl|temp' || cat /sys/class/thermal/thermal_zone*/temp 2>/dev/null"]),
        (&["temperature"], "system temperature query", "thermal", &["sensors 2>/dev/null || cat /sys/class/thermal/thermal_zone*/temp 2>/dev/null"]),
        (&["how", "hot"], "temperature query", "thermal", &["sensors 2>/dev/null | head -20"]),
        // Fan speed
        (&["fan", "speed"], "fan speed query", "thermal", &["sensors 2>/dev/null | grep -i fan"]),
        (&["fan", "status"], "fan status query", "thermal", &["sensors 2>/dev/null | grep -i fan"]),
        // Thermal sensors
        (&["sensor"], "sensor readings query", "thermal", &["sensors 2>/dev/null"]),
        (&["lm_sensor"], "lm_sensors query", "thermal", &["sensors 2>/dev/null"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| contains_word(q, kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.95,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

/// v0.0.937: Process and system load queries
/// v0.1.0: Added "average cpu" patterns for queries like "average usage of my cpu"
fn match_processes(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[FactualPattern] = &[
        // CPU usage - specific patterns first
        (&["average", "cpu"], "CPU usage query", "processes", &["mpstat 1 1 2>/dev/null || top -bn1 | head -15"]),
        (&["average", "usage"], "system usage query", "processes", &["top -bn1 | head -15", "free -h"]),
        (&["cpu", "usage"], "CPU usage query", "processes", &["top -bn1 | head -15"]),
        (&["cpu", "utilization"], "CPU utilization query", "processes", &["mpstat 1 1 2>/dev/null || top -bn1 | head -15"]),
        (&["cpu", "load"], "CPU load query", "processes", &["uptime", "cat /proc/loadavg"]),
        (&["system", "load"], "system load query", "processes", &["uptime", "cat /proc/loadavg"]),
        (&["load", "average"], "load average query", "processes", &["uptime"]),
        // Memory usage
        (&["memory", "usage"], "memory usage query", "processes", &["free -h", "ps aux --sort=-%mem | head -10"]),
        (&["what", "using", "memory"], "memory consumers query", "processes", &["ps aux --sort=-%mem | head -10"]),
        (&["what", "using", "ram"], "RAM consumers query", "processes", &["ps aux --sort=-%mem | head -10"]),
        // Process list
        (&["running", "process"], "running processes query", "processes", &["ps aux --sort=-%cpu | head -15"]),
        (&["list", "process"], "process list query", "processes", &["ps aux | head -20"]),
        (&["top", "process"], "top processes query", "processes", &["ps aux --sort=-%cpu | head -10"]),
        // What's using CPU
        (&["what", "using", "cpu"], "CPU consumers query", "processes", &["ps aux --sort=-%cpu | head -10"]),
        (&["high", "cpu"], "high CPU usage query", "processes", &["ps aux --sort=-%cpu | head -10"]),
        // Zombie processes
        (&["zombie", "process"], "zombie processes query", "processes", &["ps aux | grep -w Z | grep -v grep || echo 'No zombies found'"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| contains_word(q, kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.95,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

/// v0.0.937: Audio and sound queries
fn match_audio(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[FactualPattern] = &[
        // Audio status
        (&["audio", "device"], "audio devices query", "audio", &["pactl list sinks short 2>/dev/null || wpctl status 2>/dev/null | head -30"]),
        (&["sound", "device"], "sound devices query", "audio", &["pactl list sinks short 2>/dev/null || wpctl status 2>/dev/null | head -30"]),
        (&["audio", "output"], "audio output query", "audio", &["pactl get-default-sink 2>/dev/null || wpctl status 2>/dev/null | head -20"]),
        // Volume
        (&["volume"], "volume level query", "audio", &["pactl get-sink-volume @DEFAULT_SINK@ 2>/dev/null || wpctl get-volume @DEFAULT_AUDIO_SINK@ 2>/dev/null"]),
        (&["audio", "level"], "audio level query", "audio", &["pactl get-sink-volume @DEFAULT_SINK@ 2>/dev/null"]),
        // Muted
        (&["muted"], "mute status query", "audio", &["pactl get-sink-mute @DEFAULT_SINK@ 2>/dev/null"]),
        // Microphone
        (&["microphone"], "microphone query", "audio", &["pactl list sources short 2>/dev/null | grep -v monitor"]),
        (&["mic"], "mic query", "audio", &["pactl list sources short 2>/dev/null | grep -v monitor"]),
        // Pipewire/Pulse
        (&["pipewire", "status"], "Pipewire status query", "audio", &["systemctl --user status pipewire", "wpctl status | head -30"]),
        (&["pulseaudio", "status"], "PulseAudio status query", "audio", &["pactl info | head -15"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| contains_word(q, kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.95,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

/// v0.0.937: Boot and log queries
fn match_logs(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[FactualPattern] = &[
        // Boot time
        (&["boot", "time"], "boot time query", "logs", &["systemd-analyze"]),
        (&["startup", "time"], "startup time query", "logs", &["systemd-analyze"]),
        (&["boot", "slow"], "slow boot analysis", "logs", &["systemd-analyze blame | head -15"]),
        // Boot logs
        (&["boot", "log"], "boot log query", "logs", &["journalctl -b -p err..warning | head -30"]),
        (&["boot", "error"], "boot errors query", "logs", &["journalctl -b -p err | head -20"]),
        (&["dmesg"], "kernel messages query", "logs", &["dmesg | tail -30"]),
        // v0.1.0: "Any errors" patterns - show actual errors, not counts
        (&["any", "errors"], "check for errors", "logs", &["journalctl -b -p err --no-pager | head -20 || echo 'No errors found'"]),
        (&["any", "log", "errors"], "check log errors", "logs", &["journalctl -b -p err --no-pager | head -20 || echo 'No errors found'"]),
        (&["there", "errors"], "check for errors", "logs", &["journalctl -b -p err --no-pager | head -20 || echo 'No errors in logs'"]),
        (&["no", "errors"], "verify no errors", "logs", &["journalctl -b -p err --no-pager | head -20 || echo 'Confirmed: No errors found'"]),
        (&["log", "errors"], "show log errors", "logs", &["journalctl -b -p err --no-pager | head -30"]),
        // System logs
        (&["system", "log"], "system log query", "logs", &["journalctl -p err..warning --since '1 hour ago' | head -30"]),
        (&["error", "log"], "error log query", "logs", &["journalctl -p err --since '1 hour ago' | head -30"]),
        (&["recent", "error"], "recent errors query", "logs", &["journalctl -p err --since '1 hour ago' | head -20"]),
        (&["recent", "errors"], "recent errors query", "logs", &["journalctl -p err --since '1 hour ago' | head -20"]),
        // Journal
        (&["journal"], "journal query", "logs", &["journalctl --since '1 hour ago' | tail -30"]),
        (&["journalctl"], "journalctl query", "logs", &["journalctl --since '1 hour ago' | tail -30"]),
        // Kernel messages
        (&["kernel", "log"], "kernel log query", "logs", &["dmesg | tail -30"]),
        (&["kernel", "error"], "kernel errors query", "logs", &["dmesg --level=err,warn | tail -20"]),
        (&["kernel", "errors"], "kernel errors query", "logs", &["dmesg --level=err,warn | tail -20"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| contains_word(q, kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.95,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

/// v0.0.945: Time and date queries
fn match_time(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[FactualPattern] = &[
        // Current time
        (&["what", "time"], "current time query", "time", &["date +%H:%M:%S"]),
        (&["current", "time"], "current time query", "time", &["date +%H:%M:%S"]),
        (&["show", "time"], "current time query", "time", &["date +%H:%M:%S"]),
        // Current date
        (&["what", "date"], "current date query", "time", &["date +%Y-%m-%d"]),
        (&["current", "date"], "current date query", "time", &["date +%Y-%m-%d"]),
        (&["today", "date"], "current date query", "time", &["date +%Y-%m-%d"]),
        // Full datetime
        (&["date", "time"], "datetime query", "time", &["date"]),
        // Timezone
        (&["timezone"], "timezone query", "time", &["timedatectl | grep 'Time zone'"]),
        (&["time", "zone"], "timezone query", "time", &["timedatectl | grep 'Time zone'"]),
        (&["what", "tz"], "timezone query", "time", &["timedatectl | grep 'Time zone'"]),
        // Calendar
        (&["calendar"], "calendar query", "time", &["cal"]),
        (&["what", "day"], "day of week query", "time", &["date +%A"]),
        (&["what", "month"], "month query", "time", &["date +%B"]),
        (&["what", "year"], "year query", "time", &["date +%Y"]),
        // NTP status
        (&["ntp", "status"], "NTP status query", "time", &["timedatectl | grep -E 'NTP|synchronized'"]),
        (&["time", "sync"], "time sync status query", "time", &["timedatectl | grep -E 'NTP|synchronized'"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| contains_word(q, kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.95,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

/// v0.0.945: Environment and shell queries
/// v0.1.0: Added exclusion check for size-related queries
fn match_environment(q: &str) -> Option<DeepUnderstanding> {
    // v0.1.0: Skip environment patterns if query is about size/space/largest
    // These should go to filesystem patterns instead
    let size_keywords = ["largest", "biggest", "size", "space", "usage", "top", "du ", "how big", "how much"];
    if size_keywords.iter().any(|kw| contains_word(q, kw)) {
        return None;
    }

    let patterns: &[FactualPattern] = &[
        // Shell
        (&["what", "shell"], "shell query", "environment", &["echo $SHELL", "basename $SHELL"]),
        (&["which", "shell"], "shell query", "environment", &["echo $SHELL"]),
        (&["my", "shell"], "shell query", "environment", &["echo $SHELL"]),
        (&["default", "shell"], "default shell query", "environment", &["getent passwd $USER | cut -d: -f7"]),
        // Home directory
        (&["home", "directory"], "home directory query", "environment", &["echo $HOME"]),
        (&["home", "folder"], "home directory query", "environment", &["echo $HOME"]),
        // PATH
        (&["show", "path"], "PATH query", "environment", &["echo $PATH | tr ':' '\\n'"]),
        (&["what", "path"], "PATH query", "environment", &["echo $PATH | tr ':' '\\n'"]),
        // Environment variables
        (&["environment", "variable"], "env vars query", "environment", &["env | head -30"]),
        (&["env", "var"], "env vars query", "environment", &["env | head -30"]),
        (&["list", "env"], "env vars query", "environment", &["env | head -30"]),
        // Editor
        (&["default", "editor"], "editor query", "environment", &["echo $EDITOR", "echo $VISUAL"]),
        (&["what", "editor"], "editor query", "environment", &["echo $EDITOR", "echo $VISUAL"]),
        // Display
        (&["display", "variable"], "display query", "environment", &["echo $DISPLAY", "echo $WAYLAND_DISPLAY"]),
        (&["session", "type"], "session type query", "environment", &["echo $XDG_SESSION_TYPE"]),
        // Locale
        (&["locale"], "locale query", "environment", &["locale"]),
        (&["language", "setting"], "locale query", "environment", &["echo $LANG", "locale"]),
        // Terminal
        (&["terminal", "emulator"], "terminal query", "environment", &["echo $TERM"]),
        (&["what", "term"], "terminal query", "environment", &["echo $TERM"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| contains_word(q, kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.95,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

/// v0.0.945: User and group queries
fn match_users(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[FactualPattern] = &[
        // Current user
        (&["my", "username"], "username query", "users", &["whoami"]),
        (&["my", "user"], "user info query", "users", &["id"]),
        (&["what", "user", "am"], "username query", "users", &["whoami", "id"]),
        // User groups
        (&["my", "group"], "groups query", "users", &["groups"]),
        (&["what", "group"], "groups query", "users", &["groups"]),
        (&["list", "group"], "all groups query", "users", &["cat /etc/group | cut -d: -f1 | head -20"]),
        // All users
        (&["list", "user"], "all users query", "users", &["cat /etc/passwd | cut -d: -f1 | head -20"]),
        (&["system", "user"], "system users query", "users", &["cat /etc/passwd | awk -F: '$3 < 1000 {print $1}'"]),
        // User info
        (&["user", "id"], "user ID query", "users", &["id"]),
        (&["uid"], "UID query", "users", &["id -u"]),
        (&["gid"], "GID query", "users", &["id -g"]),
        // Sudo
        (&["sudo", "access"], "sudo access query", "users", &["sudo -l 2>&1 | head -10"]),
        (&["can", "sudo"], "sudo capability query", "users", &["groups | grep -q sudo && echo 'Yes' || echo 'No'"]),
        (&["sudoer"], "sudoers query", "users", &["getent group sudo wheel | cut -d: -f4"]),
        // Login history
        (&["login", "history"], "login history query", "users", &["last | head -10"]),
        (&["last", "login"], "last login query", "users", &["lastlog -u $USER"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| contains_word(q, kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.95,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_usage() {
        let result = match_patterns("what is my disk usage");
        assert!(result.is_some());
        let u = result.unwrap();
        assert!(!u.suggested_commands.is_empty());
        assert!(u.suggested_commands.iter().any(|c| c.contains("df")));
    }

    #[test]
    fn test_ram() {
        let result = match_patterns("how much ram do I have");
        assert!(result.is_some());
        let u = result.unwrap();
        assert!(u.suggested_commands.iter().any(|c| c.contains("free")));
    }

    #[test]
    fn test_gpu() {
        let result = match_patterns("what gpu do I have");
        assert!(result.is_some());
        let u = result.unwrap();
        assert!(u.suggested_commands.iter().any(|c| c.contains("lspci")));
    }

    #[test]
    fn test_ip() {
        let result = match_patterns("what is my ip address");
        assert!(result.is_some());
        let u = result.unwrap();
        assert!(u.suggested_commands.iter().any(|c| c.contains("ip")));
    }

    #[test]
    fn test_kernel() {
        let result = match_patterns("what kernel am I running");
        assert!(result.is_some());
        let u = result.unwrap();
        assert!(u.suggested_commands.iter().any(|c| c.contains("uname")));
    }

    #[test]
    fn test_failed_services() {
        let result = match_patterns("list failed services");
        assert!(result.is_some());
        let u = result.unwrap();
        assert!(u.suggested_commands.iter().any(|c| c.contains("systemctl")));
    }
}
