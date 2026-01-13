//! Display utilities for CLI output - colors, step printing, status display.
//! v0.0.992: Added proactive alert display
//! v0.1.0: Added debug mode separation for clean "fly on the wall" experience
//! v0.1.1: Removed box drawing for cleaner look
//! v0.3.20: Output contract compliance - no icons, ASCII only, truecolor

use anna_shared::config::AnnaConfig;
use anna_shared::memory::memory_path;
use anna_shared::monitor::{IssueStore, Severity};
use anna_shared::rpc::{AskResult, StepType};
use anna_shared::stats::PersistentStats;
use std::io::{self, Write};
use std::sync::OnceLock;

/// Cached debug mode flag (loaded once at startup)
static DEBUG_MODE: OnceLock<bool> = OnceLock::new();

/// Check if debug mode is enabled (cached)
fn is_debug_mode() -> bool {
    *DEBUG_MODE.get_or_init(|| {
        AnnaConfig::load().map(|c| c.debug_mode).unwrap_or(true)
    })
}

// Color constants
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const RED: &str = "\x1b[31m";
pub const BLUE: &str = "\x1b[34m";
pub const CYAN: &str = "\x1b[36m";
pub const MAGENTA: &str = "\x1b[35m";
pub const WHITE: &str = "\x1b[37;1m";
pub const DIM: &str = "\x1b[2m";
pub const BOLD: &str = "\x1b[1m";
pub const RESET: &str = "\x1b[0m";

/// Print colored text (no newline)
pub fn print_colored(text: &str, color: &str) {
    print!("{}{}{}", color, text, RESET);
}

/// Print colored text with newline
pub fn println_colored(text: &str, color: &str) {
    println!("{}{}{}", color, text, RESET);
}

/// v0.3.26: Count local documentation files for integrity display
fn count_local_docs() -> (usize, usize, usize) {
    use anna_shared::docs::{man_cache_dir, help_cache_dir};
    use anna_shared::wiki::wiki_articles_dir;

    let wiki_count = std::fs::read_dir(wiki_articles_dir())
        .map(|entries| entries.filter_map(|e| e.ok()).count())
        .unwrap_or(0);

    let man_count = std::fs::read_dir(man_cache_dir())
        .map(|entries| entries.filter_map(|e| e.ok()).count())
        .unwrap_or(0);

    let help_count = std::fs::read_dir(help_cache_dir())
        .map(|entries| entries.filter_map(|e| e.ok()).count())
        .unwrap_or(0);

    (wiki_count, man_count, help_count)
}

/// Print the greeting
pub fn print_greeting() {
    println!();
    println_colored("Anna - Arch Linux Assistant", BOLD);
    println_colored("Ask questions about your system in plain English.", DIM);
    println_colored("Type 'quit' or Ctrl-D to exit.", DIM);
    println!();
}

/// Print status - clean format without boxes
pub async fn print_status() {
    match crate::rpc::get_status().await {
        Ok(status) => {
            let config = AnnaConfig::load().ok();
            let debug_mode = config.as_ref().map(|c| c.debug_mode).unwrap_or(false);

            println!();
            println_colored("ANNA STATUS", BOLD);
            println!();

            // v0.3.29: Non-debug mode STATUS SUMMARY at top
            if !debug_mode {
                println_colored("SUMMARY", CYAN);

                // Health status
                let has_errors = status.error_summary.error_count > 0;
                let has_warnings = status.error_summary.warning_count > 0;
                print!("  health:        ");
                if has_errors {
                    println_colored(&format!("ISSUES ({} errors, {} warnings)",
                        status.error_summary.error_count,
                        status.error_summary.warning_count), RED);
                } else if has_warnings {
                    println_colored(&format!("OK ({} warnings)", status.error_summary.warning_count), YELLOW);
                } else {
                    println_colored("OK", GREEN);
                }

                // Current mode/ticket
                print!("  mode:          ");
                if !status.ticket_tracker.active_tickets.is_empty() {
                    let ticket = &status.ticket_tracker.active_tickets[0];
                    println_colored(&format!("{} ticket {}", ticket.status.to_string().to_uppercase(), ticket.id), CYAN);
                } else {
                    println_colored("IDLE", DIM);
                }

                // Learning summary
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

                // Updates
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

            // VERSION - v0.3.23: Show both client and daemon versions with mismatch detection
            println_colored("VERSION", CYAN);
            let client_version = env!("CARGO_PKG_VERSION");
            let daemon_version = &status.version;
            let versions_match = client_version == daemon_version;

            print!("  annactl:       ");
            print_colored(client_version, if versions_match { GREEN } else { YELLOW });
            println!();

            print!("  annad:         ");
            print_colored(daemon_version, if versions_match { GREEN } else { YELLOW });
            // v0.3.22: Show build info (git sha)
            if !status.build_info.git_sha.is_empty() {
                let dirty = if status.build_info.git_dirty { "*" } else { "" };
                print_colored(&format!(" ({}{})", status.build_info.git_sha, dirty), DIM);
            }
            println!();

            // Warn on version mismatch
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

            // v0.3.22: Version integrity check
            if !status.build_info.integrity_ok {
                if let Some(ref err) = status.build_info.integrity_error {
                    print_colored("  [!] integrity: ", YELLOW);
                    println_colored(err, YELLOW);
                }
            }

            println!();

            // UPDATES (v0.3.25)
            println_colored("UPDATES", CYAN);

            // Check interval
            print!("  interval:      ");
            if status.update_check_interval_secs > 0 {
                println!("{}s", status.update_check_interval_secs);
            } else {
                println_colored("disabled", DIM);
            }

            // Last check
            print!("  last check:    ");
            if let Some(ref last) = status.last_update_check {
                println_colored(&format_time_ago(last), DIM);
            } else {
                println_colored("never", DIM);
            }

            // Last result
            print!("  last result:   ");
            let state_color = match status.update_state {
                anna_shared::status::UpdateCheckState::Success => GREEN,
                anna_shared::status::UpdateCheckState::Failed => RED,
                anna_shared::status::UpdateCheckState::Checking => YELLOW,
                anna_shared::status::UpdateCheckState::NeverChecked => DIM,
            };
            println_colored(&status.update_state.to_string(), state_color);

            // Next check
            print!("  next check:    ");
            if let Some(ref next) = status.next_update_check {
                println_colored(&format_time_ago(next), DIM);
            } else {
                println_colored("not scheduled", DIM);
            }
            println!();

            // BACKUPS (v0.3.24)
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

            // TRUTH (v0.3.26) - Truth enforcement status
            println_colored("TRUTH", CYAN);

            // ClaimGate status
            print!("  claimgate:     ");
            println_colored("enabled", GREEN);

            // Local docs corpus status
            print!("  local docs:    ");
            let (wiki_count, man_count, help_count) = count_local_docs();
            if wiki_count > 0 || man_count > 0 || help_count > 0 {
                let mut parts = Vec::new();
                if wiki_count > 0 {
                    parts.push(format!("{} wiki", wiki_count));
                }
                if man_count > 0 {
                    parts.push(format!("{} man", man_count));
                }
                if help_count > 0 {
                    parts.push(format!("{} help", help_count));
                }
                println_colored(&parts.join(", "), GREEN);
            } else {
                println_colored("none cached", DIM);
            }
            println!();

            // LEARNING (v0.3.27) - Skill learning status
            println_colored("LEARNING", CYAN);

            // Learning mode
            print!("  mode:          ");
            if status.learning_status.enabled {
                println_colored("enabled", GREEN);
            } else {
                println_colored("disabled", DIM);
            }

            // Skill tiers
            print!("  skills:        ");
            let learning = &status.learning_status;
            if learning.candidate_skills > 0 || learning.probation_skills > 0 || learning.trusted_skills > 0 {
                let mut parts = Vec::new();
                if learning.candidate_skills > 0 {
                    parts.push(format!("{} candidate", learning.candidate_skills));
                }
                if learning.probation_skills > 0 {
                    parts.push(format!("{} probation", learning.probation_skills));
                }
                if learning.trusted_skills > 0 {
                    parts.push(format!("{} trusted", learning.trusted_skills));
                }
                println!("{}", parts.join(", "));
            } else {
                println_colored("none", DIM);
            }

            // Promotions/demotions
            if learning.promotions > 0 || learning.demotions > 0 {
                print!("  transitions:   ");
                let mut parts = Vec::new();
                if learning.promotions > 0 {
                    parts.push(format!("{} promotions", learning.promotions));
                }
                if learning.demotions > 0 {
                    parts.push(format!("{} demotions", learning.demotions));
                }
                println!("{}", parts.join(", "));
            }

            // Failed experiments
            if learning.failed_experiments > 0 {
                print!("  negative:      ");
                println_colored(&format!("{} failed experiments", learning.failed_experiments), DIM);
            }
            println!();

            // DAEMON
            println_colored("DAEMON", CYAN);

            print!("  state:         ");
            let state_color = match status.state {
                anna_shared::status::DaemonState::Ready => GREEN,
                anna_shared::status::DaemonState::Starting => YELLOW,
                anna_shared::status::DaemonState::Error => RED,
            };
            print_colored(&status.state.to_string().to_lowercase(), state_color);
            println_colored(&format!(" (uptime: {})", format_duration(status.uptime_secs)), DIM);

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

            // PERMISSIONS
            let perms = &status.permissions;
            if !perms.user.is_empty() {
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

            // KNOWLEDGE
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

            // HELPERS
            let helpers = get_helpers_list();
            if !helpers.is_empty() {
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

            // QUICK STATS (brief version for status, detailed in stats command)
            println_colored("STATS", CYAN);
            let rpg = &status.rpg_stats;

            // Title and XP bar
            print!("  ");
            print_colored(&rpg.title, MAGENTA);
            print!(" ");
            println_colored(&rpg.xp_bar(), DIM);

            // Questions answered
            if rpg.total_questions > 0 {
                print!("  requests:      ");
                print!("{}", rpg.total_questions);
                let solved_alone = status.solved_alone_count;
                if solved_alone > 0 {
                    let pct = (solved_alone as f64 / rpg.total_questions as f64 * 100.0) as u32;
                    print_colored(&format!(" ({}% solved alone)", pct), DIM);
                }
                println!();
            }

            // Reliability
            if rpg.total_questions > 5 {
                print!("  reliability:   ");
                let rel_pct = (rpg.reliability * 100.0) as u32;
                let rel_color = if rel_pct >= 90 { GREEN } else if rel_pct >= 70 { YELLOW } else { RED };
                println_colored(&format!("{}%", rel_pct), rel_color);
            }

            // Escalated tickets
            if status.escalated_tickets_count > 0 {
                print!("  escalated:     ");
                println_colored(&format!("{}", status.escalated_tickets_count), YELLOW);
            }
            println!();

            // TEAM ROSTER (v0.3.3)
            let roster = &status.team_roster;
            if roster.total_specialists > 0 {
                println_colored("TEAM", CYAN);
                print!("  specialists:   ");
                print_colored(&format!("{}", roster.total_specialists), GREEN);
                println_colored(&format!(" across {} departments", roster.specialists.len()), DIM);

                // Show departments with specialists
                for (dept, specialists) in &roster.specialists {
                    let junior_count = specialists.iter().filter(|s| !s.is_senior).count();
                    let senior_count = specialists.iter().filter(|s| s.is_senior).count();
                    print!("    ");
                    print_colored(&format!("{:12}", dept), DIM);
                    print!(" ");
                    if junior_count > 0 {
                        print_colored(&format!("{}J", junior_count), CYAN);
                    }
                    if senior_count > 0 {
                        print!(" ");
                        print_colored(&format!("{}Sr", senior_count), YELLOW);
                    }
                    println!();
                }

                // v0.3.5: Top performers leaderboard
                let mut all_specialists: Vec<_> = roster.specialists.values().flatten().collect();
                all_specialists.sort_by(|a, b| b.tickets_resolved.cmp(&a.tickets_resolved));
                let top_performers: Vec<_> = all_specialists.into_iter()
                    .filter(|s| s.tickets_handled > 0)
                    .take(3)
                    .collect();

                if !top_performers.is_empty() {
                    println_colored("  top performers:", DIM);
                    for (i, spec) in top_performers.iter().enumerate() {
                        let medal = match i { 0 => "1.", 1 => "2.", 2 => "3.", _ => "  " };
                        let rate = if spec.tickets_handled > 0 {
                            spec.tickets_resolved as f64 / spec.tickets_handled as f64 * 100.0
                        } else { 0.0 };
                        print!("    ");
                        print_colored(medal, YELLOW);
                        print!(" ");
                        print_colored(&format!("{:12}", spec.name), CYAN);
                        print!(" {} resolved", spec.tickets_resolved);
                        print_colored(&format!(" ({:.0}%)", rate),
                            if rate >= 80.0 { GREEN } else if rate >= 50.0 { YELLOW } else { DIM });
                        if spec.avg_resolution_ms > 0 {
                            let secs = spec.avg_resolution_ms / 1000;
                            print_colored(&format!(" avg {}s", secs), DIM);
                        }
                        println!();
                    }
                }
                println!();
            }

            // TICKETS (v0.3.3)
            let tickets = &status.ticket_tracker;
            if tickets.next_number > 1 || !tickets.active_tickets.is_empty() {
                println_colored("TICKETS", CYAN);
                print!("  today:         ");
                println!("{} tickets", tickets.today_count);

                // Show per-department stats
                for (dept, stats) in &tickets.dept_stats {
                    if stats.total_received > 0 {
                        print!("    ");
                        print_colored(&format!("{:12}", dept), DIM);
                        print!(" {} handled", stats.total_received);
                        if stats.resolved > 0 {
                            let rate = stats.resolved as f64 / stats.total_received as f64 * 100.0;
                            print_colored(&format!(" ({:.0}% resolved)", rate),
                                if rate >= 80.0 { GREEN } else if rate >= 50.0 { YELLOW } else { RED });
                        }
                        println!();
                    }
                }

                // Show active tickets with state and elapsed time
                if !tickets.active_tickets.is_empty() {
                    println!();
                    println_colored("  active:", YELLOW);
                    for ticket in &tickets.active_tickets {
                        print!("    ");
                        print_colored(&ticket.id, CYAN);
                        print!(" ");

                        // v0.3.29: Show status with color
                        let status_color = match ticket.status {
                            anna_shared::status::TicketStatus::Open => DIM,
                            anna_shared::status::TicketStatus::Investigating => CYAN,
                            anna_shared::status::TicketStatus::Experimenting => MAGENTA,
                            anna_shared::status::TicketStatus::InProgress => YELLOW,
                            anna_shared::status::TicketStatus::Escalated => RED,
                            anna_shared::status::TicketStatus::Resolved => GREEN,
                            anna_shared::status::TicketStatus::Failed => RED,
                        };
                        print_colored(&format!("[{}]", ticket.status), status_color);
                        print!(" ");

                        // v0.3.29: Show elapsed time
                        if let Ok(created) = chrono::DateTime::parse_from_rfc3339(&ticket.created_at) {
                            let elapsed_secs = (chrono::Utc::now() - created.with_timezone(&chrono::Utc)).num_seconds();
                            if elapsed_secs >= 0 {
                                print_colored(&format!("({})", format_duration(elapsed_secs as u64)), DIM);
                                print!(" ");
                            }
                        }

                        print_colored(&ticket.summary, DIM);
                        println!();
                    }
                }
                println!();
            }

            // v0.3.22: HEALTH (only show if there are issues)
            let has_socket_issues = status.socket_health.status != anna_shared::status::SocketStatus::Healthy
                && status.socket_health.status != anna_shared::status::SocketStatus::Unknown;
            let has_errors = status.error_summary.error_count > 0;

            if has_socket_issues || has_errors {
                println_colored("HEALTH", CYAN);

                if has_socket_issues {
                    print!("  socket:        ");
                    println_colored(&status.socket_health.status.to_string().to_lowercase(), YELLOW);
                    if let Some(ref err) = status.socket_health.last_error {
                        print_colored("    ", DIM);
                        println_colored(err, YELLOW);
                    }
                }

                if has_errors {
                    print!("  errors:        ");
                    println_colored(&format!("{} errors, {} warnings",
                        status.error_summary.error_count,
                        status.error_summary.warning_count),
                        if status.error_summary.error_count > 0 { RED } else { YELLOW });

                    for err in status.error_summary.recent_errors.iter().take(3) {
                        print_colored("    [X] ", RED);
                        println!("{}", err.message);
                    }
                }
                println!();
            }

            // CONFIG
            println_colored("CONFIG", CYAN);
            print!("  debug mode:    ");
            if debug_mode {
                println_colored("on", YELLOW);
            } else {
                println_colored("off", GREEN);
            }

            if let Some(ref cfg) = config {
                print!("  auto helpers:  ");
                println_colored(if cfg.auto_install_helpers { "on" } else { "off" },
                    if cfg.auto_install_helpers { GREEN } else { DIM });
            }

            // v0.3.22: Show model and URL from config snapshot
            if !status.config_snapshot.ollama_model.is_empty() {
                print!("  model:         ");
                println_colored(&status.config_snapshot.ollama_model, DIM);
            }
            println!();
        }
        Err(e) => {
            print_colored("Error: ", RED);
            println!("{}", e);
        }
    }
}

/// Format seconds as human-readable duration
fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else if secs < 86400 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}d {}h", secs / 86400, (secs % 86400) / 3600)
    }
}

/// Format RFC3339 timestamp as "X ago"
fn format_time_ago(rfc3339: &str) -> String {
    use chrono::{DateTime, Utc};
    if let Ok(dt) = DateTime::parse_from_rfc3339(rfc3339) {
        let now = Utc::now();
        let diff = now.signed_duration_since(dt.with_timezone(&Utc));
        let secs = diff.num_seconds();
        if secs < 0 {
            return "just now".to_string();
        }
        format_duration(secs as u64) + " ago"
    } else {
        rfc3339.to_string()
    }
}

/// Get user groups
fn get_user_groups() -> String {
    std::process::Command::new("groups")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Get list of helpers and whether they were installed by Anna
fn get_helpers_list() -> Vec<(String, bool)> {
    // v0.3.32: Use system path for installed_deps.txt
    let deps_path = anna_shared::paths::paths().installed_deps_file();

    let anna_installed: std::collections::HashSet<String> = if deps_path.exists() {
        std::fs::read_to_string(&deps_path)
            .ok()
            .map(|c| c.lines().filter(|l| !l.is_empty()).map(|s| s.to_string()).collect())
            .unwrap_or_default()
    } else {
        std::collections::HashSet::new()
    };

    let tools = ["nethogs", "iotop", "htop", "lsof", "strace", "bc", "jq", "yq", "fzf"];
    let mut result = Vec::new();

    for tool in tools {
        if std::process::Command::new("which")
            .arg(tool)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            let by_anna = anna_installed.contains(tool);
            result.push((tool.to_string(), by_anna));
        }
    }

    result
}

/// Print a single dialogue step
fn print_step_internal(step: &anna_shared::rpc::DialogueStep, force_final_answer: bool) {
    let debug = is_debug_mode();

    match step.step_type {
        // ALWAYS VISIBLE
        StepType::UserQuestion => {
            print_colored("You: ", CYAN);
            println!("{}", step.content);
            println!();
        }
        StepType::FinalAnswer => {
            if !step.content.is_empty() || force_final_answer {
                println!();
                print_colored("Anna: ", GREEN);
                if force_final_answer {
                    println!();
                }
                println!("{}", step.content);
                println!();
            }
        }
        StepType::ClarificationQuestion => {
            print_colored("Anna: ", YELLOW);
            println!("{}", step.content);
            println!();
        }
        StepType::ClarificationResponse => {
            print_colored("You: ", CYAN);
            println!("{}", step.content);
            println!();
        }
        StepType::IntentClassifying => {
            if debug {
                println_colored("  understanding question...", DIM);
            }
        }
        StepType::UnderstandingCheck => {
            print_colored("Anna: ", CYAN);
            println!("{}", step.content);
        }
        StepType::ConfirmationRequest => {
            println!();
            print_colored("Anna: ", YELLOW);
            println!("Please confirm:");
            for line in step.content.lines() {
                println!("  {}", line);
            }
            println!();
        }
        StepType::MissingInfo => {
            print_colored("Anna: ", RED);
            println!("Missing information:");
            for line in step.content.lines() {
                println!("  - {}", line);
            }
        }
        StepType::SystemAlert => {
            println!();
            println_colored("SYSTEM ALERT", YELLOW);
            for line in step.content.lines() {
                println!("  {}", line);
            }
            println!();
        }
        StepType::LlmError => {
            if debug {
                print_colored("Error: ", RED);
                if let Ok(ctx) =
                    serde_json::from_str::<anna_shared::rpc::LlmErrorContext>(&step.content)
                {
                    println!("{}", ctx.message);
                } else {
                    println!("{}", step.content);
                }
            } else {
                print_colored("  [X] ", RED);
                if let Ok(ctx) =
                    serde_json::from_str::<anna_shared::rpc::LlmErrorContext>(&step.content)
                {
                    println_colored(&ctx.message, RED);
                } else {
                    println_colored("An error occurred", RED);
                }
            }
            println!();
        }
        // Team dialogue (always visible - fly on the wall)
        StepType::TicketCreated => {
            println!();
            print_colored("Ticket ", CYAN);
            println_colored(&step.content, WHITE);
        }
        // v0.3.30: Use plain text instead of emojis
        StepType::TeamAssignment => {
            print_colored("Anna -> ", MAGENTA);
            println!("{}", step.content);
        }
        StepType::TeamDialogue => {
            println!("  {}", step.content);
        }
        StepType::TeamEscalation => {
            println!();
            print_colored("  [^] Escalating: ", YELLOW);
            println!("{}", step.content);
        }
        // v0.2.9: Team dispatch and specialist working
        StepType::TeamDispatch => {
            print_colored("  ", DIM);
            println!("{}", step.content);
        }
        StepType::SpecialistWorking => {
            print_colored("  ", DIM);
            println_colored(&step.content, CYAN);
        }

        // v0.3.29: Investigator mode (always visible - explicit entry/exit)
        StepType::InvestigationStart => {
            println!();
            print_colored("INVESTIGATING: ", CYAN);
            println!("{}", step.content);
        }
        StepType::InvestigationHypothesis => {
            print_colored("  Hypothesis: ", DIM);
            println!("{}", step.content);
        }
        StepType::InvestigationProbe => {
            print_colored("  Probe: ", DIM);
            println_colored(&step.content, CYAN);
        }
        StepType::InvestigationResult => {
            if debug {
                print_colored("    -> ", DIM);
                println!("{}", step.content);
            }
        }
        StepType::InvestigationComplete => {
            println!();
            print_colored("INVESTIGATION COMPLETE: ", GREEN);
            println!("{}", step.content);
        }
        StepType::ExperimentStart => {
            println!();
            print_colored("EXPERIMENT: ", MAGENTA);
            println!("{}", step.content);
        }
        StepType::ExperimentResult => {
            print_colored("  Result: ", DIM);
            println!("{}", step.content);
        }

        // DEBUG ONLY
        StepType::AnnaToLlm => {
            if debug {
                println_colored("  [prompt to LLM]", DIM);
            }
        }
        StepType::LlmCommands => {
            if debug {
                println_colored("  [LLM response]", DIM);
                if step.content != "NONE" && step.content != "DONE" {
                    for line in step.content.lines() {
                        let line = line.trim();
                        if !line.is_empty() {
                            print_colored("    $ ", DIM);
                            println_colored(line, CYAN);
                        }
                    }
                }
            }
        }
        StepType::CommandExec => {
            if debug {
                print_colored("  $ ", DIM);
                println!("{}", step.content);
            }
        }
        StepType::CommandOutput => {
            if debug {
                println_colored(&format!("  {}", step.content), DIM);
            }
        }
        StepType::ValidationPrompt | StepType::ValidationResponse | StepType::FinalPrompt => {
            if debug {
                println_colored("  [internal]", DIM);
            }
        }
        StepType::WikiSearch => {
            if debug {
                println_colored("  Checking Arch Wiki...", DIM);
            }
        }
        StepType::WikiResults | StepType::WikiCommands => {
            if debug {
                println_colored("  [wiki results]", DIM);
            }
        }
        StepType::IntentResult => {
            if debug {
                println_colored(&format!("  intent: {}", step.content), DIM);
            }
        }
        StepType::SubQuestion | StepType::SubQuestionResult => {
            if debug {
                println_colored(&format!("  {}", step.content), DIM);
            }
        }
    }
}

/// Print a single dialogue step (streaming mode)
pub fn print_step(step: &anna_shared::rpc::DialogueStep) {
    print_step_internal(step, false);
}

/// Print the full dialogue
#[allow(dead_code)]
pub fn print_dialogue(result: &AskResult) {
    for step in &result.dialogue {
        print_step_internal(step, true);
    }
}

/// Print timeout error
pub fn print_timeout_error(timeout_secs: u64) {
    println!();
    println_colored("REQUEST TIMED OUT", RED);
    println!();
    println!("  The request took longer than {}s.", timeout_secs);
    println!();
    println_colored("Possible causes:", YELLOW);
    println!("  - Ollama model is loading (first query is slow)");
    println!("  - Complex question requiring many iterations");
    println!("  - LLM server is overloaded");
    println!();
    println_colored("Try:", GREEN);
    println!("  - Run again - model may be loaded now");
    println!("  - Check: annactl status");
    println!();
}

/// Flush stdout
pub fn flush_stdout() {
    io::stdout().flush().ok();
}

/// Show proactive alerts from monitoring system
pub fn show_proactive_alerts() -> bool {
    let store = match IssueStore::load() {
        Ok(s) => s,
        Err(_) => return false,
    };

    let critical: Vec<_> = store.active_issues.iter()
        .filter(|i| i.severity == Severity::Critical && !i.acknowledged)
        .collect();
    let warnings: Vec<_> = store.active_issues.iter()
        .filter(|i| i.severity == Severity::Warning && !i.acknowledged)
        .collect();

    if critical.is_empty() && warnings.is_empty() {
        return false;
    }

    println!();

    if !critical.is_empty() {
        println_colored("Issues detected:", YELLOW);
        println!();
        for issue in &critical {
            print_colored("  [X] ", RED);
            println!("{}", issue.summary);
            if let Some(ref fix) = issue.suggested_fix {
                println_colored(&format!("      -> {}", fix), DIM);
            }
        }
    }

    if !warnings.is_empty() {
        if critical.is_empty() {
            println_colored("Heads up:", YELLOW);
            println!();
        }
        for issue in warnings.iter().take(3) {
            print_colored("  [!] ", YELLOW);
            println!("{}", issue.summary);
        }
        if warnings.len() > 3 {
            println_colored(&format!("      ... and {} more", warnings.len() - 3), DIM);
        }
    }

    println!();
    true
}

/// Mark alerts as notified after showing them
pub fn mark_alerts_shown() {
    if let Ok(mut store) = IssueStore::load() {
        store.mark_notified();
        let _ = store.save();
    }
}

/// Print comprehensive stats (full RPG system per spec)
/// v0.3.10: Added detailed flag for expanded information
/// v0.3.20: Updated to match spec requirements
pub fn print_stats(detailed: bool) {
    // v0.3.32: Use system paths for all state
    let mem_path = memory_path();
    let p = anna_shared::paths::paths();

    println!();
    println_colored("ANNA STATISTICS", BOLD);
    println!();

    // PROGRESSION - RPG stats
    let stats = PersistentStats::load().unwrap_or_default();
    let rpg = stats.get_rpg_stats();

    println_colored("PROGRESSION", CYAN);
    print!("  title:         ");
    println_colored(&format!("\"{}\"", rpg.title), MAGENTA);

    print!("  xp:            ");
    println!("{}", rpg.xp_bar());
    println!();

    // REQUESTS
    println_colored("REQUESTS", CYAN);
    println!("  total:         {}", rpg.total_questions);

    // Solved alone (instant + memory, without LLM)
    let solved_alone = rpg.instant_answers + rpg.memory_answers;
    if rpg.total_questions > 0 {
        print!("  solved alone:  ");
        let alone_pct = solved_alone as f64 / rpg.total_questions as f64 * 100.0;
        print_colored(&format!("{}", solved_alone), if alone_pct > 50.0 { GREEN } else { DIM });
        println_colored(&format!(" ({:.0}%)", alone_pct), DIM);
    }

    // Breakdown
    if detailed && rpg.total_questions > 0 {
        print!("    instant:     ");
        let instant_pct = rpg.instant_answers as f64 / rpg.total_questions as f64 * 100.0;
        print_colored(&format!("{} ({:.0}%)", rpg.instant_answers, instant_pct), if instant_pct > 50.0 { GREEN } else { DIM });
        println!();
        print!("    memory:      ");
        let memory_pct = rpg.memory_answers as f64 / rpg.total_questions as f64 * 100.0;
        print_colored(&format!("{} ({:.0}%)", rpg.memory_answers, memory_pct), DIM);
        println!();
        print!("    llm:         ");
        let llm_pct = rpg.llm_answers as f64 / rpg.total_questions as f64 * 100.0;
        print_colored(&format!("{} ({:.0}%)", rpg.llm_answers, llm_pct), DIM);
        println!();
    }
    println!();

    // PERFORMANCE
    println_colored("PERFORMANCE", CYAN);

    // Reliability (always shown)
    print!("  reliability:   ");
    let rel_pct = rpg.reliability * 100.0;
    let rel_color = if rel_pct >= 95.0 { GREEN } else if rel_pct >= 80.0 { YELLOW } else { RED };
    println_colored(&format!("{:.1}%", rel_pct), rel_color);

    // Response times
    if rpg.avg_response_ms > 0 {
        print!("  avg response:  ");
        let avg_color = if rpg.avg_response_ms < 100 { GREEN } else if rpg.avg_response_ms < 1000 { YELLOW } else { DIM };
        println_colored(&format!("{}ms", rpg.avg_response_ms), avg_color);
    }

    if detailed {
        if rpg.fastest_response_ms > 0 {
            print!("  fastest:       ");
            println_colored(&format!("{}ms", rpg.fastest_response_ms), GREEN);
        }

        if rpg.slowest_response_ms > 0 {
            print!("  slowest:       ");
            println_colored(&format!("{}ms", rpg.slowest_response_ms), DIM);
        }
    }
    println!();

    // LEARNING
    println_colored("LEARNING", CYAN);

    let (exp_count, pattern_count, cluster_count, memory_hits, memory_misses) = load_memory_stats(&mem_path);

    println!("  recipes:       {} learned", rpg.recipes_learned);
    println!("  experiences:   {}", exp_count);
    println!("  patterns:      {}", pattern_count);

    if detailed {
        println!("  clusters:      {}", cluster_count);

        // Memory hit rate
        let total_queries = memory_hits + memory_misses;
        if total_queries > 0 {
            let hit_rate = memory_hits as f64 / total_queries as f64 * 100.0;
            print!("  memory hits:   ");
            let rate_color = if hit_rate >= 50.0 { GREEN } else if hit_rate >= 25.0 { YELLOW } else { DIM };
            print_colored(&format!("{:.1}%", hit_rate), rate_color);
            println_colored(&format!(" ({}/{})", memory_hits, total_queries), DIM);
        }
    }
    println!();

    // TICKET METRICS
    let tickets_path = p.tickets_file();
    if tickets_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&tickets_path) {
            if let Ok(store) = serde_json::from_str::<serde_json::Value>(&content) {
                println_colored("TICKETS", CYAN);

                let total_resolved = store.get("total_resolved").and_then(|v| v.as_u64()).unwrap_or(0);
                let total_failed = store.get("total_failed").and_then(|v| v.as_u64()).unwrap_or(0);
                let total_escalated = store.get("total_escalated").and_then(|v| v.as_u64()).unwrap_or(0);

                // v0.3.29: Tickets by final state
                println_colored("  by state:", DIM);
                print!("    resolved:    ");
                println_colored(&format!("{}", total_resolved), GREEN);
                print!("    failed:      ");
                println_colored(&format!("{}", total_failed), if total_failed > 0 { RED } else { DIM });
                print!("    escalated:   ");
                println_colored(&format!("{}", total_escalated), if total_escalated > 0 { YELLOW } else { DIM });

                if total_resolved > 0 || total_failed > 0 {
                    let success_rate = total_resolved as f64 / (total_resolved + total_failed).max(1) as f64 * 100.0;
                    print!("  success rate:  ");
                    let rate_color = if success_rate >= 90.0 { GREEN } else if success_rate >= 70.0 { YELLOW } else { RED };
                    println_colored(&format!("{:.1}%", success_rate), rate_color);
                }

                // v0.3.29: Resolution time statistics
                if let Some(tickets_arr) = store.get("tickets").and_then(|v| v.as_array()) {
                    let resolution_times: Vec<i64> = tickets_arr
                        .iter()
                        .filter_map(|t| {
                            let created = t.get("created_at")?.as_str()?;
                            let resolved = t.get("resolved_at")?.as_str()?;
                            let created_dt = chrono::DateTime::parse_from_rfc3339(created).ok()?;
                            let resolved_dt = chrono::DateTime::parse_from_rfc3339(resolved).ok()?;
                            Some((resolved_dt - created_dt).num_seconds())
                        })
                        .filter(|&s| s >= 0)
                        .collect();

                    if !resolution_times.is_empty() {
                        println!();
                        println_colored("  resolution times:", DIM);

                        // Average
                        let avg = resolution_times.iter().sum::<i64>() as f64 / resolution_times.len() as f64;
                        print!("    average:     ");
                        println_colored(&format_duration(avg as u64), if avg < 30.0 { GREEN } else if avg < 120.0 { YELLOW } else { DIM });

                        // Min (fastest)
                        if let Some(&min) = resolution_times.iter().min() {
                            print!("    fastest:     ");
                            println_colored(&format_duration(min as u64), GREEN);
                        }

                        // Max (slowest)
                        if let Some(&max) = resolution_times.iter().max() {
                            print!("    slowest:     ");
                            println_colored(&format_duration(max as u64), DIM);
                        }
                    }
                }
                println!();
            }
        }
    }

    // ACTIVITY
    if detailed {
        println_colored("ACTIVITY", CYAN);

        // v0.3.32: Use system paths
        let fix_history_path = p.fix_history_file();
        let fixes_count = count_json_array(&fix_history_path, "fixes");
        println!("  fixes applied: {}", fixes_count);

        let deps_path = p.installed_deps_file();
        let helpers_count = if deps_path.exists() {
            std::fs::read_to_string(&deps_path).ok()
                .map(|c| c.lines().filter(|l| !l.is_empty()).count())
                .unwrap_or(0)
        } else { 0 };
        println!("  helpers:       {} installed", helpers_count);

        // Total uptime
        if rpg.total_uptime_secs > 0 {
            print!("  total uptime:  ");
            println_colored(&format_duration(rpg.total_uptime_secs), DIM);
        }
        println!();
    }
}

/// Load memory statistics from file
fn load_memory_stats(path: &std::path::Path) -> (usize, usize, usize, u64, u64) {
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(memory) = serde_json::from_str::<serde_json::Value>(&content) {
                let experiences = memory.get("experiences")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                let patterns = memory.get("patterns")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                let clusters = memory.get("clusters")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                let stats = memory.get("stats");
                let hits = stats
                    .and_then(|s| s.get("memory_hits"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let misses = stats
                    .and_then(|s| s.get("memory_misses"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                return (experiences, patterns, clusters, hits, misses);
            }
        }
    }
    (0, 0, 0, 0, 0)
}

/// Load XP data from file
fn load_xp_data(path: &std::path::Path) -> (u32, u64, &'static str, f64, u64) {
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(xp) = serde_json::from_str::<serde_json::Value>(&content) {
                let level = xp.get("level").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
                let total = xp.get("total_xp").and_then(|v| v.as_u64()).unwrap_or(0);
                let tickets = xp.get("tickets_resolved").and_then(|v| v.as_u64()).unwrap_or(0);
                let title = get_title_for_level(level);
                let xp_needed = xp_for_level(level + 1);
                let xp_current = xp_for_level(level);
                let prog = if xp_needed > xp_current {
                    ((total.saturating_sub(xp_current)) as f64 / (xp_needed - xp_current) as f64 * 100.0).min(100.0)
                } else { 100.0 };
                return (level, total, title, prog, tickets);
            }
        }
    }
    (1, 0, "Helpdesk Newbie", 0.0, 0)
}

/// Count items in a JSON array field
fn count_json_array(path: &std::path::Path, field: &str) -> usize {
    if path.exists() {
        std::fs::read_to_string(path).ok()
            .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
            .and_then(|h| h.get(field).and_then(|f| f.as_array()).map(|a| a.len()))
            .unwrap_or(0)
    } else { 0 }
}

/// Print a simple progress bar (ASCII only)
#[allow(dead_code)]
fn print_progress_bar(progress: f64) {
    let width = 20;
    let filled = (progress / 100.0 * width as f64) as usize;
    print!("[");
    print_colored(&"=".repeat(filled), GREEN);
    print_colored(&"-".repeat(width - filled), DIM);
    print_colored(&format!("] {:.0}%", progress), GREEN);
}

/// Get title for level
fn get_title_for_level(level: u32) -> &'static str {
    match level {
        0..=5 => "Helpdesk Newbie",
        6..=10 => "Support Rookie",
        11..=15 => "Tech Apprentice",
        16..=20 => "Junior Analyst",
        21..=30 => "IT Assistant",
        31..=40 => "Tech Support Pro",
        41..=50 => "System Expert",
        51..=60 => "Tech Guru",
        61..=70 => "System Master",
        71..=80 => "Tech Wizard",
        81..=90 => "IT Sage",
        91..=99 => "System Overlord",
        100 => "The One Who Knows All",
        _ => "Unknown Entity",
    }
}

/// Calculate XP needed for level
fn xp_for_level(level: u32) -> u64 {
    let base = 100.0;
    let xp = base * (level as f64).powf(1.5) + (level as f64 * 50.0);
    xp as u64
}
