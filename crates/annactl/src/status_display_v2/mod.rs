//! Status display v2 - Rich formatted dashboard (v0.0.345).
//!
//! Matches the user's vision of a sectioned, terminal-friendly display:
//! - Header with status indicator
//! - Bracketed sections like [core], [updates], [llm], [ollama], [config]
//! - Consistent key-value alignment
//! v0.0.267: Added models_downloaded_by_anna section.
//! v0.0.339: Use centralized UI helpers for consistency.
//! v0.0.449: Enhanced per VISION.md - separate Ollama status, all config settings.
//!
//! Modularized into separate files to keep each under 400 lines.

mod config_section;
mod core_section;
mod formatters;
mod llm_section;
mod teams_section;

use anna_shared::rpc::DaemonInfo;
use anna_shared::status::{DaemonStatus, LlmState};
use anna_shared::status_snapshot::StatusSnapshot;
use anna_shared::ui::{colors, print_footer, print_hr};
use anna_shared::version::VersionInfo;

use config_section::{print_config_section, print_health_section, print_logs_section};
use core_section::{print_core_section, print_permissions_section, print_updates_section};
use llm_section::{print_llm_section, print_ollama_section};
use teams_section::{print_helpers_section, print_teams_section, print_tickets_section};

/// Print the new rich status display
pub fn print_status_display_v2(
    status: &DaemonStatus,
    snapshot: Option<&StatusSnapshot>,
    daemon_info: Option<&DaemonInfo>,
) {
    let client_version = VersionInfo::current();
    let daemon_version_matches = daemon_info
        .map(|info| client_version.matches(&info.version_info))
        .unwrap_or_else(|| status.version == anna_shared::version::VERSION);
    let update_available = snapshot
        .map(|s| s.update_available())
        .unwrap_or(status.update.update_available);

    // Determine overall status
    let overall_status = if status.llm.state == LlmState::Bootstrapping {
        ("STARTING", colors::WARN)
    } else if status.last_error.is_some() {
        ("ERROR", colors::ERR)
    } else if !daemon_version_matches {
        ("VERSION_MISMATCH", colors::WARN)
    } else {
        ("OPERATIONAL", colors::OK)
    };

    let debug_mode = snapshot
        .map(|s| if s.config.debug_mode { "ON" } else { "OFF" })
        .unwrap_or(if status.debug_mode { "ON" } else { "OFF" });

    // === HEADER ===
    print_hr();
    println!(
        "{}Anna Service Desk{} (local) | status: {}{}{}  | debug_mode: {}",
        colors::HEADER,
        colors::RESET,
        overall_status.1,
        overall_status.0,
        colors::RESET,
        debug_mode
    );
    print_hr();

    // === [core] ===
    println!();
    print_core_section(status, daemon_info);

    // === [updates] ===
    println!();
    print_updates_section(status, snapshot);

    // === [permissions] ===
    if let Some(snap) = snapshot {
        println!();
        print_permissions_section(snap);
    }

    // === [ollama] === v0.0.449: Separate Ollama status per VISION.md
    if let Some(snap) = snapshot {
        println!();
        print_ollama_section(snap);
    }

    // === [llm] ===
    println!();
    print_llm_section(status);

    // === [helpers] === v0.0.453: Enhanced per VISION.md
    if let Some(snap) = snapshot {
        if snap.helpers.total > 0 {
            println!();
            print_helpers_section(snap);
        }
    }

    // === [tickets] ===
    print_tickets_section();

    // === [annad logs] ===
    println!();
    print_logs_section(status);

    // v0.0.300: Removed [statistics], [telemetry], [learning] from status
    // These belong in "annactl stats" not "annactl status"
    // Status should focus on system health and daemon state

    // === [config] === v0.0.449: Show all config settings per VISION.md
    if let Some(snap) = snapshot {
        println!();
        print_config_section(snap);
    }

    // === [teams] === v0.0.454: Dynamic team availability per VISION.md
    if let Some(snap) = snapshot {
        if snap.teams.available_count > 0 {
            println!();
            print_teams_section(snap);
        }
    }

    // === [health] ===
    println!();
    print_health_section();

    print_footer();
}
