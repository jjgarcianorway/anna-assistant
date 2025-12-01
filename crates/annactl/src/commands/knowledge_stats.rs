//! Knowledge Stats Command v6.0.0 - Grounded System Statistics
//!
//! v6.0.0: Shows real system statistics from real sources.
//! No fake "coverage" or "quality" metrics.

use anyhow::Result;
use owo_colors::OwoColorize;

use anna_common::grounded::{
    packages::PackageCounts,
    commands::count_path_executables,
    services::ServiceCounts,
    errors::ErrorCounts,
};

const THIN_SEP: &str = "------------------------------------------------------------";

/// Run the knowledge stats command
pub async fn run() -> Result<()> {
    println!();
    println!("{}", "  Anna System Statistics".bold());
    println!("{}", THIN_SEP);
    println!();

    // [PACKAGES] - from pacman
    print_packages_section();

    // [COMMANDS] - from PATH
    print_commands_section();

    // [SERVICES] - from systemctl
    print_services_section();

    // [SYSTEM HEALTH] - from journalctl
    print_health_section();

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

fn print_packages_section() {
    println!("{}", "[PACKAGES]".cyan());
    println!("  {}", "(source: pacman)".dimmed());

    let counts = PackageCounts::query();

    println!("  Total:       {}", counts.total);
    println!("  Explicit:    {} (user-installed)", counts.explicit);
    println!("  Dependency:  {} (auto-installed)", counts.dependency);
    println!("  AUR/Foreign: {}", counts.aur);

    println!();
}

fn print_commands_section() {
    println!("{}", "[COMMANDS]".cyan());
    println!("  {}", "(source: $PATH directories)".dimmed());

    let count = count_path_executables();
    println!("  Executables: {}", count);

    println!();
}

fn print_services_section() {
    println!("{}", "[SERVICES]".cyan());
    println!("  {}", "(source: systemctl)".dimmed());

    let counts = ServiceCounts::query();

    println!("  Unit files:  {}", counts.total);
    println!("  Running:     {}", counts.running);
    println!("  Enabled:     {}", counts.enabled);

    if counts.failed > 0 {
        println!("  Failed:      {}", counts.failed.to_string().red());
    } else {
        println!("  Failed:      {}", "0".green());
    }

    println!();
}

fn print_health_section() {
    println!("{}", "[SYSTEM HEALTH]".cyan());
    println!("  {}", "(source: journalctl, last 24h)".dimmed());

    let counts = ErrorCounts::query_24h();

    if counts.errors == 0 && counts.warnings == 0 {
        println!("  {}", "No errors or warnings".green());
    } else {
        if counts.errors > 0 {
            println!("  Errors:      {}", counts.errors.to_string().red());
        }
        if counts.warnings > 0 {
            println!("  Warnings:    {}", counts.warnings.to_string().yellow());
        }
        if counts.critical > 0 {
            println!("  Critical:    {}", counts.critical.to_string().red().bold());
        }
    }

    println!();
}
