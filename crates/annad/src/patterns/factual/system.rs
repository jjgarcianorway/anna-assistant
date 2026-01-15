//! System info, storage, and hardware patterns.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};
use super::super::contains_word;
use super::FactualPattern;

pub fn match_system_info(q: &str) -> Option<DeepUnderstanding> {
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

pub fn match_storage(q: &str) -> Option<DeepUnderstanding> {
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

pub fn match_hardware(q: &str) -> Option<DeepUnderstanding> {
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
