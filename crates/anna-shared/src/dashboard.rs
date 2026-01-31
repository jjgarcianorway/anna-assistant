//! Unified system dashboard combining health, issues, and predictions.
//!
//! v0.3.116: Comprehensive visual dashboard for system monitoring.

use crate::charts::{Gauge, Sparkline};
use crate::live_state::LiveState;
use crate::monitor::LongTermHistory;
use crate::prediction::{Forecaster, TrendDirection};
use crate::proactive::{scan_for_issues, IssueSeverity};

/// Generate comprehensive system dashboard.
pub fn generate_dashboard() -> String {
    let mut output = String::new();

    // Header
    output.push_str("\n");
    output.push_str("╔══════════════════════════════════════════════════════════════════════════╗\n");
    output.push_str("║                         ANNA SYSTEM DASHBOARD                            ║\n");
    output.push_str("╚══════════════════════════════════════════════════════════════════════════╝\n");
    output.push_str("\n");

    // Get live state
    let state = LiveState::capture();

    // === CURRENT STATUS ===
    output.push_str("┌─ CURRENT STATUS ──────────────────────────────────────────────────────────┐\n");
    output.push_str(&render_status_line(&state));
    output.push_str("└───────────────────────────────────────────────────────────────────────────┘\n\n");

    // === RESOURCE GAUGES ===
    output.push_str("┌─ RESOURCES ───────────────────────────────────────────────────────────────┐\n");
    output.push_str(&render_gauges(&state));
    output.push_str("└───────────────────────────────────────────────────────────────────────────┘\n\n");

    // === PREDICTIONS ===
    output.push_str("┌─ PREDICTIONS ─────────────────────────────────────────────────────────────┐\n");
    output.push_str(&render_predictions());
    output.push_str("└───────────────────────────────────────────────────────────────────────────┘\n\n");

    // === ISSUES ===
    let issues = scan_for_issues();
    if !issues.is_empty() {
        output.push_str("┌─ DETECTED ISSUES ──────────────────────────────────────────────────────────┐\n");
        output.push_str(&render_issues_compact(&issues));
        output.push_str("└───────────────────────────────────────────────────────────────────────────┘\n\n");
    }

    // === TRENDS ===
    output.push_str("┌─ TRENDS (7 DAY) ──────────────────────────────────────────────────────────┐\n");
    output.push_str(&render_trends());
    output.push_str("└───────────────────────────────────────────────────────────────────────────┘\n\n");

    // === SERVICES ===
    if !state.failed_units.is_empty() {
        output.push_str("┌─ FAILED SERVICES ─────────────────────────────────────────────────────────┐\n");
        for unit in state.failed_units.iter().take(5) {
            output.push_str(&format!("│  [X] {:<66}│\n", unit));
        }
        if state.failed_units.len() > 5 {
            output.push_str(&format!("│  ... and {} more{:>55}│\n", state.failed_units.len() - 5, ""));
        }
        output.push_str("└───────────────────────────────────────────────────────────────────────────┘\n\n");
    }

    // Footer
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    output.push_str(&format!("  Generated: {} | Run 'annactl issues' for details\n\n", timestamp));

    output
}

/// Render compact status line with indicators.
fn render_status_line(state: &LiveState) -> String {
    let cpu_pct = (state.load_avg.0 / num_cpus() as f32 * 100.0).min(100.0);
    let mem_pct = state.memory.percent_used();
    let disk_pct = state.disk.percent_used();

    let cpu_indicator = status_indicator(cpu_pct);
    let mem_indicator = status_indicator(mem_pct);
    let disk_indicator = status_indicator(disk_pct);
    let svc_indicator = if state.failed_units.is_empty() { "OK" } else { "!!" };

    let uptime = format_uptime(state.uptime_hours);

    format!(
        "│  CPU [{:>3}%] {}  MEM [{:>3}%] {}  DISK [{:>3}%] {}  SVC [{}]  UP: {:>10} │\n",
        cpu_pct as u32, cpu_indicator,
        mem_pct as u32, mem_indicator,
        disk_pct as u32, disk_indicator,
        svc_indicator,
        uptime
    )
}

/// Render resource gauges.
fn render_gauges(state: &LiveState) -> String {
    let mut output = String::new();

    let cpu_pct = (state.load_avg.0 / num_cpus() as f32 * 100.0).min(100.0);
    let cpu_gauge = Gauge::new("CPU", cpu_pct as f64, 100.0);
    let mem_gauge = Gauge::new("Memory", state.memory.percent_used() as f64, 100.0);
    let disk_gauge = Gauge::new("Disk", state.disk.percent_used() as f64, 100.0);

    output.push_str(&format!("│  {}  │\n", cpu_gauge.render().trim()));
    output.push_str(&format!("│  {}  │\n", mem_gauge.render().trim()));
    output.push_str(&format!("│  {}  │\n", disk_gauge.render().trim()));

    // Add swap if significant
    if state.memory.swap_used_gb > 0.0 {
        let swap_pct = if state.memory.swap_total_gb > 0.0 {
            (state.memory.swap_used_gb / state.memory.swap_total_gb * 100.0) as f64
        } else {
            0.0
        };
        let swap_gauge = Gauge::new("Swap", swap_pct, 100.0);
        output.push_str(&format!("│  {}  │\n", swap_gauge.render().trim()));
    }

    output
}

/// Render predictions based on historical data.
fn render_predictions() -> String {
    let mut output = String::new();
    let history = LongTermHistory::load();

    if history.daily_snapshots.len() < 3 {
        output.push_str("│  Collecting data... (need 3+ days for predictions)                       │\n");
        return output;
    }

    let forecaster = Forecaster::default();

    // Disk prediction
    let disk_values: Vec<f64> = history.daily_snapshots.iter()
        .map(|s| {
            // Convert disk_used_gb to percentage (estimate)
            let total_gb = get_total_disk_gb();
            if total_gb > 0.0 {
                (s.disk_used_gb as f64 / total_gb) * 100.0
            } else {
                0.0
            }
        })
        .collect();

    if !disk_values.is_empty() {
        let disk_forecast = forecaster.forecast_disk(&disk_values);
        let trend_symbol = match disk_forecast.trend.as_ref().map(|t| &t.direction) {
            Some(TrendDirection::Increasing) => "^",
            Some(TrendDirection::Decreasing) => "v",
            _ => "-",
        };

        let prediction = if let Some(days) = disk_forecast.days_until_critical {
            if days < 30.0 {
                format!("Will reach 95% in {:.0} days", days)
            } else {
                "Stable".to_string()
            }
        } else if let Some(days) = disk_forecast.days_until_warning {
            if days < 30.0 {
                format!("Will reach 85% in {:.0} days", days)
            } else {
                "Stable".to_string()
            }
        } else {
            "Stable".to_string()
        };

        output.push_str(&format!("│  Disk:   [{}] {:60}│\n", trend_symbol, prediction));
    }

    // Memory trend
    let mem_values: Vec<f64> = history.daily_snapshots.iter()
        .map(|s| s.avg_memory_pct as f64)
        .collect();

    if !mem_values.is_empty() {
        let mem_forecast = forecaster.forecast_memory(&mem_values);
        let trend_symbol = match mem_forecast.trend.as_ref().map(|t| &t.direction) {
            Some(TrendDirection::Increasing) => "^",
            Some(TrendDirection::Decreasing) => "v",
            _ => "-",
        };

        let is_leak = mem_forecast.trend.as_ref()
            .map(|t| t.direction == TrendDirection::Increasing && t.slope > 1.0 && t.r_squared > 0.7)
            .unwrap_or(false);

        let prediction = if is_leak {
            "POSSIBLE MEMORY LEAK - usage consistently increasing"
        } else {
            "Normal"
        };

        output.push_str(&format!("│  Memory: [{}] {:60}│\n", trend_symbol, prediction));
    }

    // Boot time trend
    let boot_values: Vec<f64> = history.daily_snapshots.iter()
        .map(|s| s.avg_boot_time as f64)
        .collect();

    if !boot_values.is_empty() {
        let boot_forecast = forecaster.forecast_boot_time(&boot_values);
        let trend_symbol = match boot_forecast.trend.as_ref().map(|t| &t.direction) {
            Some(TrendDirection::Increasing) => "^",
            Some(TrendDirection::Decreasing) => "v",
            _ => "-",
        };

        let is_degrading = boot_forecast.trend.as_ref()
            .map(|t| t.direction == TrendDirection::Increasing && t.slope > 0.5)
            .unwrap_or(false);

        let prediction = if is_degrading {
            format!("Degrading (+{:.1}s per boot) - check startup services", boot_forecast.trend.as_ref().map(|t| t.slope).unwrap_or(0.0))
        } else {
            "Stable".to_string()
        };

        output.push_str(&format!("│  Boot:   [{}] {:60}│\n", trend_symbol, prediction));
    }

    output
}

/// Render issues in compact format.
fn render_issues_compact(issues: &[crate::proactive::DetectedIssue]) -> String {
    let mut output = String::new();

    let critical: Vec<_> = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Critical)).collect();
    let warning: Vec<_> = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Warning)).collect();
    let info: Vec<_> = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Info)).collect();

    // Critical issues first
    for issue in critical.iter().take(3) {
        output.push_str(&format!("│  [X] {:66}│\n", truncate(&issue.title, 66)));
    }

    // Then warnings
    for issue in warning.iter().take(3) {
        output.push_str(&format!("│  [!] {:66}│\n", truncate(&issue.title, 66)));
    }

    // Info count only
    if !info.is_empty() {
        output.push_str(&format!("│  [i] {} informational items{:>46}│\n", info.len(), ""));
    }

    if issues.len() > 6 {
        output.push_str(&format!("│      ... {} total issues. Run 'annactl issues' for details{:>14}│\n", issues.len(), ""));
    }

    output
}

/// Render historical trends as sparklines.
fn render_trends() -> String {
    let mut output = String::new();
    let history = LongTermHistory::load();

    if history.daily_snapshots.len() < 2 {
        output.push_str("│  Collecting data... (need 2+ days for trends)                            │\n");
        return output;
    }

    // Take last 14 days
    let recent: Vec<_> = history.daily_snapshots.iter().rev().take(14).collect();
    let recent: Vec<_> = recent.into_iter().rev().collect();

    // Memory sparkline
    let mem_values: Vec<f64> = recent.iter().map(|s| s.avg_memory_pct as f64).collect();
    if !mem_values.is_empty() {
        let sparkline = Sparkline::new(&mem_values);
        output.push_str(&format!("│  Memory:  {} {:>48}│\n", sparkline.render(), format_range(&mem_values)));
    }

    // Load sparkline
    let load_values: Vec<f64> = recent.iter().map(|s| s.avg_load as f64).collect();
    if !load_values.is_empty() {
        let sparkline = Sparkline::new(&load_values);
        output.push_str(&format!("│  Load:    {} {:>48}│\n", sparkline.render(), format_range(&load_values)));
    }

    // Boot time sparkline
    let boot_values: Vec<f64> = recent.iter().map(|s| s.avg_boot_time as f64).collect();
    if !boot_values.is_empty() {
        let sparkline = Sparkline::new(&boot_values);
        output.push_str(&format!("│  Boot:    {} {:>48}│\n", sparkline.render(), format_range_secs(&boot_values)));
    }

    // Packages sparkline
    let pkg_values: Vec<f64> = recent.iter().map(|s| s.packages_installed as f64).collect();
    if !pkg_values.is_empty() {
        let sparkline = Sparkline::new(&pkg_values);
        output.push_str(&format!("│  Packages:{} {:>48}│\n", sparkline.render(), format_range_int(&pkg_values)));
    }

    output
}

/// Generate one-line dashboard summary.
pub fn dashboard_summary() -> String {
    let state = LiveState::capture();
    let issues = scan_for_issues();

    let cpu_pct = (state.load_avg.0 / num_cpus() as f32 * 100.0).min(100.0);
    let mem_pct = state.memory.percent_used();
    let disk_pct = state.disk.percent_used();
    let critical = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Critical)).count();
    let warnings = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Warning)).count();

    let status = if critical > 0 {
        "CRITICAL"
    } else if warnings > 0 {
        "WARNING"
    } else if !state.failed_units.is_empty() {
        "DEGRADED"
    } else {
        "HEALTHY"
    };

    format!(
        "[{}] CPU:{:>3}% MEM:{:>3}% DISK:{:>3}% | {} critical, {} warnings | Up: {}",
        status,
        cpu_pct as u32,
        mem_pct as u32,
        disk_pct as u32,
        critical,
        warnings,
        format_uptime(state.uptime_hours)
    )
}

// === Helper functions ===

fn status_indicator(pct: f32) -> &'static str {
    if pct >= 90.0 { "!!" }
    else if pct >= 75.0 { "!" }
    else { "OK" }
}

fn format_uptime(hours: f32) -> String {
    if hours < 1.0 {
        format!("{}m", (hours * 60.0) as u32)
    } else if hours < 24.0 {
        format!("{:.1}h", hours)
    } else {
        format!("{:.1}d", hours / 24.0)
    }
}

fn num_cpus() -> usize {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .map(|s| s.matches("processor").count())
        .unwrap_or(1)
        .max(1)
}

fn get_total_disk_gb() -> f64 {
    std::process::Command::new("df")
        .args(["--output=size", "-BG", "/"])
        .output()
        .ok()
        .and_then(|o| {
            let out = String::from_utf8_lossy(&o.stdout);
            out.lines().nth(1)
                .and_then(|l| l.trim().trim_end_matches('G').parse::<f64>().ok())
        })
        .unwrap_or(100.0)
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max - 3])
    } else {
        s.to_string()
    }
}

fn format_range(values: &[f64]) -> String {
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    format!("{:.0}-{:.0}%", min, max)
}

fn format_range_secs(values: &[f64]) -> String {
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    format!("{:.1}-{:.1}s", min, max)
}

fn format_range_int(values: &[f64]) -> String {
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min) as i64;
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max) as i64;
    format!("{}-{}", min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_generates() {
        let dashboard = generate_dashboard();
        assert!(dashboard.contains("ANNA SYSTEM DASHBOARD"));
        assert!(dashboard.contains("CURRENT STATUS"));
        assert!(dashboard.contains("RESOURCES"));
    }

    #[test]
    fn test_summary() {
        let summary = dashboard_summary();
        assert!(summary.contains("CPU:"));
        assert!(summary.contains("MEM:"));
        assert!(summary.contains("DISK:"));
    }
}
