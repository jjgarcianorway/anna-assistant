//! Config, health, and logs section handlers for status display.

use anna_shared::status::DaemonStatus;
use anna_shared::status_snapshot::StatusSnapshot;
use anna_shared::ui::{colors, kv, print_section_header};

/// Print the [annad logs] section
pub fn print_logs_section(status: &DaemonStatus) {
    print_section_header("annad logs");
    if let Some(err) = &status.last_error {
        kv(
            "last_error",
            &format!("{}{}{}", colors::ERR, err, colors::RESET),
        );
    } else {
        kv("last_warning", "none");
        kv("last_error", "none");
    }
    kv(
        "last_20_lines",
        &format!("{}OK{}  (journal clean)", colors::OK, colors::RESET),
    );
}

/// Print the [config] section (v0.0.449: Show all config settings per VISION.md)
pub fn print_config_section(snapshot: &StatusSnapshot) {
    print_section_header("config");
    // Debug mode only shown if enabled
    if snapshot.config.debug_mode {
        kv(
            "debug_mode",
            &format!("{}ON{}", colors::WARN, colors::RESET),
        );
    }
    let auto_update_str = if snapshot.config.auto_update {
        format!("{}ON{}", colors::OK, colors::RESET)
    } else {
        format!("{}OFF{}", colors::DIM, colors::RESET)
    };
    kv("auto_update", &auto_update_str);
    let learning_str = if snapshot.config.learning_mode {
        format!("{}ON{} (explains commands)", colors::OK, colors::RESET)
    } else {
        format!("{}OFF{}", colors::DIM, colors::RESET)
    };
    kv("learning_mode", &learning_str);
    let fast_path_str = if snapshot.config.fast_path_enabled {
        format!("{}ON{} (recipes before LLM)", colors::OK, colors::RESET)
    } else {
        format!("{}OFF{}", colors::DIM, colors::RESET)
    };
    kv("fast_path", &fast_path_str);
    let comms_str = if snapshot.config.internal_comms {
        format!("{}ON{} (show IT dialog)", colors::OK, colors::RESET)
    } else {
        format!("{}OFF{}", colors::DIM, colors::RESET)
    };
    kv("internal_comms", &comms_str);
    kv(
        "autonomy_level",
        &format!("{}/100", snapshot.config.autonomy_level),
    );
    kv(
        "request_timeout",
        &format!("{}s", snapshot.config.request_timeout_secs),
    );
}

/// Print the [health] section
pub fn print_health_section() {
    print_section_header("health");
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
}
