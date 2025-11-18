//! Startup Health Summary
//!
//! Beta.53: Display comprehensive system health on startup
//! Shows current state + 30-day Historian trends

use anna_common::display::UI;
use anna_common::historian::SystemSummary;
use anna_common::types::SystemFacts;

/// Display startup health summary with Historian data
pub fn display_startup_summary(
    facts: &SystemFacts,
    historian: Option<&SystemSummary>,
    current_model: &str,
) {
    let ui = UI::auto();

    // Health status
    let status = if !facts.failed_services.is_empty() {
        "⚠️  Warning"
    } else {
        "✓ Healthy"
    };

    println!();
    ui.info(&format!("System Health: {}", status));
    println!();

    // 30-Day Summary section
    if let Some(hist) = historian {
        display_historian_summary(hist);
    } else {
        ui.info("(Historian data not yet available - collecting...)");
        println!();
    }

    // Recent issues/alerts
    display_current_alerts(facts);

    // Model recommendation if needed
    if current_model == "llama3.2:3b" && facts.total_memory_gb >= 8.0 {
        println!();
        ui.warning("💡 Tip: Your system can run better models. Try 'upgrade model' for smarter responses.");
    }

    println!("────────────────────────────────────────────────────────────────────");
    println!();
}

/// Display Historian summary section
fn display_historian_summary(sys_summary: &SystemSummary) {
    let ui = UI::auto();

    ui.info("30-Day Summary:");

    // Boot performance
    let boot = &sys_summary.boot_trends;
    let trend_str = match boot.trend {
        anna_common::historian::Trend::Down => "↓ improving",
        anna_common::historian::Trend::Flat => "→ stable",
        anna_common::historian::Trend::Up => "↑ degrading",
    };
    println!("  • Boot time: {:.1}s average ({})",
        boot.avg_boot_time_ms as f64 / 1000.0, trend_str);

    // CPU usage
    let cpu = &sys_summary.cpu_trends;
    let cpu_trend = match cpu.trend {
        anna_common::historian::Trend::Down => "↓ decreasing",
        anna_common::historian::Trend::Flat => "→ stable",
        anna_common::historian::Trend::Up => "↑ increasing",
    };
    println!("  • CPU: {:.1}% average ({})",
        cpu.avg_utilization_percent, cpu_trend);

    // Health scores
    let health = &sys_summary.health_summary;
    println!("  • Health: stability {}/100, performance {}/100",
        health.avg_stability_score, health.avg_performance_score);

    // Error trends
    let errors = &sys_summary.error_trends;
    if errors.total_errors > 0 || errors.total_criticals > 0 {
        println!("  • Errors: {} total, {} critical",
            errors.total_errors, errors.total_criticals);
    }

    println!();
}

/// Display current alerts and issues
fn display_current_alerts(facts: &SystemFacts) {
    if facts.failed_services.is_empty() &&
       facts.package_cache_size_gb < 5.0 &&
       facts.orphan_packages.is_empty() {
        return; // No alerts to show
    }

    let ui = UI::auto();

    if !facts.failed_services.is_empty() {
        ui.info("Recent Trends (Historian):");
        for (i, service) in facts.failed_services.iter().take(3).enumerate() {
            println!("  {}. ⚠️  Service failed: {}", i + 1, service);
        }
        if facts.failed_services.len() > 3 {
            println!("     ... and {} more", facts.failed_services.len() - 3);
        }
        println!();
    }

    // Package cache warning
    if facts.package_cache_size_gb > 5.0 {
        println!("  ℹ️  Package cache: {:.1} GB (consider cleanup)", facts.package_cache_size_gb);
    }

    // Orphan packages
    if !facts.orphan_packages.is_empty() {
        println!("  ℹ️  Orphaned packages: {} (consider reviewing)", facts.orphan_packages.len());
    }

    if facts.package_cache_size_gb > 5.0 || !facts.orphan_packages.is_empty() {
        println!();
    }
}

/// Compact one-line summary for quick status checks
pub fn get_one_line_summary(facts: &SystemFacts, _historian: Option<&SystemSummary>) -> String {
    let health = if !facts.failed_services.is_empty() {
        "⚠️"
    } else {
        "✓"
    };

    let uptime_hours = facts.system_health
        .as_ref()
        .map(|h| h.system_uptime.uptime_seconds as f64 / 3600.0)
        .unwrap_or(0.0);

    let uptime_str = if uptime_hours >= 24.0 {
        format!("{:.0}d", uptime_hours / 24.0)
    } else {
        format!("{:.0}h", uptime_hours)
    };

    let mem_pct = if let Some(mem_info) = &facts.memory_usage_info {
        ((mem_info.used_ram_gb as f64 / facts.total_memory_gb) * 100.0) as u32
    } else {
        0
    };

    let issues_count = facts.failed_services.len();

    if issues_count > 0 {
        format!("{} System: {} uptime, {}% mem, {} issues",
            health, uptime_str, mem_pct, issues_count)
    } else {
        format!("{} System: {} uptime, {}% mem, healthy",
            health, uptime_str, mem_pct)
    }
}
