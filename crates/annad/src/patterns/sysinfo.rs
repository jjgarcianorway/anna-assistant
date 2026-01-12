//! System info patterns for neofetch, inxi, dmidecode.
//! v0.0.980: Initial implementation.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create a sysinfo-related DeepUnderstanding
fn make_understanding(interpreted: &str, topic: &str, commands: &[&str]) -> DeepUnderstanding {
    DeepUnderstanding {
        interpreted_as: interpreted.to_string(),
        category: IntentCategory::Factual,
        confidence: 0.9,
        topic: Some(topic.to_string()),
        needs_confirmation: false,
        suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    }
}

type SysinfoPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match system info patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_fetch_tools(q)
        .or_else(|| match_hardware_info(q))
        .or_else(|| match_os_info(q))
        .or_else(|| match_bios_info(q))
        .or_else(|| match_system_summary(q))
        .or_else(|| match_health_check(q))
}

/// v0.0.990: System health check patterns
fn match_health_check(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SysinfoPattern] = &[
        // Health checks
        (&["system", "health"], "show system health", "health",
         &["systemctl --failed", "free -h | head -2", "df -h / | tail -1", "cat /proc/loadavg", "sensors 2>/dev/null | grep -E '°C|°F' | head -5"]),
        (&["health", "check"], "run health check", "health",
         &["systemctl --failed", "journalctl -p err -b --no-pager -n 5", "df -h | grep -E '(9[0-9]|100)%'", "free -h"]),
        (&["health", "status"], "show health status", "health",
         &["systemctl --failed", "uptime", "free -h | head -2", "sensors 2>/dev/null | head -10"]),
        // What's wrong
        (&["what", "wrong"], "check what's wrong", "health",
         &["systemctl --failed", "journalctl -p err -b --no-pager -n 10", "dmesg --level=err | tail -10"]),
        (&["any", "problems"], "check for problems", "health",
         &["systemctl --failed", "journalctl -p err -b --no-pager -n 5"]),
        (&["any", "issues"], "check for issues", "health",
         &["systemctl --failed", "journalctl -p err -b --no-pager -n 5", "df -h | grep -E '(9[0-9]|100)%'"]),
        // System status
        (&["system", "status"], "show system status", "health",
         &["uptime", "free -h | head -2", "df -h / | tail -1", "systemctl --failed"]),
        // Check system
        (&["check", "system"], "check system status", "health",
         &["systemctl --failed", "free -h", "df -h /", "cat /proc/loadavg"]),
        // Is system ok
        (&["system", "ok"], "check if system is ok", "health",
         &["systemctl --failed", "journalctl -p err -b --no-pager -n 3"]),
        (&["everything", "ok"], "check if everything ok", "health",
         &["systemctl --failed", "uptime", "free -h | head -2"]),
        // What changed
        (&["what", "changed"], "check what changed", "health",
         &["tail -20 /var/log/pacman.log | grep -E '\\[ALPM\\]'", "journalctl -b --no-pager -n 20", "last -5"]),
        (&["recent", "changes"], "show recent changes", "health",
         &["tail -30 /var/log/pacman.log | grep -E '\\[ALPM\\] (installed|upgraded|removed)'", "last -5"]),
        (&["changes", "detected"], "show detected changes", "health",
         &["tail -20 /var/log/pacman.log | grep -E '\\[ALPM\\]'", "journalctl --since '24 hours ago' -p warning --no-pager -n 10"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Fetch tool patterns (neofetch, fastfetch, etc.)
fn match_fetch_tools(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SysinfoPattern] = &[
        // Neofetch
        (&["neofetch"], "run neofetch", "sysinfo",
         &["neofetch 2>/dev/null || echo 'neofetch not installed'"]),
        // Fastfetch
        (&["fastfetch"], "run fastfetch", "sysinfo",
         &["fastfetch 2>/dev/null || echo 'fastfetch not installed'"]),
        // Screenfetch
        (&["screenfetch"], "run screenfetch", "sysinfo",
         &["screenfetch 2>/dev/null || echo 'screenfetch not installed'"]),
        // Inxi
        (&["inxi"], "run inxi", "sysinfo",
         &["inxi -Fxz 2>/dev/null || echo 'inxi not installed'"]),
        (&["inxi", "full"], "run inxi full", "sysinfo",
         &["inxi -Fxxxrz"]),
        // Hwinfo
        (&["hwinfo"], "run hwinfo", "sysinfo",
         &["hwinfo --short 2>/dev/null | head -50 || echo 'hwinfo not installed'"]),
        // System profiler
        (&["system", "profiler"], "show system profile", "sysinfo",
         &["inxi -Fxz 2>/dev/null || neofetch 2>/dev/null || cat /etc/os-release"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Hardware info patterns
fn match_hardware_info(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SysinfoPattern] = &[
        // lshw
        (&["lshw"], "show hardware with lshw", "sysinfo",
         &["lshw -short 2>/dev/null | head -40"]),
        (&["lshw", "full"], "show full hardware info", "sysinfo",
         &["lshw 2>/dev/null | head -100"]),
        // lscpu
        (&["lscpu"], "show CPU info with lscpu", "sysinfo",
         &["lscpu"]),
        // lspci
        (&["lspci"], "show PCI devices", "sysinfo",
         &["lspci"]),
        (&["lspci", "verbose"], "show PCI devices verbose", "sysinfo",
         &["lspci -v | head -100"]),
        // lsusb
        (&["lsusb"], "show USB devices", "sysinfo",
         &["lsusb"]),
        (&["lsusb", "verbose"], "show USB devices verbose", "sysinfo",
         &["lsusb -v 2>/dev/null | head -100"]),
        // lsblk
        (&["lsblk"], "show block devices", "sysinfo",
         &["lsblk -f"]),
        // Hardware summary
        (&["hardware", "summary"], "show hardware summary", "sysinfo",
         &["lshw -short 2>/dev/null || inxi -Fxz 2>/dev/null"]),
        (&["hardware", "overview"], "show hardware overview", "sysinfo",
         &["lshw -short 2>/dev/null | head -30"]),
        // Motherboard
        (&["motherboard", "info"], "show motherboard info", "sysinfo",
         &["cat /sys/devices/virtual/dmi/id/board_name 2>/dev/null", "cat /sys/devices/virtual/dmi/id/board_vendor 2>/dev/null"]),
        (&["baseboard"], "show baseboard info", "sysinfo",
         &["dmidecode -t baseboard 2>/dev/null | head -20"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// OS info patterns
fn match_os_info(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SysinfoPattern] = &[
        // OS release
        (&["os", "release"], "show OS release info", "sysinfo",
         &["cat /etc/os-release"]),
        (&["distro", "info"], "show distro info", "sysinfo",
         &["cat /etc/os-release", "lsb_release -a 2>/dev/null"]),
        // lsb_release
        (&["lsb_release"], "show LSB release info", "sysinfo",
         &["lsb_release -a 2>/dev/null"]),
        (&["lsb", "release"], "show LSB release", "sysinfo",
         &["lsb_release -a 2>/dev/null"]),
        // Arch version
        (&["arch", "version"], "show Arch version", "sysinfo",
         &["cat /etc/arch-release 2>/dev/null", "pacman -Q linux | head -1"]),
        // Kernel version
        (&["uname"], "show uname info", "sysinfo",
         &["uname -a"]),
        // Hostname
        (&["hostnamectl"], "show hostnamectl info", "sysinfo",
         &["hostnamectl"]),
        // Machine ID
        (&["machine", "id"], "show machine ID", "sysinfo",
         &["cat /etc/machine-id"]),
        // Product name
        (&["product", "name"], "show product name", "sysinfo",
         &["cat /sys/devices/virtual/dmi/id/product_name 2>/dev/null"]),
        // System vendor
        (&["system", "vendor"], "show system vendor", "sysinfo",
         &["cat /sys/devices/virtual/dmi/id/sys_vendor 2>/dev/null"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// BIOS/UEFI info patterns
fn match_bios_info(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SysinfoPattern] = &[
        // dmidecode
        (&["dmidecode"], "run dmidecode", "sysinfo",
         &["dmidecode 2>/dev/null | head -50"]),
        (&["dmidecode", "bios"], "show BIOS with dmidecode", "sysinfo",
         &["dmidecode -t bios 2>/dev/null"]),
        (&["dmidecode", "system"], "show system with dmidecode", "sysinfo",
         &["dmidecode -t system 2>/dev/null"]),
        (&["dmidecode", "memory"], "show memory with dmidecode", "sysinfo",
         &["dmidecode -t memory 2>/dev/null | head -100"]),
        // BIOS info
        (&["bios", "info"], "show BIOS info", "sysinfo",
         &["cat /sys/devices/virtual/dmi/id/bios_version 2>/dev/null", "dmidecode -t bios 2>/dev/null | head -20"]),
        (&["bios", "version"], "show BIOS version", "sysinfo",
         &["cat /sys/devices/virtual/dmi/id/bios_version 2>/dev/null"]),
        // UEFI info
        (&["uefi", "info"], "show UEFI info", "sysinfo",
         &["efibootmgr -v 2>/dev/null | head -20", "ls /sys/firmware/efi/ 2>/dev/null"]),
        (&["efi", "variables"], "show EFI variables", "sysinfo",
         &["ls /sys/firmware/efi/efivars/ 2>/dev/null | head -20"]),
        // Firmware
        (&["firmware", "version"], "show firmware version", "sysinfo",
         &["cat /sys/devices/virtual/dmi/id/bios_version 2>/dev/null", "dmesg | grep -i firmware | head -10"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// System summary patterns
fn match_system_summary(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SysinfoPattern] = &[
        // System info
        (&["system", "info"], "show system info", "sysinfo",
         &["hostnamectl", "cat /etc/os-release | head -5"]),
        (&["system", "details"], "show system details", "sysinfo",
         &["inxi -Fxz 2>/dev/null || (hostnamectl && lscpu | head -10)"]),
        // Computer info
        (&["computer", "info"], "show computer info", "sysinfo",
         &["hostnamectl", "lshw -short 2>/dev/null | head -20"]),
        (&["my", "computer"], "show my computer info", "sysinfo",
         &["neofetch 2>/dev/null || fastfetch 2>/dev/null || hostnamectl"]),
        // What system
        (&["what", "system"], "show what system", "sysinfo",
         &["cat /etc/os-release | head -3", "uname -a"]),
        // Specs
        (&["system", "specs"], "show system specs", "sysinfo",
         &["inxi -Fxz 2>/dev/null || (lscpu | head -10 && free -h && df -h /)"]),
        (&["my", "specs"], "show my system specs", "sysinfo",
         &["neofetch 2>/dev/null || inxi -Fxz 2>/dev/null"]),
        // Overview
        (&["system", "overview"], "show system overview", "sysinfo",
         &["hostnamectl", "uptime", "free -h | head -2", "df -h / | tail -1"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_tools() {
        assert!(match_patterns("neofetch").is_some());
        assert!(match_patterns("fastfetch").is_some());
        assert!(match_patterns("inxi").is_some());
    }

    #[test]
    fn test_hardware_info() {
        assert!(match_patterns("lshw").is_some());
        assert!(match_patterns("lscpu").is_some());
        assert!(match_patterns("lspci").is_some());
        assert!(match_patterns("lsusb").is_some());
    }

    #[test]
    fn test_os_info() {
        assert!(match_patterns("os release").is_some());
        assert!(match_patterns("distro info").is_some());
        assert!(match_patterns("hostnamectl").is_some());
    }

    #[test]
    fn test_bios_info() {
        assert!(match_patterns("dmidecode").is_some());
        assert!(match_patterns("bios info").is_some());
        assert!(match_patterns("uefi info").is_some());
    }

    #[test]
    fn test_system_summary() {
        assert!(match_patterns("system info").is_some());
        assert!(match_patterns("my computer").is_some());
        assert!(match_patterns("system specs").is_some());
    }
}
