//! Performance and resource usage patterns
//! v0.0.914: Added suggested_commands for diagnostics

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Pattern with keywords, description, and diagnostic commands
type PerfPattern = (&'static [&'static str], &'static str, &'static [&'static str]);

/// Match performance-related queries
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    // Thermal/fan issues
    if let Some(u) = match_thermal(q) {
        return Some(u);
    }
    // Memory issues
    if let Some(u) = match_memory(q) {
        return Some(u);
    }
    // CPU/process issues
    if let Some(u) = match_cpu(q) {
        return Some(u);
    }
    // Service/shutdown issues
    if let Some(u) = match_services(q) {
        return Some(u);
    }
    // General slowness
    if let Some(u) = match_slowness(q) {
        return Some(u);
    }
    None
}

fn match_thermal(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[PerfPattern] = &[
        (&["fan", "spin", "idle"], "fan running when idle",
            &["sensors", "cat /sys/class/thermal/thermal_zone*/temp",
              "ps aux --sort=-%cpu | head -5"]),
        (&["fan", "loud"], "loud fan noise",
            &["sensors", "ps aux --sort=-%cpu | head -5"]),
        (&["overheating"], "system overheating",
            &["sensors", "cat /sys/class/thermal/thermal_zone*/temp"]),
        (&["thermal", "throttl"], "thermal throttling",
            &["dmesg | grep -i thermal | tail -10", "sensors"]),
        (&["cpu", "temp", "high"], "high CPU temperature",
            &["sensors", "cat /sys/class/thermal/thermal_zone*/temp"]),
        (&["hot", "laptop"], "laptop overheating",
            &["sensors", "cat /proc/acpi/thermal_zone/*/temperature 2>/dev/null"]),
    ];

    for (keywords, interpreted, commands) in patterns {
        if keywords.iter().all(|kw| q.to_lowercase().contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.9,
                topic: Some("hardware".to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

fn match_memory(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[PerfPattern] = &[
        (&["memory", "leak"], "memory leak detection",
            &["ps aux --sort=-%mem | head -10", "free -h"]),
        (&["ram", "usage", "high"], "high RAM usage",
            &["free -h", "ps aux --sort=-%mem | head -10"]),
        (&["ram", "full"], "RAM full",
            &["free -h", "ps aux --sort=-%mem | head -10"]),
        (&["using", "all", "ram"], "high RAM usage",
            &["free -h", "ps aux --sort=-%mem | head -10"]),
        (&["firefox", "memory"], "Firefox memory usage",
            &["ps aux | grep -i firefox | head -5", "about:memory in Firefox"]),
        (&["chrome", "memory"], "Chrome memory usage",
            &["ps aux | grep -i chrom | head -5"]),
        (&["browser", "memory"], "browser memory usage",
            &["ps aux | grep -E 'firefox|chrom' | head -5"]),
        (&["oom", "killer"], "OOM killer triggered",
            &["dmesg | grep -i 'out of memory' | tail -10", "free -h"]),
        (&["out of memory"], "out of memory error",
            &["dmesg | grep -i 'out of memory' | tail -5", "free -h"]),
        (&["swap", "full"], "swap space full",
            &["swapon --show", "free -h", "ps aux --sort=-%mem | head -5"]),
    ];

    for (keywords, interpreted, commands) in patterns {
        if keywords.iter().all(|kw| q.to_lowercase().contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.9,
                topic: Some("performance".to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

fn match_cpu(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[PerfPattern] = &[
        (&["cpu", "usage", "high"], "high CPU usage",
            &["ps aux --sort=-%cpu | head -10"]),
        (&["cpu", "100"], "CPU at 100%",
            &["ps aux --sort=-%cpu | head -10"]),
        (&["what", "using", "cpu"], "CPU usage query",
            &["ps aux --sort=-%cpu | head -10"]),
        (&["process", "cpu"], "process CPU usage",
            &["ps aux --sort=-%cpu | head -10"]),
        (&["zombie", "process"], "zombie processes",
            &["ps aux | grep 'Z' | head -10"]),
        (&["process", "still", "running"], "orphan process",
            &["ps aux | head -10"]),
    ];

    for (keywords, interpreted, commands) in patterns {
        if keywords.iter().all(|kw| q.to_lowercase().contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.9,
                topic: Some("performance".to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

/// Service pattern with category
type ServicePattern = (&'static [&'static str], &'static str, &'static str, IntentCategory, &'static [&'static str]);

fn match_services(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[ServicePattern] = &[
        (&["failed", "service"], "failed systemd services", "services", IntentCategory::Troubleshoot,
            &["systemctl --failed", "journalctl -p err -b | tail -20"]),
        (&["service", "fail"], "service failure", "services", IntentCategory::Troubleshoot,
            &["systemctl --failed", "journalctl -xe | tail -30"]),
        (&["what", "using", "port"], "port usage query", "network", IntentCategory::Factual,
            &["ss -tulpn | head -20"]),
        (&["port", "in", "use"], "port in use query", "network", IntentCategory::Factual,
            &["ss -tulpn | head -20"]),
        (&["won't", "shut", "down"], "shutdown hanging", "services", IntentCategory::Troubleshoot,
            &["systemctl list-jobs", "systemctl --state=running"]),
        (&["shutdown", "stuck"], "shutdown stuck", "services", IntentCategory::Troubleshoot,
            &["systemctl list-jobs", "echo 'Try: sudo systemctl --force poweroff'"]),
        (&["prevent", "shutdown"], "process preventing shutdown", "services", IntentCategory::Troubleshoot,
            &["systemctl list-jobs"]),
    ];

    for (keywords, interpreted, topic, category, commands) in patterns {
        if keywords.iter().all(|kw| q.to_lowercase().contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: category.clone(),
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

/// Slowness pattern with topic
type SlownessPattern = (&'static [&'static str], &'static str, &'static str, &'static [&'static str]);

fn match_slowness(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SlownessPattern] = &[
        // Boot time
        (&["boot", "time", "slow"], "slow boot time", "boot",
            &["systemd-analyze", "systemd-analyze blame | head -10"]),
        (&["boot", "takes", "long"], "slow boot time", "boot",
            &["systemd-analyze", "systemd-analyze blame | head -10"]),
        (&["slow", "boot"], "slow boot time", "boot",
            &["systemd-analyze", "systemd-analyze blame | head -10"]),
        // System slow
        (&["system", "slow"], "system performance issue", "performance",
            &["ps aux --sort=-%cpu | head -5", "free -h", "df -h"]),
        (&["computer", "slow"], "system performance issue", "performance",
            &["ps aux --sort=-%cpu | head -5", "free -h"]),
        (&["it's", "slow"], "system performance issue", "performance",
            &["ps aux --sort=-%cpu | head -5", "free -h"]),
        (&["everything", "slow"], "system performance issue", "performance",
            &["ps aux --sort=-%cpu | head -5", "free -h", "iostat 1 2"]),
        // Desktop/UI slow
        (&["workspace", "stutter"], "workspace switching stutter", "display",
            &["echo 'Check compositor: try picom -b or disable effects'"]),
        (&["compositor", "lag"], "compositor lag", "display",
            &["echo 'Try: picom --vsync or disable compositor'"]),
        (&["animation", "stutter"], "animation stuttering", "display",
            &["nvidia-smi 2>/dev/null || echo 'Check GPU drivers'"]),
        // Network slow
        (&["bandwidth", "using"], "bandwidth usage query", "network",
            &["ss -s", "ip -s link"]),
        (&["what", "using", "network"], "network usage query", "network",
            &["ss -tulpn | head -10"]),
        (&["internet", "slow"], "slow internet connection", "network",
            &["ping -c 3 8.8.8.8", "curl -s https://fast.com/api 2>/dev/null | head -1"]),
        (&["download", "slow"], "slow download speed", "network",
            &["ping -c 3 8.8.8.8", "cat /sys/class/net/*/operstate"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.to_lowercase().contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}
