//! Health Report - Comprehensive system health visualization.
//!
//! Generates beautiful ASCII reports of system health with charts and trends.

use crate::charts::{BarChart, Gauge, Sparkline, Status, StatusBox, TrendChart, HealthReport};
use crate::live_state::LiveState;
use std::process::Command;

/// Generate a comprehensive health report.
pub fn generate_health_report() -> String {
    let mut report = HealthReport::new();
    let state = LiveState::capture();

    // Header
    report.add_section(format!(
        "╔══════════════════════════════════════════════════════════════╗\n\
         ║              ANNA SYSTEM HEALTH REPORT                       ║\n\
         ║              {}                              ║\n\
         ╚══════════════════════════════════════════════════════════════╝",
        chrono::Local::now().format("%Y-%m-%d %H:%M")
    ));

    // Quick status overview
    report.add_section(generate_status_overview(&state));

    // Resource gauges
    report.add_section(generate_resource_gauges(&state));

    // Disk usage chart
    report.add_section(generate_disk_chart());

    // Top processes
    report.add_section(generate_top_processes());

    // Service status
    report.add_section(generate_service_status(&state));

    // System trends (if history available)
    report.add_section(generate_system_trends());

    // Recommendations
    report.add_section(generate_recommendations(&state));

    report.render()
}

fn generate_status_overview(state: &LiveState) -> String {
    let mut status = StatusBox::new("Quick Status");

    // CPU status
    let cpu_status = if state.load_avg.0 > 4.0 {
        Status::Critical
    } else if state.load_avg.0 > 2.0 {
        Status::Warning
    } else {
        Status::Good
    };
    status.add("CPU Load", &format!("{:.2}", state.load_avg.0), cpu_status);

    // Memory status
    let mem_pct = state.memory.percent_used();
    let mem_status = if mem_pct > 90.0 {
        Status::Critical
    } else if mem_pct > 80.0 {
        Status::Warning
    } else {
        Status::Good
    };
    status.add("Memory", &format!("{:.0}%", mem_pct), mem_status);

    // Disk status
    let disk_pct = state.disk.percent_used();
    let disk_status = if disk_pct > 90.0 {
        Status::Critical
    } else if disk_pct > 80.0 {
        Status::Warning
    } else {
        Status::Good
    };
    status.add("Disk", &format!("{:.0}%", disk_pct), disk_status);

    // Network status
    let net_status = match &state.network_status {
        crate::live_state::NetworkStatus::Connected { .. } => Status::Good,
        crate::live_state::NetworkStatus::Disconnected => Status::Critical,
        crate::live_state::NetworkStatus::Unknown => Status::Unknown,
    };
    let net_label = match &state.network_status {
        crate::live_state::NetworkStatus::Connected { interface, .. } => interface.clone(),
        _ => "disconnected".to_string(),
    };
    status.add("Network", &net_label, net_status);

    // Failed services
    let svc_status = if state.failed_units.is_empty() {
        Status::Good
    } else {
        Status::Warning
    };
    status.add("Services", &format!("{} failed", state.failed_units.len()), svc_status);

    status.render()
}

fn generate_resource_gauges(state: &LiveState) -> String {
    let mut lines = vec!["Resource Utilization".to_string(), "───────────────────".to_string()];

    // CPU gauge (based on load average vs cores)
    let cores = num_cpus();
    let cpu_pct = (state.load_avg.0 / cores as f32 * 100.0).min(100.0);
    lines.push(Gauge::percentage("CPU", cpu_pct as f64).render());

    // Memory gauge
    lines.push(Gauge::new("Memory", state.memory.used_gb as f64, state.memory.total_gb as f64).render());

    // Swap gauge (if used)
    if state.memory.swap_total_gb > 0.0 {
        lines.push(Gauge::new("Swap", state.memory.swap_used_gb as f64, state.memory.swap_total_gb as f64).render());
    }

    // Disk gauge
    lines.push(Gauge::new("Disk /", state.disk.used_gb as f64, state.disk.total_gb as f64).render());

    lines.join("\n")
}

fn generate_disk_chart() -> String {
    let mut chart = BarChart::new("Disk Usage by Partition");

    if let Ok(output) = Command::new("df")
        .args(["-h", "--output=target,pcent"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let mount = parts[0];
                let pct_str = parts[1].trim_end_matches('%');
                if let Ok(pct) = pct_str.parse::<f64>() {
                    // Skip virtual filesystems
                    if mount.starts_with("/dev") || mount == "/" || mount.starts_with("/home") || mount.starts_with("/boot") {
                        continue; // These are mount points, not sources
                    }
                    if mount.starts_with("/run") || mount.starts_with("/sys") || mount.starts_with("/proc") {
                        continue;
                    }
                    let color = if pct >= 90.0 {
                        "critical"
                    } else if pct >= 75.0 {
                        "warning"
                    } else {
                        "good"
                    };
                    chart.add_colored(mount, pct, color);
                }
            }
        }
    }

    // Fallback: at least show root
    if chart.bars.is_empty() {
        if let Ok(output) = Command::new("df").args(["--output=pcent", "/"]).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = stdout.lines().nth(1) {
                if let Ok(pct) = line.trim().trim_end_matches('%').parse::<f64>() {
                    chart.add("/", pct);
                }
            }
        }
    }

    chart.render()
}

fn generate_top_processes() -> String {
    let mut lines = vec!["Top Processes by CPU".to_string(), "────────────────────".to_string()];

    if let Ok(output) = Command::new("ps")
        .args(["--no-headers", "-eo", "pcpu,pmem,comm", "--sort=-pcpu"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        lines.push(format!("{:>6} {:>6}  {}", "CPU%", "MEM%", "Process"));
        lines.push("─".repeat(30));

        for line in stdout.lines().take(5) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let cpu = parts[0];
                let mem = parts[1];
                let proc = parts[2];
                lines.push(format!("{:>6} {:>6}  {}", cpu, mem, proc));
            }
        }
    }

    lines.join("\n")
}

fn generate_service_status(state: &LiveState) -> String {
    let mut status = StatusBox::new("Service Health");

    // Count running services
    let running = Command::new("systemctl")
        .args(["list-units", "--type=service", "--state=running", "--no-legend", "--no-pager"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
        .unwrap_or(0);

    status.add("Running", &format!("{} services", running), Status::Good);
    status.add("Failed", &format!("{} services", state.failed_units.len()),
        if state.failed_units.is_empty() { Status::Good } else { Status::Warning });

    // Show failed service names
    for unit in state.failed_units.iter().take(3) {
        status.add(&format!("  └ {}", unit), "failed", Status::Critical);
    }

    status.render()
}

fn generate_system_trends() -> String {
    let mut lines = vec!["System Trends (Last Hour)".to_string(), "─────────────────────────".to_string()];

    // Load average history from /proc/loadavg doesn't give history
    // But we can show the 1/5/15 minute loads as a simple trend indicator
    if let Ok(content) = std::fs::read_to_string("/proc/loadavg") {
        let parts: Vec<f64> = content
            .split_whitespace()
            .take(3)
            .filter_map(|s| s.parse().ok())
            .collect();

        if parts.len() == 3 {
            let spark = Sparkline::new(&parts);
            lines.push(format!("Load (1/5/15m): {} ({:.2} → {:.2} → {:.2})",
                spark.render(), parts[0], parts[1], parts[2]));

            // Trend indicator
            let trend = if parts[0] > parts[2] * 1.2 {
                "↗ increasing"
            } else if parts[0] < parts[2] * 0.8 {
                "↘ decreasing"
            } else {
                "→ stable"
            };
            lines.push(format!("Trend: {}", trend));
        }
    }

    lines.join("\n")
}

fn generate_recommendations(state: &LiveState) -> String {
    let mut recommendations = Vec::new();

    // Memory recommendations
    let mem_pct = state.memory.percent_used();
    if mem_pct > 90.0 {
        recommendations.push("⚠ CRITICAL: Memory usage over 90%. Consider closing applications or adding swap.");
    } else if mem_pct > 80.0 {
        recommendations.push("⚡ Memory usage high. Monitor for memory-hungry processes.");
    }

    // Disk recommendations
    let disk_pct = state.disk.percent_used();
    if disk_pct > 90.0 {
        recommendations.push("⚠ CRITICAL: Disk usage over 90%. Run 'paccache -rk1' and clean logs.");
    } else if disk_pct > 80.0 {
        recommendations.push("⚡ Disk usage elevated. Consider cleanup soon.");
    }

    // Load recommendations
    let cores = num_cpus();
    if state.load_avg.0 > cores as f32 * 2.0 {
        recommendations.push("⚠ Very high CPU load. Check for runaway processes.");
    } else if state.load_avg.0 > cores as f32 {
        recommendations.push("⚡ CPU load above core count. System may feel sluggish.");
    }

    // Failed services
    if !state.failed_units.is_empty() {
        recommendations.push("⚡ Some services have failed. Run 'systemctl --failed' for details.");
    }

    // Swap usage
    if state.memory.swap_percent_used() > 50.0 {
        recommendations.push("⚡ High swap usage indicates memory pressure.");
    }

    if recommendations.is_empty() {
        recommendations.push("✓ System looks healthy. No immediate actions needed.");
    }

    let mut lines = vec!["Recommendations".to_string(), "───────────────".to_string()];
    lines.extend(recommendations.iter().map(|s| s.to_string()));
    lines.join("\n")
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(1)
}

/// Generate a compact health summary (one line).
pub fn health_summary() -> String {
    let state = LiveState::capture();

    let cpu_indicator = if state.load_avg.0 > 2.0 { "▓" } else { "░" };
    let mem_indicator = if state.memory.percent_used() > 80.0 { "▓" } else { "░" };
    let disk_indicator = if state.disk.percent_used() > 80.0 { "▓" } else { "░" };
    let svc_indicator = if state.failed_units.is_empty() { "░" } else { "▓" };

    format!(
        "Health: CPU[{}] MEM[{}] DISK[{}] SVC[{}] | Load: {:.1} | Mem: {:.0}% | Disk: {:.0}%",
        cpu_indicator, mem_indicator, disk_indicator, svc_indicator,
        state.load_avg.0, state.memory.percent_used(), state.disk.percent_used()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_report_generation() {
        let report = generate_health_report();
        assert!(report.contains("ANNA SYSTEM HEALTH REPORT"));
        assert!(report.contains("Quick Status"));
    }

    #[test]
    fn test_health_summary() {
        let summary = health_summary();
        assert!(summary.contains("Health:"));
        assert!(summary.contains("Load:"));
    }
}
