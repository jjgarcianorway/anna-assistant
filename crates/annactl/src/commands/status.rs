//! Status Command v6.0.2 - Grounded System Status
//!
//! v6.0.2: Added [UPDATES] section for auto-update visibility
//! v6.0.0: Complete rewrite with real data sources
//! - Every number comes from a verifiable command
//! - No invented metrics, no fake percentages
//!
//! Data sources:
//! - Daemon: HTTP API at :7865/v1/health
//! - Packages: pacman -Q, pacman -Qe, pacman -Qm
//! - Commands: $PATH directory scanning
//! - Services: systemctl list-unit-files, systemctl --failed
//! - Errors: journalctl -p err/warning --since "24 hours ago"
//! - Updates: /var/lib/anna/update_state.json

use anyhow::Result;
use owo_colors::OwoColorize;

use anna_common::grounded::{
    packages::PackageCounts,
    commands::count_path_executables,
    services::ServiceCounts,
    errors::{ErrorCounts, get_top_error_units},
};
use anna_common::config::{AnnaConfig, UpdateState};
use anna_common::format_duration_secs;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const THIN_SEP: &str = "------------------------------------------------------------";

/// Run the status command
pub async fn run() -> Result<()> {
    println!();
    println!("{}", "  Anna Status".bold());
    println!("{}", THIN_SEP);
    println!();

    // [VERSION]
    print_version_section();

    // [DAEMON]
    print_daemon_section().await;

    // [PACKAGES] - real counts from pacman
    print_packages_section();

    // [COMMANDS] - real counts from PATH
    print_commands_section();

    // [SERVICES] - real counts from systemctl
    print_services_section();

    // [ERRORS] - real counts from journalctl (24h)
    print_errors_section();

    // [UPDATES] - auto-update status
    print_updates_section();

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

fn print_version_section() {
    println!("{}", "[VERSION]".cyan());
    println!("  Anna:       v{}", VERSION);
    println!();
}

async fn print_daemon_section() {
    println!("{}", "[DAEMON]".cyan());

    match get_daemon_info().await {
        Some(info) => {
            let uptime_str = format_duration_secs(info.uptime_secs);
            println!("  Status:     {} (up {})", "running".green(), uptime_str);
        }
        None => {
            println!("  Status:     {}", "stopped".red());
        }
    }

    println!();
}

fn print_packages_section() {
    println!("{}", "[PACKAGES]".cyan());
    println!("  {}", "(source: pacman -Q)".dimmed());

    let counts = PackageCounts::query();

    println!("  Installed:  {}", counts.total);
    println!(
        "  Explicit:   {} ({}%)",
        counts.explicit,
        if counts.total > 0 {
            (counts.explicit * 100) / counts.total
        } else {
            0
        }
    );
    println!(
        "  AUR:        {} ({}%)",
        counts.aur,
        if counts.total > 0 {
            (counts.aur * 100) / counts.total
        } else {
            0
        }
    );

    println!();
}

fn print_commands_section() {
    println!("{}", "[COMMANDS]".cyan());
    println!("  {}", "(source: $PATH)".dimmed());

    let count = count_path_executables();
    println!("  In PATH:    {}", count);

    println!();
}

fn print_services_section() {
    println!("{}", "[SERVICES]".cyan());
    println!("  {}", "(source: systemctl)".dimmed());

    let counts = ServiceCounts::query();

    println!("  Total:      {}", counts.total);
    println!("  Running:    {}", counts.running);

    if counts.failed > 0 {
        println!("  Failed:     {}", counts.failed.to_string().red());
    } else {
        println!("  Failed:     {}", "0".green());
    }

    println!();
}

fn print_errors_section() {
    println!("{}", "[ERRORS]".cyan());
    println!("  {}", "(source: journalctl, last 24h)".dimmed());

    let counts = ErrorCounts::query_24h();

    if counts.errors > 0 {
        println!("  Errors:     {}", counts.errors.to_string().red());
    } else {
        println!("  Errors:     {}", "0".green());
    }

    if counts.warnings > 0 {
        println!("  Warnings:   {}", counts.warnings.to_string().yellow());
    } else {
        println!("  Warnings:   0");
    }

    if counts.critical > 0 {
        println!("  Critical:   {}", counts.critical.to_string().red().bold());
    }

    // Show dominant error source if there are errors
    if counts.errors > 0 {
        let top_units = get_top_error_units(24, 1);
        if let Some(top) = top_units.first() {
            println!(
                "  Top source: {} ({} errors)",
                top.unit.cyan(),
                top.error_count
            );
        }
    }

    println!();
}

fn print_updates_section() {
    println!("{}", "[UPDATES]".cyan());
    println!("  {}", "(source: /var/lib/anna/update_state.json)".dimmed());

    let config = AnnaConfig::load();
    let state = UpdateState::load();

    // Mode
    let mode_str = if config.update.enabled {
        "auto".green().to_string()
    } else {
        "disabled".yellow().to_string()
    };
    println!("  Mode:       {} ({})", mode_str, if config.update.enabled { "enabled" } else { "disabled" });

    // Interval
    println!("  Interval:   {}m", config.update.interval_minutes);

    // Last check
    println!("  Last check: {}", state.format_last_check());

    // Last result
    let result_display = if state.last_result.is_empty() {
        "n/a".to_string()
    } else if state.last_result.contains("success") || state.last_result.contains("up to date") {
        state.last_result.green().to_string()
    } else if state.last_result.contains("available") {
        state.last_result.cyan().to_string()
    } else {
        state.last_result.clone()
    };
    println!("  Result:     {}", result_display);

    // Show latest version if different from current
    if let Some(ref latest) = state.latest_version {
        if !state.current_version.is_empty() && latest != &state.current_version {
            println!("  Update:     {} → {}", state.current_version.dimmed(), latest.cyan());
        }
    }

    // Next check
    if config.update.enabled {
        println!("  Next check: {}", state.format_next_check());
    }

    println!();
}

// ============================================================================
// Helper Functions
// ============================================================================

#[derive(serde::Deserialize)]
struct HealthResponse {
    #[allow(dead_code)]
    status: String,
    #[allow(dead_code)]
    version: String,
    #[allow(dead_code)]
    phase: String,
    uptime_secs: u64,
    #[allow(dead_code)]
    objects_tracked: usize,
}

async fn get_daemon_info() -> Option<HealthResponse> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .ok()?;

    let response = client
        .get("http://127.0.0.1:7865/v1/health")
        .send()
        .await
        .ok()?;

    response.json::<HealthResponse>().await.ok()
}
