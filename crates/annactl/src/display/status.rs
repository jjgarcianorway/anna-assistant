//! Status display for annactl status command.
//! Each section is a separate function to maintain <400 line files.

use anna_shared::config::AnnaConfig;
use anna_shared::status::DaemonStatus;

use super::colors::*;
use super::formatting::*;
use super::status_detail;

/// Print full status output
pub async fn print_status() {
    match crate::rpc::get_status().await {
        Ok(status) => {
            let config = AnnaConfig::load().ok();
            let debug_mode = config.as_ref().map(|c| c.debug_mode).unwrap_or(false);

            println!();
            println_colored("ANNA STATUS", BOLD);
            println!();

            if !debug_mode {
                print_summary(&status);
            }
            print_version(&status);
            print_updates(&status);
            print_backups(&status);
            print_truth();
            print_learning(&status);
            print_daemon(&status);
            status_detail::print_recovery(&status);
            print_permissions(&status);
            print_knowledge(&status);
            print_helpers();
            status_detail::print_stats_brief(&status);
            status_detail::print_team(&status);
            status_detail::print_tickets(&status);
            status_detail::print_health(&status);
            status_detail::print_models(&status, debug_mode);
            status_detail::print_config(&config, debug_mode, &status);
        }
        Err(e) => {
            print_colored("Error: ", RED);
            println!("{}", e);
        }
    }
}

fn print_summary(status: &DaemonStatus) {
    println_colored("SUMMARY", CYAN);

    let has_errors = status.error_summary.error_count > 0;
    let has_warnings = status.error_summary.warning_count > 0;
    print!("  health:        ");
    if has_errors {
        println_colored(&format!("ISSUES ({} errors, {} warnings)",
            status.error_summary.error_count, status.error_summary.warning_count), RED);
    } else if has_warnings {
        println_colored(&format!("OK ({} warnings)", status.error_summary.warning_count), YELLOW);
    } else {
        println_colored("OK", GREEN);
    }

    print!("  mode:          ");
    if !status.ticket_tracker.active_tickets.is_empty() {
        let ticket = &status.ticket_tracker.active_tickets[0];
        println_colored(&format!("{} ticket {}", ticket.status.to_string().to_uppercase(), ticket.id), CYAN);
    } else {
        println_colored("IDLE", DIM);
    }

    let learning = &status.learning_status;
    print!("  learning:      ");
    if learning.enabled {
        let total = learning.candidate_skills + learning.probation_skills + learning.trusted_skills;
        if total > 0 {
            println!("{} candidate, {} probation, {} trusted",
                learning.candidate_skills, learning.probation_skills, learning.trusted_skills);
        } else {
            println_colored("enabled (no skills)", DIM);
        }
    } else {
        println_colored("disabled", DIM);
    }

    print!("  updates:       ");
    if let Some(ref latest) = status.latest_version {
        if latest != &status.version {
            println_colored(&format!("AVAILABLE ({})", latest), YELLOW);
        } else {
            match status.update_state {
                anna_shared::status::UpdateCheckState::Success => {
                    if let Some(ref last) = status.last_update_check {
                        println_colored(&format!("OK (checked {})", format_time_ago(last)), GREEN);
                    } else {
                        println_colored("OK", GREEN);
                    }
                }
                anna_shared::status::UpdateCheckState::Failed => println_colored("FAILED", RED),
                anna_shared::status::UpdateCheckState::Checking => println_colored("CHECKING...", YELLOW),
                anna_shared::status::UpdateCheckState::NeverChecked => println_colored("never checked", DIM),
            }
        }
    } else {
        println_colored("not checked", DIM);
    }
    println!();
}

fn print_version(status: &DaemonStatus) {
    println_colored("VERSION", CYAN);
    let client_version = env!("CARGO_PKG_VERSION");
    let daemon_version = &status.version;
    let versions_match = client_version == daemon_version;

    print!("  annactl:       ");
    print_colored(client_version, if versions_match { GREEN } else { YELLOW });
    println!();

    print!("  annad:         ");
    print_colored(daemon_version, if versions_match { GREEN } else { YELLOW });
    if !status.build_info.git_sha.is_empty() {
        let dirty = if status.build_info.git_dirty { "*" } else { "" };
        print_colored(&format!(" ({}{})", status.build_info.git_sha, dirty), DIM);
    }
    println!();

    if !versions_match {
        print_colored("  [!] mismatch:  ", YELLOW);
        println_colored(&format!("client {} vs daemon {}", client_version, daemon_version), YELLOW);
    }

    if let Some(ref latest) = status.latest_version {
        print!("  available:     ");
        if latest != &status.version {
            print_colored(latest, YELLOW);
            println_colored(" (update available)", YELLOW);
        } else {
            print_colored(latest, GREEN);
            println_colored(" [current]", DIM);
        }
    }

    if let Some(ref ollama_ver) = status.ollama_version {
        print!("  ollama:        ");
        println_colored(ollama_ver, DIM);
    }

    if !status.build_info.integrity_ok {
        if let Some(ref err) = status.build_info.integrity_error {
            print_colored("  [!] integrity: ", YELLOW);
            println_colored(err, YELLOW);
        }
    }
    println!();
}

fn print_updates(status: &DaemonStatus) {
    println_colored("UPDATES", CYAN);

    print!("  interval:      ");
    if status.update_check_interval_secs > 0 {
        println!("{}s", status.update_check_interval_secs);
    } else {
        println_colored("disabled", DIM);
    }

    print!("  last check:    ");
    if let Some(ref last) = status.last_update_check {
        println_colored(&format_time_ago(last), DIM);
    } else {
        println_colored("never", DIM);
    }

    print!("  last result:   ");
    let state_color = match status.update_state {
        anna_shared::status::UpdateCheckState::Success => GREEN,
        anna_shared::status::UpdateCheckState::Failed => RED,
        anna_shared::status::UpdateCheckState::Checking => YELLOW,
        anna_shared::status::UpdateCheckState::NeverChecked => DIM,
    };
    println_colored(&status.update_state.to_string(), state_color);

    print!("  next check:    ");
    if let Some(ref next) = status.next_update_check {
        println_colored(&format_time_ago(next), DIM);
    } else {
        println_colored("not scheduled", DIM);
    }
    println!();
}

fn print_backups(status: &DaemonStatus) {
    println_colored("BACKUPS", CYAN);
    print!("  directory:     ");
    println_colored(&status.backup_info.directory, DIM);
    print!("  count:         ");
    println!("{}", status.backup_info.backup_count);
    if let Some(ref last) = status.backup_info.last_backup {
        print!("  last backup:   ");
        println_colored(last, DIM);
    } else {
        print!("  last backup:   ");
        println_colored("none", DIM);
    }
    if status.backup_info.total_size_bytes > 0 {
        print!("  total size:    ");
        let size_kb = status.backup_info.total_size_bytes / 1024;
        if size_kb > 1024 {
            println_colored(&format!("{:.1} MB", size_kb as f64 / 1024.0), DIM);
        } else {
            println_colored(&format!("{} KB", size_kb), DIM);
        }
    }
    println!();
}

fn print_truth() {
    println_colored("TRUTH", CYAN);
    print!("  claimgate:     ");
    println_colored("enabled", GREEN);
    print!("  local docs:    ");
    let (wiki_count, man_count, help_count) = count_local_docs();
    if wiki_count > 0 || man_count > 0 || help_count > 0 {
        let mut parts = Vec::new();
        if wiki_count > 0 { parts.push(format!("{} wiki", wiki_count)); }
        if man_count > 0 { parts.push(format!("{} man", man_count)); }
        if help_count > 0 { parts.push(format!("{} help", help_count)); }
        println_colored(&parts.join(", "), GREEN);
    } else {
        println_colored("none cached", DIM);
    }
    println!();
}

fn print_learning(status: &DaemonStatus) {
    println_colored("LEARNING", CYAN);
    print!("  mode:          ");
    if status.learning_status.enabled {
        println_colored("enabled", GREEN);
    } else {
        println_colored("disabled", DIM);
    }

    let learning = &status.learning_status;
    print!("  skills:        ");
    if learning.candidate_skills > 0 || learning.probation_skills > 0 || learning.trusted_skills > 0 {
        let mut parts = Vec::new();
        if learning.candidate_skills > 0 { parts.push(format!("{} candidate", learning.candidate_skills)); }
        if learning.probation_skills > 0 { parts.push(format!("{} probation", learning.probation_skills)); }
        if learning.trusted_skills > 0 { parts.push(format!("{} trusted", learning.trusted_skills)); }
        println!("{}", parts.join(", "));
    } else {
        println_colored("none", DIM);
    }

    if learning.promotions > 0 || learning.demotions > 0 {
        print!("  transitions:   ");
        let mut parts = Vec::new();
        if learning.promotions > 0 { parts.push(format!("{} promotions", learning.promotions)); }
        if learning.demotions > 0 { parts.push(format!("{} demotions", learning.demotions)); }
        println!("{}", parts.join(", "));
    }

    if learning.failed_experiments > 0 {
        print!("  negative:      ");
        println_colored(&format!("{} failed experiments", learning.failed_experiments), DIM);
    }
    println!();
}

fn print_daemon(status: &DaemonStatus) {
    println_colored("DAEMON", CYAN);
    print!("  state:         ");
    let state_color = match status.state {
        anna_shared::status::DaemonState::Ready => GREEN,
        anna_shared::status::DaemonState::Starting => YELLOW,
        anna_shared::status::DaemonState::Error => RED,
    };
    print_colored(&status.state.to_string().to_lowercase(), state_color);
    println_colored(&format!(" (uptime: {})", format_duration(status.uptime_secs)), DIM);

    // Socket info
    if !status.socket_health.path.is_empty() {
        print!("  socket:        ");
        if status.socket_health.exists {
            print_colored(&status.socket_health.path, DIM);
            let status_color = match status.socket_health.status {
                anna_shared::status::SocketStatus::Healthy => GREEN,
                anna_shared::status::SocketStatus::Unknown => DIM,
                _ => YELLOW,
            };
            print_colored(&format!(" [{}]", status.socket_health.status.to_string().to_lowercase()), status_color);
            println!();
        } else {
            println_colored(&format!("{} [missing]", status.socket_health.path), RED);
        }
    }

    print!("  ollama:        ");
    if status.ollama_running {
        print_colored("running", GREEN);
        if let Some(model) = &status.model {
            println_colored(&format!(" ({})", model), DIM);
        } else {
            println!();
        }
    } else {
        println_colored("not running", RED);
    }

    if let Some(gpu) = &status.gpu {
        print!("  gpu:           ");
        print_colored(gpu, CYAN);
        if let Some(vram) = status.vram_mb {
            println_colored(&format!(" ({} MB)", vram), DIM);
        } else {
            println!();
        }
    }
    println!();
}

fn print_permissions(status: &DaemonStatus) {
    let perms = &status.permissions;
    if perms.user.is_empty() { return; }

    println_colored("PERMISSIONS", CYAN);
    print!("  user:          ");
    print_colored(&perms.user, if perms.is_root { YELLOW } else { GREEN });
    if perms.is_root {
        println_colored(" [root]", YELLOW);
    } else {
        println!();
    }

    print!("  sudo:          ");
    if perms.has_sudo {
        println_colored("yes", GREEN);
    } else {
        println_colored("no", DIM);
    }

    if !perms.admin_groups.is_empty() {
        print!("  groups:        ");
        println_colored(&perms.admin_groups.join(", "), DIM);
    }
    println!();
}

fn print_knowledge(status: &DaemonStatus) {
    println_colored("KNOWLEDGE", CYAN);
    println!("  patterns:      {} built-in", status.pattern_count);
    println!("  recipes:       {} learned", status.recipe_count);
    print!("  memory:        ");
    if status.memory_experiences == 0 {
        println_colored("empty", DIM);
    } else {
        println!("{} experiences", status.memory_experiences);
    }
    for issue in &status.memory_health_issues {
        print_colored("    [!] ", YELLOW);
        println_colored(issue, YELLOW);
    }
    println!();
}

fn print_helpers() {
    let helpers = get_helpers_list();
    if helpers.is_empty() { return; }

    println_colored("HELPERS", CYAN);
    for (name, by_anna) in &helpers {
        print!("  ");
        print_colored(&format!("{:16}", name), DIM);
        if *by_anna {
            println_colored("[anna]", CYAN);
        } else {
            println_colored("[user]", DIM);
        }
    }
    println!();
}
