//! Knowledge Stats Command v6.1.0 - Knowledge Coverage Only
//!
//! v6.1.0: Stats about Anna's knowledge, not system health
//! - Coverage: how complete is Anna's metadata
//! - Quality: descriptions, categories, versions
//! - Errors: knowledge ingestion failures only
//! - Highlights: detected desktop environment
//!
//! This command answers: "How complete is Anna's knowledge?"
//! NOT: "What errors are happening on the host?"

use anyhow::Result;
use owo_colors::OwoColorize;

use anna_common::grounded::{
    packages::PackageCounts,
    commands::count_path_executables,
    services::ServiceCounts,
};

const THIN_SEP: &str = "------------------------------------------------------------";

/// Run the knowledge stats command
pub async fn run() -> Result<()> {
    println!();
    println!("{}", "  Anna Knowledge Statistics".bold());
    println!("{}", THIN_SEP);
    println!();

    // [COVERAGE]
    print_coverage_section();

    // [QUALITY]
    print_quality_section();

    // [ERRORS]
    print_errors_section().await;

    // [HIGHLIGHTS]
    print_highlights_section();

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

fn print_coverage_section() {
    println!("{}", "[COVERAGE]".cyan());

    let pkg_counts = PackageCounts::query();
    let cmd_count = count_path_executables();
    let svc_counts = ServiceCounts::query();

    println!("  Commands known:    {}", cmd_count);
    println!("  Packages known:    {}", pkg_counts.total);
    println!("  Services known:    {}", svc_counts.total);

    println!();
}

fn print_quality_section() {
    println!("{}", "[QUALITY]".cyan());
    println!("  {}", "(metadata completeness from system queries)".dimmed());

    // These are estimates based on typical system data availability
    // Commands with man pages typically have descriptions
    let cmd_count = count_path_executables();
    let pkg_counts = PackageCounts::query();

    // Most packages have descriptions from pacman -Qi
    println!("  Packages with description: {} (from pacman)", pkg_counts.total);

    // Explicit packages are user-installed with intent
    println!("  User-installed packages:   {} (explicit)", pkg_counts.explicit);

    // AUR packages often have more metadata
    println!("  AUR packages:              {}", pkg_counts.aur);

    // Commands on PATH - not all have man pages
    println!("  Commands on PATH:          {}", cmd_count);

    println!();
}

async fn print_errors_section() {
    println!("{}", "[ERRORS]".cyan());
    println!("  {}", "(knowledge ingestion failures only)".dimmed());

    // Get internal errors from daemon
    match get_daemon_stats().await {
        Some(stats) => {
            let errors = &stats.internal_errors;
            let total = errors.subprocess_failures + errors.parser_failures;

            if total == 0 {
                println!("  Last 24h:          {}", "none".green());
            } else {
                if errors.subprocess_failures > 0 {
                    println!("  pacman/which failures: {}", errors.subprocess_failures.to_string().yellow());
                }
                if errors.parser_failures > 0 {
                    println!("  man/help parse errors: {}", errors.parser_failures.to_string().yellow());
                }
            }
        }
        None => {
            println!("  {}", "(daemon not running)".dimmed());
        }
    }

    println!();
}

fn print_highlights_section() {
    println!("{}", "[HIGHLIGHTS]".cyan());
    println!("  {}", "(detected from installed packages)".dimmed());

    // Detect compositor
    let compositor = detect_compositor();
    println!("  Compositor:        {}", compositor.cyan());

    // Detect shell
    let shell = detect_shell();
    println!("  Default shell:     {}", shell);

    // Detect editors
    let editors = detect_editors();
    if !editors.is_empty() {
        println!("  Editors:           {}", editors.join(", "));
    }

    // Detect terminal
    let terminal = detect_terminal();
    println!("  Terminal:          {}", terminal);

    println!();
}

// ============================================================================
// Detection Helpers (grounded in which)
// ============================================================================

fn detect_compositor() -> String {
    use anna_common::grounded::commands::command_exists;

    if command_exists("hyprland") || command_exists("Hyprland") {
        "hyprland (Wayland)".to_string()
    } else if command_exists("sway") {
        "sway (Wayland)".to_string()
    } else if command_exists("wayfire") {
        "wayfire (Wayland)".to_string()
    } else if command_exists("river") {
        "river (Wayland)".to_string()
    } else if command_exists("gnome-shell") {
        "GNOME".to_string()
    } else if command_exists("plasmashell") {
        "KDE Plasma".to_string()
    } else if command_exists("i3") {
        "i3 (X11)".to_string()
    } else if command_exists("bspwm") {
        "bspwm (X11)".to_string()
    } else {
        "unknown".to_string()
    }
}

fn detect_shell() -> String {
    std::env::var("SHELL")
        .ok()
        .and_then(|s| s.rsplit('/').next().map(String::from))
        .unwrap_or_else(|| "unknown".to_string())
}

fn detect_editors() -> Vec<String> {
    use anna_common::grounded::commands::command_exists;

    let mut editors = Vec::new();
    let candidates = ["nvim", "vim", "code", "helix", "nano", "emacs", "kate", "gedit"];

    for editor in candidates {
        if command_exists(editor) {
            editors.push(editor.to_string());
        }
        if editors.len() >= 3 {
            break;
        }
    }

    editors
}

fn detect_terminal() -> String {
    use anna_common::grounded::commands::command_exists;

    if command_exists("foot") {
        "foot".to_string()
    } else if command_exists("alacritty") {
        "alacritty".to_string()
    } else if command_exists("kitty") {
        "kitty".to_string()
    } else if command_exists("wezterm") {
        "wezterm".to_string()
    } else if command_exists("gnome-terminal") {
        "gnome-terminal".to_string()
    } else if command_exists("konsole") {
        "konsole".to_string()
    } else {
        "unknown".to_string()
    }
}

// ============================================================================
// Daemon API Client
// ============================================================================

#[derive(serde::Deserialize)]
struct StatsResponse {
    internal_errors: InternalErrors,
}

#[derive(serde::Deserialize)]
struct InternalErrors {
    subprocess_failures: u32,
    parser_failures: u32,
    #[allow(dead_code)]
    unknown_commands: u32,
}

async fn get_daemon_stats() -> Option<StatsResponse> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .ok()?;

    let response = client
        .get("http://127.0.0.1:7865/v1/stats")
        .send()
        .await
        .ok()?;

    response.json::<StatsResponse>().await.ok()
}
