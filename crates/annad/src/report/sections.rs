//! Report section generators.

use chrono::{DateTime, Local};

use crate::anomaly::AnomalyStore;
use crate::update_system::check_updates;

use super::user::ReportPreferences;

/// Generate personalized greeting
pub fn generate_greeting(prefs: &ReportPreferences, now: &DateTime<Local>) -> String {
    let greetings = [
        "Good morning! Here's your system health report.",
        "Morning! I've prepared your daily system overview.",
        "Hello! Your daily system report is ready.",
        "Hi! Here's what's happening with your system.",
        "Good morning! Everything you need to know about your system today.",
    ];

    let idx = (now.timestamp() as usize / 86400) % greetings.len();
    let base = greetings[idx];

    if let Some(ref name) = prefs.user_name {
        base.replace("!", &format!(", {}!", name))
    } else {
        base.to_string()
    }
}

/// Generate executive summary of system health
pub fn generate_executive_summary() -> String {
    let mut issues = Vec::new();
    let mut positives = Vec::new();

    // Check disk
    if let Ok(output) = std::process::Command::new("df")
        .args(["--output=pcent,avail", "-BG", "/"])
        .output()
    {
        let out = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = out.lines().nth(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let pct: u32 = parts[0].trim_end_matches('%').parse().unwrap_or(0);
                let avail = parts[1].trim_end_matches('G');
                if pct > 85 {
                    issues.push(format!("Disk usage is high at {}% ({} GB free)", pct, avail));
                } else {
                    positives.push(format!("Disk space is healthy with {} GB free", avail));
                }
            }
        }
    }

    // Check updates
    let updates = check_updates();
    let security: Vec<_> = updates.iter().filter(|u| u.is_security).collect();
    if !security.is_empty() {
        issues.push(format!("{} security updates pending", security.len()));
    } else if !updates.is_empty() {
        positives.push(format!("{} regular updates available", updates.len()));
    } else {
        positives.push("System is fully up to date".to_string());
    }

    // Check services
    if let Ok(output) = std::process::Command::new("systemctl")
        .args(["--failed", "--no-pager", "--no-legend"])
        .output()
    {
        let count = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|l| !l.trim().is_empty())
            .count();
        if count > 0 {
            issues.push(format!("{} service(s) in failed state", count));
        } else {
            positives.push("All services running normally".to_string());
        }
    }

    if issues.is_empty() {
        format!("Your system is in excellent health. {}", positives.join(". "))
    } else if issues.len() == 1 {
        format!("Overall healthy, but {}. {}", issues[0].to_lowercase(), positives.join(". "))
    } else {
        format!("A few items need attention: {}. On the positive side: {}",
            issues.join("; "), positives.join(". "))
    }
}

/// Generate metrics trends summary
pub fn generate_metrics_summary() -> String {
    let store = AnomalyStore::load();
    let mut summary = Vec::new();

    // RAM trends
    if let Some(history) = store.metrics.get("RAM") {
        if let Some(baseline) = &history.baseline {
            let current = history.samples.last().map(|s| s.value).unwrap_or(0.0);
            let trend = if current > baseline.mean + baseline.std_dev {
                "higher than usual"
            } else if current < baseline.mean - baseline.std_dev {
                "lower than usual"
            } else {
                "within normal range"
            };
            summary.push(format!("Memory usage: {:.1}% ({})", current, trend));
        }
    }

    // Load trends
    if let Some(history) = store.metrics.get("Load1") {
        if let Some(baseline) = &history.baseline {
            let current = history.samples.last().map(|s| s.value).unwrap_or(0.0);
            let trend = if current > baseline.mean * 1.5 {
                "elevated"
            } else {
                "normal"
            };
            summary.push(format!("System load: {:.2} ({})", current, trend));
        }
    }

    // Disk trends
    if let Some(history) = store.metrics.get("Disk") {
        if let Some(baseline) = &history.baseline {
            let current = history.samples.last().map(|s| s.value).unwrap_or(0.0);
            if current > 85.0 {
                summary.push(format!("Disk usage: {:.1}% - attention needed", current));
            } else {
                summary.push(format!("Disk usage: {:.1}% - healthy", current));
            }
        }
    }

    if summary.is_empty() {
        "Metrics collection in progress. Full trends available after 24 hours.".to_string()
    } else {
        summary.join("\n")
    }
}

/// Generate current system status section
pub fn generate_status_section() -> String {
    let mut lines = Vec::new();

    // Uptime
    if let Ok(uptime) = std::fs::read_to_string("/proc/uptime") {
        if let Some(secs) = uptime.split_whitespace().next() {
            if let Ok(s) = secs.parse::<f64>() {
                let days = (s / 86400.0) as u32;
                let hours = ((s % 86400.0) / 3600.0) as u32;
                lines.push(format!("Uptime: {} days, {} hours", days, hours));
            }
        }
    }

    // Load
    if let Ok(load) = std::fs::read_to_string("/proc/loadavg") {
        let parts: Vec<&str> = load.split_whitespace().collect();
        if parts.len() >= 3 {
            let load1: f32 = parts[0].parse().unwrap_or(0.0);
            if load1 > 4.0 {
                lines.push(format!("Load average: {} / {} / {} - elevated",
                    parts[0], parts[1], parts[2]));
            } else {
                lines.push(format!("Load average: {} / {} / {} - normal",
                    parts[0], parts[1], parts[2]));
            }
        }
    }

    // Memory
    if let Ok(output) = std::process::Command::new("free").args(["-h"]).output() {
        let out = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = out.lines().nth(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                lines.push(format!("Memory: {} used of {} total", parts[2], parts[1]));
            }
        }
    }

    // Disk
    if let Ok(output) = std::process::Command::new("df").args(["-h", "/"]).output() {
        let out = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = out.lines().nth(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                lines.push(format!("Disk: {} used ({} available)", parts[4], parts[3]));
            }
        }
    }

    lines.join("\n")
}

/// Generate software updates section
pub fn generate_updates_section() -> String {
    let updates = check_updates();

    if updates.is_empty() {
        return "Your system is fully up to date. No pending updates.".to_string();
    }

    let security: Vec<_> = updates.iter().filter(|u| u.is_security).collect();
    let kernel: Vec<_> = updates.iter().filter(|u| u.is_kernel).collect();
    let regular_count = updates.len() - security.len() - kernel.len();

    let mut parts = Vec::new();

    if !security.is_empty() {
        let names: Vec<_> = security.iter().take(3).map(|u| u.name.as_str()).collect();
        parts.push(format!("{} security update(s) including: {}",
            security.len(), names.join(", ")));
    }

    if !kernel.is_empty() {
        parts.push(format!("{} kernel update(s) - reboot will be required", kernel.len()));
    }

    if regular_count > 0 {
        parts.push(format!("{} regular package update(s)", regular_count));
    }

    format!("{} total updates available. {}", updates.len(), parts.join(". "))
}

/// Generate recommendations based on system analysis
pub fn generate_recommendations() -> Vec<String> {
    let mut recs = Vec::new();
    let suggestions = crate::anomaly::check_optimizations();

    for s in suggestions.iter().take(3) {
        let rec = if let Some(ref savings) = s.potential_savings {
            format!("{}: {} (could save {})", s.category, s.description, savings)
        } else {
            format!("{}: {}", s.category, s.description)
        };
        recs.push(rec);
    }

    // Check if reboot needed
    if crate::update_system::needs_reboot() {
        recs.push("A reboot is recommended - your kernel was updated".to_string());
    }

    recs
}

/// Generate automated maintenance description
pub fn generate_healing_section() -> String {
    "Anna automatically maintains your system by restarting failed services, \
     clearing disk space when low, and removing stale locks. All maintenance \
     actions are logged and require no manual intervention.".to_string()
}

/// Generate closing message
pub fn generate_closing() -> String {
    let closings = [
        "That's all for today. Have a productive day!",
        "Report complete. I'm here if you need anything.",
        "Everything's covered. Reach out if questions come up.",
        "That's the overview. Let me know if you need more details.",
    ];

    let now = Local::now();
    let idx = (now.timestamp() as usize / 3600) % closings.len();
    closings[idx].to_string()
}
