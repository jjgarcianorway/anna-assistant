//! Performance-related suggestion checks.

use chrono::Utc;
use super::types::{Suggestion, SuggestionPriority};

/// Check for boot time degradation
pub async fn check_boot_performance() -> Option<Suggestion> {
    // Get current boot time
    let output = std::process::Command::new("systemd-analyze")
        .output()
        .ok()?;

    let boot_info = String::from_utf8_lossy(&output.stdout);

    // Parse boot time (e.g., "Startup finished in 2.5s (kernel) + 8.2s (userspace) = 10.7s")
    let total_time = boot_info
        .split('=')
        .nth(1)
        .and_then(|s| s.trim().trim_end_matches('s').parse::<f32>().ok())?;

    if total_time > 30.0 {
        Some(Suggestion {
            id: "slow-boot-time".to_string(),
            priority: if total_time > 60.0 {
                SuggestionPriority::High
            } else {
                SuggestionPriority::Medium
            },
            title: format!("Boot time is {:.1} seconds", total_time),
            description: "Your system takes a long time to boot. This could be due to slow services or hardware issues.".to_string(),
            reasoning: "Fast boot times improve system usability and indicate healthy services.".to_string(),
            action: Some("Ask: 'why is my system slow to boot?'".to_string()),
            created_at: Utc::now().to_rfc3339(),
            shown_count: 0,
            dismissed: false,
        })
    } else {
        None
    }
}

/// Check for memory pressure or potential leaks
pub async fn check_memory_usage() -> Option<Suggestion> {
    // Get memory info
    let output = std::process::Command::new("free")
        .args(["-m"])
        .output()
        .ok()?;

    let mem_info = String::from_utf8_lossy(&output.stdout);

    // Parse memory line
    for line in mem_info.lines() {
        if line.starts_with("Mem:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let total: f32 = parts[1].parse().ok()?;
                let used: f32 = parts[2].parse().ok()?;
                let used_pct = (used / total) * 100.0;

                if used_pct > 90.0 {
                    return Some(Suggestion {
                        id: "high-memory-usage".to_string(),
                        priority: SuggestionPriority::High,
                        title: format!("Memory usage at {:.0}%", used_pct),
                        description: "Your system is running low on RAM. This can cause slowdowns and application crashes.".to_string(),
                        reasoning: "High memory usage impacts performance and stability.".to_string(),
                        action: Some("Ask: 'what's using my memory?'".to_string()),
                        created_at: Utc::now().to_rfc3339(),
                        shown_count: 0,
                        dismissed: false,
                    });
                }
            }
        }
    }

    None
}

/// Check for high CPU usage or runaway processes
pub async fn check_cpu_usage() -> Option<Suggestion> {
    // Get load average
    let load_info = std::fs::read_to_string("/proc/loadavg").ok()?;
    let load_parts: Vec<&str> = load_info.split_whitespace().collect();

    let load_1min: f32 = load_parts.first()?.parse().ok()?;

    // Get CPU count from /proc/cpuinfo
    let cpuinfo = std::fs::read_to_string("/proc/cpuinfo").ok()?;
    let cpu_count = cpuinfo.lines().filter(|l| l.starts_with("processor")).count() as f32;

    // Load above CPU count indicates saturation
    if load_1min > cpu_count * 2.0 {
        Some(Suggestion {
            id: "high-cpu-load".to_string(),
            priority: SuggestionPriority::High,
            title: format!("High system load: {:.2}", load_1min),
            description: format!(
                "Load average is {:.2}, but you only have {} CPUs. System is heavily loaded.",
                load_1min, cpu_count
            ),
            reasoning: "High load causes system slowdowns and delays.".to_string(),
            action: Some("Ask: 'what's causing high CPU load?'".to_string()),
            created_at: Utc::now().to_rfc3339(),
            shown_count: 0,
            dismissed: false,
        })
    } else {
        None
    }
}
