//! Status Command v6.0.0 - Grounded System Status
//!
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

use anyhow::Result;
use owo_colors::OwoColorize;

use anna_common::grounded::{
    packages::PackageCounts,
    commands::count_path_executables,
    services::ServiceCounts,
    errors::ErrorCounts,
};
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
