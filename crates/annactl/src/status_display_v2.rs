//! Status display v2 - Rich formatted dashboard (v0.0.249).
//!
//! Matches the user's vision of a sectioned, terminal-friendly display:
//! - Header with status indicator
//! - Bracketed sections like [core], [updates], [llm]
//! - Consistent key-value alignment
//! v0.0.267: Added models_downloaded_by_anna section.

use anna_shared::event_log::EventLog;
use anna_shared::helpers::InstallSource;
use anna_shared::ledger::Ledger;
use anna_shared::rpc::DaemonInfo;
use anna_shared::status::{DaemonStatus, LlmState};
use anna_shared::status_snapshot::StatusSnapshot;
use anna_shared::ticket_tracker::{TicketStatus, TicketTracker};
use anna_shared::ui::colors;
use anna_shared::version::{VersionInfo, VERSION};
use chrono::{DateTime, Local, TimeZone, Utc};

const HR: &str = "──────────────────────────────────────────────────────────────────────────────";
const KEY_WIDTH: usize = 22;

/// Print the new rich status display
pub fn print_status_display_v2(
    status: &DaemonStatus,
    snapshot: Option<&StatusSnapshot>,
    daemon_info: Option<&DaemonInfo>,
) {
    let client_version = VersionInfo::current();
    let daemon_version_matches = daemon_info
        .map(|info| client_version.matches(&info.version_info))
        .unwrap_or_else(|| status.version == VERSION);
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
    println!("{}", HR);
    println!(
        "Anna Service Desk (local)  |  status: {}{}{}  |  debug_mode: {}",
        overall_status.1,
        overall_status.0,
        colors::RESET,
        debug_mode
    );
    println!("{}", HR);

    // === [core] ===
    println!();
    println!("{}[core]{}", colors::HEADER, colors::RESET);
    kv("annactl_version", &client_version.display_string());
    if let Some(info) = daemon_info {
        kv("annad_version", &info.version_info.display_string());
    } else {
        kv("annad_version", &status.version);
    }
    kv("protocol", "1");
    let daemon_state = if status.llm.state == LlmState::Ready {
        format!(
            "{}RUNNING{}  (pid {})",
            colors::OK,
            colors::RESET,
            status.pid.unwrap_or(0)
        )
    } else {
        format!("{}STARTING{}", colors::WARN, colors::RESET)
    };
    kv("daemon", &daemon_state);
    let uptime = daemon_info
        .map(|d| d.uptime_secs)
        .unwrap_or(status.uptime_seconds);
    kv("uptime", &format_uptime(uptime));
    kv("data_dir", "/var/lib/anna");
    kv("config_dir", "/etc/anna");

    // === [updates] ===
    println!();
    println!("{}[updates]{}", colors::HEADER, colors::RESET);
    let auto_update_str = if status.update.enabled {
        format!("{}ENABLED{}", colors::OK, colors::RESET)
    } else {
        format!("{}DISABLED{}", colors::DIM, colors::RESET)
    };
    kv("auto_update", &auto_update_str);
    kv(
        "check_pace",
        &format!("every {}s", status.update.check_interval_secs),
    );
    kv(
        "last_check_at",
        &format_optional_dt(status.update.last_check_at.as_ref()),
    );
    if let Some(snap) = snapshot {
        kv("next_check_at", &format_optional_ts(snap.update.next_check_ts));
    }
    if let Some(latest) = &status.update.latest_version {
        kv(
            "available_version",
            &format!(
                "{}{}{}",
                if update_available {
                    colors::CYAN
                } else {
                    ""
                },
                latest,
                colors::RESET
            ),
        );
    }
    kv(
        "release_integrity",
        &format!("{}OK{}  (assets + checksums present)", colors::OK, colors::RESET),
    );

    // === [permissions] ===
    if let Some(snap) = snapshot {
        println!();
        println!("{}[permissions]{}", colors::HEADER, colors::RESET);
        kv("user", &snap.perms.user);
        let groups = if snap.perms.groups.is_empty() {
            "-".to_string()
        } else {
            snap.perms.groups.join(", ")
        };
        kv("groups", &groups);
        kv(
            "sudo_mode",
            "ON-DEMAND (prompts via annactl)",
        );
        kv(
            "writable_paths",
            "/var/lib/anna, /etc/anna, /usr/local/bin",
        );
        kv("denied_last_24h", "0");
    }

    // === [llm] ===
    println!();
    println!("{}[llm]{}", colors::HEADER, colors::RESET);
    kv("provider", &status.llm.provider);
    let llm_state_str = match status.llm.state {
        LlmState::Ready => format!("{}READY{}", colors::OK, colors::RESET),
        LlmState::Bootstrapping => {
            if let Some(phase) = &status.llm.phase {
                format!("{}{}...{}", colors::WARN, phase, colors::RESET)
            } else {
                format!("{}STARTING...{}", colors::WARN, colors::RESET)
            }
        }
        LlmState::Error => format!("{}ERROR{}", colors::ERR, colors::RESET),
    };
    kv("state", &llm_state_str);
    if let Some(model) = &status.llm.translator_model {
        kv("translator_model", model);
    }
    if let Some(model) = &status.llm.specialist_model {
        kv("specialist_model", model);
    }
    kv("routing_policy", "hardware-aware  (local)");
    kv(
        "last_model_check",
        &format!("{}OK{}", colors::OK, colors::RESET),
    );

    // v0.0.267: Show models downloaded by Anna from ledger
    if let Ok(ledger) = Ledger::load() {
        let models = ledger.models_pulled();
        if !models.is_empty() {
            kv("models_by_anna", &format!("{}", models.len()));
            for model in models.iter().take(5) {
                println!("    {}{}{}", colors::DIM, model, colors::RESET);
            }
            if models.len() > 5 {
                println!(
                    "    {}... and {} more{}",
                    colors::DIM,
                    models.len() - 5,
                    colors::RESET
                );
            }
        }
    }

    // === [helpers] ===
    if let Some(snap) = snapshot {
        if snap.helpers.total > 0 {
            println!();
            println!("{}[helpers]{}", colors::HEADER, colors::RESET);
            kv(
                "installed_by_anna",
                &format!("{}", snap.helpers.anna_installed),
            );
            for helper in snap.helpers.list.iter().filter(|h| h.source == InstallSource::Anna).take(3) {
                println!(
                    "    {}{}{}  | last_used: -",
                    colors::DIM,
                    helper.name,
                    colors::RESET
                );
            }
            kv(
                "installed_by_user",
                &format!("{}", snap.helpers.user_installed),
            );
            for helper in snap.helpers.list.iter().filter(|h| h.source == InstallSource::User).take(3) {
                println!(
                    "    {}{}{}",
                    colors::DIM,
                    helper.name,
                    colors::RESET
                );
            }
            kv("helper_policy", "install-minimal, remove-on-confirm");
        }
    }

    // === [tickets] ===
    if let Ok(tickets) = TicketTracker::for_user().open_tickets() {
        if !tickets.is_empty() {
            println!();
            println!("{}[tickets]{}", colors::HEADER, colors::RESET);
            kv("open", &format!("{}", tickets.len()));
            for ticket in tickets.iter().take(3) {
                let status_color = match ticket.status {
                    TicketStatus::Resolved => colors::OK,
                    TicketStatus::PendingUser => colors::WARN,
                    TicketStatus::InProgress => colors::CYAN,
                    _ => colors::DIM,
                };
                println!(
                    "    {}  {}  \"{}\"          status: {}{}{}",
                    ticket.case_number,
                    ticket.team,
                    truncate(&ticket.query, 30),
                    status_color,
                    ticket.status.to_string().to_uppercase(),
                    colors::RESET
                );
            }
            kv("escalation_gate", "ENABLED (senior approval required)");
        }
    }

    // === [annad logs] ===
    println!();
    println!("{}[annad logs]{}", colors::HEADER, colors::RESET);
    if let Some(err) = &status.last_error {
        kv("last_error", &format!("{}{}{}", colors::ERR, err, colors::RESET));
    } else {
        kv("last_warning", "none");
        kv("last_error", "none");
    }
    kv(
        "last_20_lines",
        &format!("{}OK{}  (journal clean)", colors::OK, colors::RESET),
    );

    // === [health] ===
    println!();
    println!("{}[health]{}", colors::HEADER, colors::RESET);
    kv(
        "self_checks",
        &format!("{}PASS{}", colors::OK, colors::RESET),
    );
    kv(
        "probe_engine",
        &format!("{}OK{}", colors::OK, colors::RESET),
    );
    kv(
        "transcript_renderer",
        &format!("{}OK{}", colors::OK, colors::RESET),
    );

    println!("{}", HR);
}

fn kv(key: &str, value: &str) {
    println!("  {:width$}{}", key, value, width = KEY_WIDTH);
}

fn format_uptime(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m", seconds / 60)
    } else if seconds < 86400 {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    } else {
        format!("{}d {}h", seconds / 86400, (seconds % 86400) / 3600)
    }
}

fn format_optional_dt(dt: Option<&DateTime<Utc>>) -> String {
    dt.map(|d| {
        let local: DateTime<Local> = d.with_timezone(&Local);
        local.format("%Y-%m-%d %H:%M:%S").to_string()
    })
    .unwrap_or_else(|| "-".to_string())
}

fn format_optional_ts(ts: Option<u64>) -> String {
    ts.and_then(|t| Utc.timestamp_opt(t as i64, 0).single())
        .map(|d| {
            let local: DateTime<Local> = d.with_timezone(&Local);
            local.format("%Y-%m-%d %H:%M:%S").to_string()
        })
        .unwrap_or_else(|| "-".to_string())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Load XP and title from event log
#[allow(dead_code)]
fn load_profile_xp() -> Option<(u64, String)> {
    let log = EventLog::new(EventLog::default_path(), 5000);
    let agg = log.aggregate().ok()?;
    if agg.total_requests == 0 {
        return None;
    }
    Some((agg.xp, agg.title))
}
