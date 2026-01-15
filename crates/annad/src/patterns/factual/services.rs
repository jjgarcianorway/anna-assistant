//! Service, package, thermal, and process patterns.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};
use super::super::contains_word;
use super::FactualPattern;

pub fn match_packages(q: &str) -> Option<DeepUnderstanding> {
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

pub fn match_services(q: &str) -> Option<DeepUnderstanding> {
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
pub fn match_thermal(q: &str) -> Option<DeepUnderstanding> {
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
pub fn match_processes(q: &str) -> Option<DeepUnderstanding> {
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
