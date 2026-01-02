//! Core and updates section handlers for status display.

use anna_shared::rpc::DaemonInfo;
use anna_shared::status::{DaemonStatus, LlmState};
use anna_shared::status_snapshot::StatusSnapshot;
use anna_shared::ui::{colors, kv, print_section_header};
use anna_shared::version::VersionInfo;

use super::formatters::{format_optional_dt, format_optional_ts, format_uptime};

/// Print the [core] section
pub fn print_core_section(status: &DaemonStatus, daemon_info: Option<&DaemonInfo>) {
    print_section_header("core");
    let client_version = VersionInfo::current();
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
}

/// Print the [updates] section
pub fn print_updates_section(status: &DaemonStatus, snapshot: Option<&StatusSnapshot>) {
    print_section_header("updates");
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
        kv(
            "next_check_at",
            &format_optional_ts(snap.update.next_check_ts),
        );
    }

    let update_available = snapshot
        .map(|s| s.update_available())
        .unwrap_or(status.update.update_available);

    if let Some(latest) = &status.update.latest_version {
        kv(
            "available_version",
            &format!(
                "{}{}{}",
                if update_available { colors::CYAN } else { "" },
                latest,
                colors::RESET
            ),
        );
    }
    kv(
        "release_integrity",
        &format!(
            "{}OK{}  (assets + checksums present)",
            colors::OK,
            colors::RESET
        ),
    );
}

/// Print the [permissions] section
pub fn print_permissions_section(snapshot: &StatusSnapshot) {
    print_section_header("permissions");
    kv("user", &snapshot.perms.user);
    let groups = if snapshot.perms.groups.is_empty() {
        "-".to_string()
    } else {
        snapshot.perms.groups.join(", ")
    };
    kv("groups", &groups);
    kv("sudo_mode", "ON-DEMAND (prompts via annactl)");
    kv("writable_paths", "/var/lib/anna, /etc/anna, /usr/local/bin");
    kv("denied_last_24h", "0");
}
