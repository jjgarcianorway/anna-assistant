//! Profiled commands for Anna v0.14.0 "Orion III" Phase 2.2
//!
//! CLI interface for continuous performance profiling

use anyhow::Result;
use crate::profiled::{Profiler, DegradationLevel};

/// Profiled command mode
pub enum ProfiledMode {
    Status,
    Summary,
    Rebuild,
}

/// Run profiled command
pub fn run_profiled(mode: ProfiledMode, json: bool) -> Result<()> {
    let profiler = Profiler::new()?;

    match mode {
        ProfiledMode::Status => show_status(&profiler, json),
        ProfiledMode::Summary => show_summary(&profiler, json),
        ProfiledMode::Rebuild => rebuild_baseline(&profiler, json),
    }
}

/// Show current profiler status
fn show_status(profiler: &Profiler, json: bool) -> Result<()> {
    // Capture current snapshot
    let entry = profiler.tick()?;

    if json {
        println!("{}", serde_json::to_string_pretty(&entry)?);
        return Ok(());
    }

    let dim = "\x1b[2m";
    let reset = "\x1b[0m";
    let bold = "\x1b[1m";
    let green = "\x1b[32m";
    let yellow = "\x1b[33m";
    let red = "\x1b[31m";

    println!();
    println!("{}╭─ Performance Profiler Status ────────────────────────────{}", dim, reset);
    println!("{}│{}", dim, reset);
    println!("{}│{}  {}Overall Status:{} {} {}{}",
        dim, reset, bold, reset,
        entry.degradation.emoji(),
        entry.degradation.name(),
        reset
    );
    println!("{}│{}", dim, reset);

    // Current metrics
    println!("{}│{}  {}Current Metrics{}", dim, reset, bold, reset);
    println!("{}│{}    RPC Latency:    {:.2} ms {}({}){}",
        dim, reset, entry.snapshot.rpc_latency_ms,
        delta_color(entry.rpc_delta_pct),
        format_delta(entry.rpc_delta_pct),
        reset
    );
    println!("{}│{}    Memory Usage:   {:.1} MB {}({}){}",
        dim, reset, entry.snapshot.memory_mb,
        delta_color(entry.memory_delta_pct),
        format_delta(entry.memory_delta_pct),
        reset
    );
    println!("{}│{}    I/O Latency:    {:.2} ms {}({}){}",
        dim, reset, entry.snapshot.io_latency_ms,
        delta_color(entry.io_delta_pct),
        format_delta(entry.io_delta_pct),
        reset
    );
    println!("{}│{}    CPU Usage:      {:.1}% {}({}){}",
        dim, reset, entry.snapshot.cpu_percent,
        delta_color(entry.cpu_delta_pct),
        format_delta(entry.cpu_delta_pct),
        reset
    );
    println!("{}│{}", dim, reset);

    // Baseline
    println!("{}│{}  {}7-Day Baseline{}", dim, reset, dim, reset);
    println!("{}│{}    RPC Latency:    {:.2} ms",
        dim, reset, entry.baseline.avg_rpc_latency_ms);
    println!("{}│{}    Memory Usage:   {:.1} MB",
        dim, reset, entry.baseline.avg_memory_mb);
    println!("{}│{}    I/O Latency:    {:.2} ms",
        dim, reset, entry.baseline.avg_io_latency_ms);
    println!("{}│{}    CPU Usage:      {:.1}%",
        dim, reset, entry.baseline.avg_cpu_percent);
    println!("{}│{}    {}Samples:{} {}",
        dim, reset, bold, reset, entry.baseline.sample_count);
    println!("{}│{}", dim, reset);

    // Check for persistent degradation
    if let Some(persistent) = profiler.detect_persistent_degradation()? {
        println!("{}│{}  {}⚠️  Persistent Degradation Detected{}", dim, reset, red, reset);
        println!("{}│{}  Level: {} {}", dim, reset, persistent.emoji(), persistent.name());
        println!("{}│{}  3+ consecutive degraded snapshots", dim, reset);
        println!("{}│{}", dim, reset);
    }

    println!("{}╰──────────────────────────────────────────────────────────{}", dim, reset);
    println!();

    Ok(())
}

/// Show profiler summary
fn show_summary(profiler: &Profiler, json: bool) -> Result<()> {
    let summary = profiler.get_summary()?;

    if json {
        println!("{}", serde_json::to_string_pretty(&summary)?);
        return Ok(());
    }

    let dim = "\x1b[2m";
    let reset = "\x1b[0m";
    let bold = "\x1b[1m";
    let green = "\x1b[32m";
    let yellow = "\x1b[33m";

    println!();
    println!("{}╭─ Performance Profiler Summary ───────────────────────────{}", dim, reset);
    println!("{}│{}", dim, reset);
    println!("{}│{}  {}Monitoring Statistics{}", dim, reset, bold, reset);
    println!("{}│{}    Total Snapshots:   {}", dim, reset, summary.total_entries);
    println!("{}│{}    Degraded Count:    {}{}{}{} {}({:.1}%){}",
        dim, reset,
        if summary.degraded_count > 0 { yellow } else { green },
        summary.degraded_count,
        reset,
        reset,
        dim,
        if summary.total_entries > 0 {
            (summary.degraded_count as f32 / summary.total_entries as f32) * 100.0
        } else {
            0.0
        },
        reset
    );
    println!("{}│{}", dim, reset);

    println!("{}│{}  {}Current Averages{}", dim, reset, bold, reset);
    println!("{}│{}    RPC Latency:      {:.2} ms", dim, reset, summary.current_avg_rpc_ms);
    println!("{}│{}    Memory Usage:     {:.1} MB", dim, reset, summary.current_avg_memory_mb);
    println!("{}│{}", dim, reset);

    println!("{}│{}  {}7-Day Baseline{}", dim, reset, bold, reset);
    println!("{}│{}    RPC Latency:      {:.2} ms", dim, reset, summary.baseline.avg_rpc_latency_ms);
    println!("{}│{}    Memory Usage:     {:.1} MB", dim, reset, summary.baseline.avg_memory_mb);
    println!("{}│{}    Samples:          {}", dim, reset, summary.baseline.sample_count);
    println!("{}│{}", dim, reset);

    // Performance health indicator
    let health = if summary.degraded_count == 0 {
        ("Excellent", green)
    } else if (summary.degraded_count as f32 / summary.total_entries as f32) < 0.1 {
        ("Good", green)
    } else if (summary.degraded_count as f32 / summary.total_entries as f32) < 0.3 {
        ("Fair", yellow)
    } else {
        ("Poor", "\x1b[31m")
    };

    println!("{}│{}  {}Overall Health:{} {}{}{}",
        dim, reset, bold, reset, health.1, health.0, reset);
    println!("{}│{}", dim, reset);

    println!("{}╰──────────────────────────────────────────────────────────{}", dim, reset);
    println!();

    Ok(())
}

/// Rebuild baseline from last 7 days
fn rebuild_baseline(profiler: &Profiler, json: bool) -> Result<()> {
    let baseline = profiler.rebuild_baseline()?;

    if json {
        println!("{}", serde_json::to_string_pretty(&baseline)?);
        return Ok(());
    }

    let dim = "\x1b[2m";
    let reset = "\x1b[0m";
    let bold = "\x1b[1m";
    let green = "\x1b[32m";

    println!();
    println!("{}╭─ Rebuild Performance Baseline ───────────────────────────{}", dim, reset);
    println!("{}│{}", dim, reset);
    println!("{}│{}  {}✅ Baseline rebuilt from last 7 days{}", dim, reset, green, reset);
    println!("{}│{}", dim, reset);
    println!("{}│{}  {}New Baseline Metrics{}", dim, reset, bold, reset);
    println!("{}│{}    RPC Latency:    {:.2} ms", dim, reset, baseline.avg_rpc_latency_ms);
    println!("{}│{}    Memory Usage:   {:.1} MB", dim, reset, baseline.avg_memory_mb);
    println!("{}│{}    I/O Latency:    {:.2} ms", dim, reset, baseline.avg_io_latency_ms);
    println!("{}│{}    CPU Usage:      {:.1}%", dim, reset, baseline.avg_cpu_percent);
    println!("{}│{}", dim, reset);
    println!("{}│{}    {}Samples:{} {}", dim, reset, bold, reset, baseline.sample_count);
    println!("{}│{}", dim, reset);
    println!("{}╰──────────────────────────────────────────────────────────{}", dim, reset);
    println!();

    Ok(())
}

/// Get color for delta percentage
fn delta_color(delta: f32) -> &'static str {
    if delta > 50.0 {
        "\x1b[31m"  // Red
    } else if delta > 30.0 {
        "\x1b[33m"  // Yellow
    } else if delta > 15.0 {
        "\x1b[33m"  // Yellow
    } else if delta < -10.0 {
        "\x1b[32m"  // Green (improvement)
    } else {
        "\x1b[2m"   // Dim (normal)
    }
}

/// Format delta percentage with sign
fn format_delta(delta: f32) -> String {
    if delta > 0.0 {
        format!("+{:.1}", delta)
    } else {
        format!("{:.1}", delta)
    }
}
