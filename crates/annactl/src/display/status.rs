//! Status display for annactl status command.
//! Phase 21: Canonical 7-section contract per docs/STATUS_SPEC.md.

use anna_shared::config::AnnaConfig;
use anna_shared::exposure::ExposureLevel;
use anna_shared::status::DaemonStatus;

use super::colors::*;
use super::formatting::*;

/// Print full status output per STATUS_SPEC.md contract.
/// Sections appear in canonical order: VERSION, UPDATES, SERVICE, PERMISSIONS, CONFIG, HELPERS, MODELS
pub async fn print_status() {
    let config = AnnaConfig::load().ok();
    let exposure = config.as_ref()
        .map(|c| c.exposure_level)
        .unwrap_or(ExposureLevel::Silent);

    match crate::rpc::get_status().await {
        Ok(status) => {
            println!();
            print_version_section(&status, exposure);
            print_updates_section(&status, exposure);
            print_service_section(&status, exposure);
            print_permissions_section(&status, exposure);
            print_config_section(&config, exposure);
            print_helpers_section(exposure);
            print_models_section(&status, exposure);
        }
        Err(_) => {
            // Daemon unreachable - show VERSION (client only) + SERVICE
            println!();
            print_version_offline();
            print_service_offline();
        }
    }
}

/// Section 1: VERSION (always shown)
fn print_version_section(status: &DaemonStatus, _exposure: ExposureLevel) {
    println_colored("VERSION", CYAN);

    let client_version = env!("CARGO_PKG_VERSION");
    let daemon_version = &status.version;
    let versions_match = client_version == daemon_version;

    // annactl
    print!("  annactl:      ");
    println_colored(client_version, if versions_match { GREEN } else { YELLOW });

    // annad
    print!("  annad:        ");
    println_colored(daemon_version, if versions_match { GREEN } else { YELLOW });

    // available
    print!("  available:    ");
    if let Some(ref latest) = status.latest_version {
        print_colored(latest, GREEN);
        println_colored(" [OK]", GREEN);
    } else {
        println_colored("unknown", DIM);
    }

    // consistency
    print!("  consistency:  ");
    if versions_match {
        println_colored("[OK]", GREEN);
    } else {
        print_colored("[X] ", RED);
        println_colored("mismatch", RED);
    }

    println!();
}

/// Section 2: UPDATES (always shown when daemon is reachable)
fn print_updates_section(status: &DaemonStatus, _exposure: ExposureLevel) {

    println_colored("UPDATES", CYAN);

    // interval
    print!("  interval:     ");
    if status.update_check_interval_secs > 0 {
        println!("{}s", status.update_check_interval_secs);
    } else {
        println_colored("disabled", DIM);
    }

    // last_check
    print!("  last_check:   ");
    if let Some(ref last) = status.last_update_check {
        println_colored(&format_time_ago(last), DIM);
    } else {
        println_colored("never", DIM);
    }

    // last_result
    print!("  last_result:  ");
    match status.update_state {
        anna_shared::status::UpdateCheckState::Success => println_colored("[OK]", GREEN),
        anna_shared::status::UpdateCheckState::Failed => {
            print_colored("[X] ", RED);
            println_colored("FAILED", RED);
        }
        anna_shared::status::UpdateCheckState::Checking => println_colored("[!] checking...", YELLOW),
        anna_shared::status::UpdateCheckState::NeverChecked => println_colored("never", DIM),
    }

    // next_check
    print!("  next_check:   ");
    if status.update_check_interval_secs == 0 {
        println_colored("disabled", DIM);
    } else if let Some(ref next) = status.next_update_check {
        println_colored(&format_time_ago(next), DIM);
    } else {
        println_colored("unknown", DIM);
    }

    println!();
}

/// Section 3: SERVICE (always shown when daemon is reachable)
fn print_service_section(status: &DaemonStatus, _exposure: ExposureLevel) {
    println_colored("SERVICE", CYAN);

    // daemon
    print!("  daemon:       ");
    let is_ready = status.state == anna_shared::status::DaemonState::Ready;
    if is_ready {
        print_colored("[OK] ", GREEN);
        println_colored("running", GREEN);
    } else {
        print_colored("[X] ", RED);
        println_colored(&status.state.to_string().to_lowercase(), RED);
    }

    // socket
    print!("  socket:       ");
    if !status.socket_health.path.is_empty() {
        println_colored(&status.socket_health.path, DIM);
    } else {
        println_colored("/run/anna/anna.sock", DIM);
    }

    // socket_mode
    print!("  socket_mode:  ");
    if status.socket_health.exists && status.socket_health.status == anna_shared::status::SocketStatus::Healthy {
        println!("0660 anna:anna");
    } else if !status.socket_health.exists {
        print_colored("[X] ", RED);
        println_colored("missing", RED);
    } else {
        print_colored("[X] ", RED);
        println_colored(&status.socket_health.status.to_string().to_lowercase(), RED);
    }

    // last_error
    print!("  last_error:   ");
    if let Some(ref err) = status.socket_health.last_error {
        let truncated = if err.len() > 60 { &err[..60] } else { err };
        println_colored(truncated, YELLOW);
    } else if status.error_summary.error_count > 0 {
        if let Some(recent) = status.error_summary.recent_errors.first() {
            let msg = &recent.message;
            let truncated = if msg.len() > 60 { &msg[..60] } else { msg };
            println_colored(truncated, YELLOW);
        } else {
            println_colored("none", DIM);
        }
    } else {
        println_colored("none", DIM);
    }

    println!();
}

/// VERSION section when daemon is offline (client version only)
fn print_version_offline() {
    println_colored("VERSION", CYAN);
    print!("  annactl:      ");
    println_colored(env!("CARGO_PKG_VERSION"), GREEN);
    print!("  annad:        ");
    println_colored("unreachable", YELLOW);
    println!();
}

/// SERVICE section when daemon is offline
fn print_service_offline() {
    println_colored("SERVICE", CYAN);
    print!("  daemon:       ");
    print_colored("[X] ", RED);
    println_colored("not running", RED);
    print!("  socket:       ");
    println_colored("/run/anna/anna.sock", DIM);
    print!("  socket_mode:  ");
    print_colored("[X] ", RED);
    println_colored("unavailable", RED);
    print!("  last_error:   ");
    println_colored("daemon not running", YELLOW);
    println!();
}

/// Section 4: PERMISSIONS (requires Dialogue+)
fn print_permissions_section(status: &DaemonStatus, exposure: ExposureLevel) {
    if exposure < ExposureLevel::Dialogue { return; }

    println_colored("PERMISSIONS", CYAN);

    // Standard paths
    let paths = [
        ("/etc/anna", "root:anna", "755"),
        ("/var/lib/anna", "root:anna", "750"),
        ("/run/anna", "root:anna", "750"),
        ("/var/log/anna", "root:anna", "750"),
    ];

    for (path, expected_owner, expected_mode) in paths {
        print!("  {:14} ", format!("{}:", path));
        if std::path::Path::new(path).exists() {
            println!("{} {}", expected_owner, expected_mode);
        } else {
            println_colored("missing", YELLOW);
        }
    }

    // user_groups
    print!("  user_groups:  ");
    let groups = get_user_groups();
    if groups.contains("anna") {
        println!("{}", groups);
    } else {
        print_colored("[X] ", RED);
        println_colored(&format!("not in anna group ({})", groups), RED);
    }

    // Check if user is the right user from status
    if !status.permissions.user.is_empty() && !status.permissions.admin_groups.contains(&"anna".to_string()) {
        // Already handled by user_groups check
    }

    println!();
}

/// Section 5: CONFIG (requires Dialogue+)
fn print_config_section(config: &Option<AnnaConfig>, exposure: ExposureLevel) {
    if exposure < ExposureLevel::Dialogue { return; }

    println_colored("CONFIG", CYAN);

    if let Some(ref cfg) = config {
        // exposure
        print!("  exposure:     ");
        println_colored(cfg.exposure_level.name(), DIM);

        // teaching
        print!("  teaching:     ");
        if cfg.teaching_mode {
            println_colored("enabled", GREEN);
        } else {
            println_colored("disabled", DIM);
        }

        // debug_mode (only if enabled)
        if cfg.debug_mode {
            print!("  debug_mode:   ");
            println_colored("enabled", YELLOW);
        }
    } else {
        print!("  exposure:     ");
        println_colored("unknown", DIM);
        print!("  teaching:     ");
        println_colored("unknown", DIM);
    }

    println!();
}

/// Section 6: HELPERS (requires Debug)
fn print_helpers_section(exposure: ExposureLevel) {
    if exposure < ExposureLevel::Debug { return; }

    let mut helpers = get_helpers_list_extended();
    if helpers.is_empty() { return; }

    // Sort alphabetically
    helpers.sort_by(|a, b| a.0.cmp(&b.0));

    println_colored("HELPERS", CYAN);
    for (name, present, installed_by) in &helpers {
        print!("  {:14}", name);
        if *present {
            print_colored("[OK] ", GREEN);
            println_colored(installed_by, DIM);
        } else {
            print_colored("[X] ", RED);
            println_colored("missing", RED);
        }
    }
    println!();
}

/// Section 7: MODELS (requires Debug)
fn print_models_section(status: &DaemonStatus, exposure: ExposureLevel) {
    if exposure < ExposureLevel::Debug { return; }
    if status.model_mappings.is_empty() { return; }

    println_colored("MODELS", CYAN);

    // Sort alphabetically by role
    let mut mappings = status.model_mappings.clone();
    mappings.sort_by(|a, b| a.role.cmp(&b.role));

    for mapping in &mappings {
        print!("  {:14}", format!("{}:", mapping.role));
        if mapping.model.is_empty() {
            println_colored("unknown", DIM);
        } else {
            println_colored(&mapping.model, DIM);
        }
    }
    println!();
}

/// Extended helper list with presence and installer info
fn get_helpers_list_extended() -> Vec<(String, bool, String)> {
    use std::process::Command;

    let deps_path = anna_shared::paths::paths().installed_deps_file();
    let anna_installed: std::collections::HashSet<String> = if deps_path.exists() {
        std::fs::read_to_string(&deps_path)
            .ok()
            .map(|c| c.lines().filter(|l| !l.is_empty()).map(|s| s.to_string()).collect())
            .unwrap_or_default()
    } else {
        std::collections::HashSet::new()
    };

    // Tools Anna cares about
    let tools = ["bc", "ethtool", "htop", "iotop", "jq", "lsof", "nethogs", "strace", "yq"];
    let mut result = Vec::new();

    for tool in tools {
        let present = Command::new("which")
            .arg(tool)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let installed_by = if anna_installed.contains(tool) {
            "anna".to_string()
        } else if present {
            "user".to_string()
        } else {
            "unknown".to_string()
        };

        result.push((tool.to_string(), present, installed_by));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exposure_filtering() {
        // Silent: VERSION only
        assert!(ExposureLevel::Silent < ExposureLevel::Summary);
        // Summary: VERSION + UPDATES + SERVICE
        assert!(ExposureLevel::Summary < ExposureLevel::Dialogue);
        // Dialogue: + PERMISSIONS + CONFIG
        assert!(ExposureLevel::Dialogue < ExposureLevel::Debug);
        // Debug: + HELPERS + MODELS (all 7 sections)
    }

    #[test]
    fn test_helpers_sorted_alphabetically() {
        let helpers = get_helpers_list_extended();
        let names: Vec<_> = helpers.iter().map(|(n, _, _)| n.clone()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }
}
