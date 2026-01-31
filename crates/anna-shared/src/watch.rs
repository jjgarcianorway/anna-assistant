//! Watch mode - Real-time system monitoring display.
//!
//! v0.3.117: Live updating dashboard similar to htop.

use crate::charts::Gauge;
use crate::live_state::LiveState;
use crate::proactive::{scan_for_issues, IssueSeverity};
use std::io::{self, Write};

/// Configuration for watch mode.
pub struct WatchConfig {
    /// Refresh interval in seconds
    pub interval_secs: u64,
    /// Show compact view
    pub compact: bool,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            interval_secs: 2,
            compact: false,
        }
    }
}

/// Render a single frame of the watch display.
pub fn render_watch_frame(compact: bool) -> String {
    let state = LiveState::capture();
    let timestamp = chrono::Local::now().format("%H:%M:%S");

    if compact {
        render_compact_frame(&state, &timestamp.to_string())
    } else {
        render_full_frame(&state, &timestamp.to_string())
    }
}

/// Render compact watch frame (single line updates).
fn render_compact_frame(state: &LiveState, timestamp: &str) -> String {
    let cpu_pct = (state.load_avg.0 / num_cpus() as f32 * 100.0).min(100.0);
    let mem_pct = state.memory.percent_used();
    let disk_pct = state.disk.percent_used();

    let issues = scan_for_issues();
    let critical = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Critical)).count();
    let warnings = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Warning)).count();

    let status = if critical > 0 {
        "CRIT"
    } else if warnings > 0 {
        "WARN"
    } else if !state.failed_units.is_empty() {
        "DGRD"
    } else {
        " OK "
    };

    format!(
        "[{}] [{}] CPU:{:>3}% MEM:{:>3}% DISK:{:>3}% Load:{:.2} Procs:{} Net:{}",
        timestamp,
        status,
        cpu_pct as u32,
        mem_pct as u32,
        disk_pct as u32,
        state.load_avg.0,
        state.process_count,
        match &state.network_status {
            crate::live_state::NetworkStatus::Connected { .. } => "UP",
            _ => "DOWN",
        }
    )
}

/// Render full watch frame with gauges.
fn render_full_frame(state: &LiveState, timestamp: &str) -> String {
    let mut output = String::new();

    // Clear screen and move cursor to top
    output.push_str("\x1B[2J\x1B[H");

    // Header
    output.push_str("╔══════════════════════════════════════════════════════════════════════╗\n");
    output.push_str(&format!("║  ANNA WATCH MODE                                     {} ║\n", timestamp));
    output.push_str("╚══════════════════════════════════════════════════════════════════════╝\n\n");

    let cpu_pct = (state.load_avg.0 / num_cpus() as f32 * 100.0).min(100.0);
    let mem_pct = state.memory.percent_used();
    let disk_pct = state.disk.percent_used();

    // Status line
    let status_char = if !state.failed_units.is_empty() { "!" } else { "+" };
    output.push_str(&format!(
        "  [{}] Status: {} | Load: {:.2} {:.2} {:.2} | Processes: {}\n\n",
        status_char,
        if state.failed_units.is_empty() { "Healthy" } else { "Degraded" },
        state.load_avg.0, state.load_avg.1, state.load_avg.2,
        state.process_count
    ));

    // Resource gauges
    let cpu_gauge = Gauge::new("CPU", cpu_pct as f64, 100.0);
    let mem_gauge = Gauge::new("Memory", mem_pct as f64, 100.0);
    let disk_gauge = Gauge::new("Disk", disk_pct as f64, 100.0);

    output.push_str(&format!("  {}\n", cpu_gauge.render()));
    output.push_str(&format!("  {}\n", mem_gauge.render()));
    output.push_str(&format!("  {}\n", disk_gauge.render()));

    // Swap if used
    if state.memory.swap_used_gb > 0.1 {
        let swap_pct = state.memory.swap_percent_used();
        let swap_gauge = Gauge::new("Swap", swap_pct as f64, 100.0);
        output.push_str(&format!("  {}\n", swap_gauge.render()));
    }

    output.push('\n');

    // Memory details
    output.push_str(&format!(
        "  Memory: {:.1} GB / {:.1} GB used\n",
        state.memory.used_gb, state.memory.total_gb
    ));
    output.push_str(&format!(
        "  Disk:   {:.1} GB / {:.1} GB used ({})\n",
        state.disk.used_gb, state.disk.total_gb, state.disk.mount_point
    ));

    // Network
    match &state.network_status {
        crate::live_state::NetworkStatus::Connected { interface, ip } => {
            output.push_str(&format!("  Network: {} ({})\n", ip, interface));
        }
        crate::live_state::NetworkStatus::Disconnected => {
            output.push_str("  Network: Disconnected\n");
        }
        crate::live_state::NetworkStatus::Unknown => {
            output.push_str("  Network: Unknown\n");
        }
    }

    output.push('\n');

    // High CPU processes
    if !state.high_cpu_procs.is_empty() {
        output.push_str("  High CPU:\n");
        for proc in state.high_cpu_procs.iter().take(5) {
            output.push_str(&format!("    {}\n", proc));
        }
        output.push('\n');
    }

    // Failed services
    if !state.failed_units.is_empty() {
        output.push_str("  Failed services:\n");
        for unit in state.failed_units.iter().take(5) {
            output.push_str(&format!("    [X] {}\n", unit));
        }
        if state.failed_units.len() > 5 {
            output.push_str(&format!("    ... and {} more\n", state.failed_units.len() - 5));
        }
        output.push('\n');
    }

    // Issues summary
    let issues = scan_for_issues();
    if !issues.is_empty() {
        let critical = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Critical)).count();
        let warnings = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Warning)).count();
        let info = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Info)).count();

        output.push_str(&format!(
            "  Issues: {} critical, {} warnings, {} info\n",
            critical, warnings, info
        ));
    }

    // Footer
    output.push_str("\n  Press Ctrl+C to exit\n");

    output
}

/// Print a watch frame to stdout.
pub fn print_watch_frame(compact: bool) {
    let frame = render_watch_frame(compact);
    print!("{}", frame);
    io::stdout().flush().ok();
}

fn num_cpus() -> usize {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .map(|s| s.matches("processor").count())
        .unwrap_or(1)
        .max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compact_frame() {
        let frame = render_watch_frame(true);
        assert!(frame.contains("CPU:"));
        assert!(frame.contains("MEM:"));
    }

    #[test]
    fn test_full_frame() {
        let frame = render_watch_frame(false);
        assert!(frame.contains("ANNA WATCH MODE"));
        assert!(frame.contains("Memory:"));
    }
}
