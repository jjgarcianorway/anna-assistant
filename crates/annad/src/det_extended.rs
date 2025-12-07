//! Extended deterministic answer functions (v0.0.77+).
//!
//! This module contains deterministic answer functions added in v0.0.77 and later,
//! extracted from deterministic.rs for modularization (file size limit).

use anna_shared::rpc::ProbeResult;

use crate::deterministic::DeterministicResult;
use crate::parsers::find_probe;

// === v0.0.77: New query class handlers ===

/// Answer meta/small-talk queries with static responses (bypass LLM)
pub fn answer_meta_small_talk(query: &str, route_class: &str) -> DeterministicResult {
    let q = query.to_lowercase();

    let answer = if q.contains("how are you") {
        "I'm functioning well! Ready to help with your Linux system questions."
    } else if q.contains("what is your name") || q.contains("what's your name") || q.contains("who are you") {
        "I'm Anna, your Linux system assistant. I help answer questions about your computer's hardware, software, and configuration."
    } else if q.contains("are you ok") || q.contains("are you okay") {
        "Yes, I'm operational and ready to assist with your system questions."
    } else if q.contains("are you using llm") || q.contains("are you an ai") || q.contains("are you a bot") {
        "Yes, I use an LLM (Large Language Model) to understand questions and generate responses. I combine this with deterministic probes to gather accurate system information."
    } else if q.contains("are you human") || q.contains("are you real") {
        "I'm an AI assistant - not human, but designed to help you with Linux system administration tasks."
    } else if q == "hello" || q == "hi" || q == "hey" {
        "Hello! I'm Anna, your Linux system assistant. How can I help you today?"
    } else if q == "thanks" || q == "thank you" {
        "You're welcome! Let me know if you have more questions."
    } else if q.starts_with("good morning") || q.starts_with("good afternoon") || q.starts_with("good evening") {
        "Hello! How can I help you with your system today?"
    } else {
        "Hello! I'm Anna, ready to help with your Linux system questions."
    };

    DeterministicResult {
        answer: answer.to_string(),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    }
}

/// Answer kernel version query using uname probe
pub fn answer_kernel_version(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "uname")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return None;
    }

    // Parse uname -a output: "Linux hostname 6.6.1-arch1-1 #1 SMP PREEMPT_DYNAMIC..."
    let parts: Vec<&str> = output.split_whitespace().collect();
    let answer = if parts.len() >= 3 {
        format!("Kernel version: {} ({})", parts[2], parts[0])
    } else {
        format!("Kernel: {}", output)
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer config file location queries using known paths
pub fn answer_config_file_location(query: &str, route_class: &str) -> Option<DeterministicResult> {
    let q = query.to_lowercase();

    // Config file location mappings
    let answer = if q.contains("vim") && !q.contains("nvim") {
        "Vim config: `~/.vimrc` or `~/.vim/vimrc`"
    } else if q.contains("nvim") || q.contains("neovim") {
        "Neovim config: `~/.config/nvim/init.lua` or `~/.config/nvim/init.vim`"
    } else if q.contains("hyprland") {
        "Hyprland config: `~/.config/hypr/hyprland.conf`"
    } else if q.contains("sway") {
        "Sway config: `~/.config/sway/config`"
    } else if q.contains("alacritty") {
        "Alacritty config: `~/.config/alacritty/alacritty.toml` or `~/.config/alacritty/alacritty.yml`"
    } else if q.contains("kitty") {
        "Kitty config: `~/.config/kitty/kitty.conf`"
    } else if q.contains("bash") {
        "Bash config: `~/.bashrc` (interactive) or `~/.bash_profile` (login shell)"
    } else if q.contains("zsh") {
        "Zsh config: `~/.zshrc` (main config) or `~/.zshenv` (environment)"
    } else if q.contains("fish") {
        "Fish config: `~/.config/fish/config.fish`"
    } else if q.contains("nano") {
        "Nano config: `~/.nanorc` or `~/.config/nano/nanorc`"
    } else if q.contains("emacs") {
        "Emacs config: `~/.emacs` or `~/.emacs.d/init.el`"
    } else if q.contains("git") {
        "Git config: `~/.gitconfig` (global) or `.git/config` (per-repo)"
    } else if q.contains("ssh") {
        "SSH config: `~/.ssh/config` (client) or `/etc/ssh/sshd_config` (server)"
    } else if q.contains("waybar") {
        "Waybar config: `~/.config/waybar/config` and `~/.config/waybar/style.css`"
    } else if q.contains("rofi") {
        "Rofi config: `~/.config/rofi/config.rasi`"
    } else if q.contains("dunst") {
        "Dunst config: `~/.config/dunst/dunstrc`"
    } else if q.contains("picom") {
        "Picom config: `~/.config/picom/picom.conf` or `~/.config/picom.conf`"
    } else if q.contains("i3") {
        "i3 config: `~/.config/i3/config` or `~/.i3/config`"
    } else if q.contains("polybar") {
        "Polybar config: `~/.config/polybar/config.ini`"
    } else if q.contains("tmux") {
        "Tmux config: `~/.tmux.conf`"
    } else {
        return None; // Unknown config location
    };

    Some(DeterministicResult {
        answer: answer.to_string(),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

// === v0.0.111: Ticket and Staff query handlers ===

/// Answer ticket history query - shows support desk activity summary (v0.0.116: includes inbox)
pub fn answer_ticket_history(route_class: &str) -> DeterministicResult {
    use anna_shared::ticket_tracker::TicketTracker;
    use anna_shared::email::inbox_path;

    let tracker = TicketTracker::for_user();
    let mut answer = String::new();

    // Check for open tickets
    let open_tickets = tracker.open_tickets().unwrap_or_default();
    let recent_tickets = tracker.recent(5).unwrap_or_default();

    // Check inbox
    let inbox = inbox_path();
    let inbox_count = if inbox.exists() {
        std::fs::read_to_string(&inbox)
            .map(|content| {
                content.lines()
                    .filter(|line| {
                        let trimmed = line.trim();
                        !trimmed.is_empty() && !trimmed.starts_with('#')
                    })
                    .count()
            })
            .unwrap_or(0)
    } else {
        0
    };

    // Build answer based on what we found
    if !open_tickets.is_empty() {
        answer.push_str(&format!("**Open Tickets ({}):**\n", open_tickets.len()));
        for ticket in open_tickets.iter().take(5) {
            // Show full query, no truncation
            answer.push_str(&format!("- {} ({})\n  {}\n",
                ticket.case_number,
                ticket.status,
                ticket.query));
        }
        answer.push('\n');
    }

    if inbox_count > 0 {
        answer.push_str(&format!("**Inbox:** {} pending {}\n",
            inbox_count,
            if inbox_count == 1 { "query" } else { "queries" }));
        answer.push_str("  Location: ~/.anna/inbox\n\n");
    }

    if open_tickets.is_empty() && inbox_count == 0 {
        answer.push_str("No open tickets or pending queries.\n\n");
    }

    // Add recent history if available
    if !recent_tickets.is_empty() && open_tickets.is_empty() {
        answer.push_str("**Recent Tickets:**\n");
        for ticket in recent_tickets.iter().take(3) {
            // Show full query, no truncation
            answer.push_str(&format!("- {} ({})\n  {}\n",
                ticket.case_number,
                ticket.status,
                ticket.query));
        }
        answer.push('\n');
    }

    // Add workflow explanation
    answer.push_str("**How it works:**\n");
    answer.push_str("1. Ask me a question (immediate) or drop it in ~/.anna/inbox (async)\n");
    answer.push_str("2. I create a support ticket and assign the right team\n");
    answer.push_str("3. You get a verified answer with reliability score\n\n");
    answer.push_str("To reply to a ticket: `annactl reply CN-XXXX \"your message\"`");

    DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: open_tickets.len() + inbox_count,
        route_class: route_class.to_string(),
    }
}

/// Answer staff roster query - shows who is on shift
pub fn answer_staff_roster(route_class: &str) -> DeterministicResult {
    use anna_shared::roster::all_persons;

    let all = all_persons();
    let on_shift: Vec<_> = all.iter().filter(|p| p.is_on_shift()).collect();
    let off_shift_count = all.len() - on_shift.len();

    let mut answer = String::from("**IT Department Staff**\n\n");
    answer.push_str(&format!("Currently on shift ({}):\n", on_shift.len()));

    // Group by team for cleaner display
    let mut teams: std::collections::HashMap<String, Vec<_>> = std::collections::HashMap::new();
    for person in &on_shift {
        teams.entry(person.team.to_string())
            .or_default()
            .push(person);
    }

    // Sort teams alphabetically
    let mut team_names: Vec<_> = teams.keys().cloned().collect();
    team_names.sort();

    for team_name in team_names {
        if let Some(members) = teams.get(&team_name) {
            answer.push_str(&format!("\n{} Team:\n", team_name));
            for person in members {
                let specs = if person.specializations.is_empty() {
                    String::new()
                } else {
                    format!(" - {}", person.specializations.join(", "))
                };
                answer.push_str(&format!("  {} ({}){}\n",
                    person.display_name, person.role_title, specs));
            }
        }
    }

    if off_shift_count > 0 {
        answer.push_str(&format!("\n{} staff members are currently off shift.", off_shift_count));
    }

    DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: on_shift.len(),
        route_class: route_class.to_string(),
    }
}

// === v0.0.122: New query class handlers ===

/// Answer package updates query using checkupdates probe
pub fn answer_package_updates(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "checkupdates")
        .or_else(|| find_probe(probes, "pacman"));

    // No probe result - can't answer
    let probe = probe?;

    let output = probe.stdout.trim();

    // Empty output or error means no updates
    if output.is_empty() || probe.exit_code != 0 {
        return Some(DeterministicResult {
            answer: "No package updates available. Your system is up to date.".to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    // Count updates (each line is a package)
    let update_count = output.lines().count();

    // Show first few updates
    let preview: Vec<&str> = output.lines().take(5).collect();
    let preview_str = preview.join("\n  ");

    let answer = if update_count == 1 {
        format!("1 package update available:\n  {}", preview_str)
    } else if update_count <= 5 {
        format!("{} package updates available:\n  {}", update_count, preview_str)
    } else {
        format!("{} package updates available:\n  {}\n  ...and {} more",
            update_count, preview_str, update_count - 5)
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: update_count,
        route_class: route_class.to_string(),
    })
}

/// Answer swap info query using free probe
pub fn answer_swap_info(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "free")?;
    if probe.exit_code != 0 {
        return None;
    }

    // Parse free -h output for swap line
    for line in probe.stdout.lines() {
        if line.starts_with("Swap:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let total = parts[1];
                let used = parts[2];
                let free = parts[3];

                let answer = format!(
                    "Swap: {} total, {} used, {} free",
                    total, used, free
                );

                return Some(DeterministicResult {
                    answer,
                    grounded: true,
                    parsed_data_count: 1,
                    route_class: route_class.to_string(),
                });
            }
        }
    }

    // No swap line found - might mean no swap configured
    Some(DeterministicResult {
        answer: "No swap space is configured on this system.".to_string(),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer timezone info query using timedatectl probe
pub fn answer_timezone_info(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "timedatectl")?;
    if probe.exit_code != 0 {
        return None;
    }

    let mut timezone = String::new();
    let mut local_time = String::new();
    let mut ntp_status = String::new();

    for line in probe.stdout.lines() {
        let line = line.trim();
        if line.starts_with("Time zone:") {
            timezone = line.strip_prefix("Time zone:").unwrap_or("").trim().to_string();
        } else if line.starts_with("Local time:") {
            local_time = line.strip_prefix("Local time:").unwrap_or("").trim().to_string();
        } else if line.starts_with("NTP service:") || line.starts_with("System clock synchronized:") {
            ntp_status = line.to_string();
        }
    }

    let mut answer = String::new();
    if !timezone.is_empty() {
        answer.push_str(&format!("Timezone: {}\n", timezone));
    }
    if !local_time.is_empty() {
        answer.push_str(&format!("Local time: {}\n", local_time));
    }
    if !ntp_status.is_empty() {
        answer.push_str(&ntp_status);
    }

    if answer.is_empty() {
        return None;
    }

    Some(DeterministicResult {
        answer: answer.trim().to_string(),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer system uptime query using uptime probe
pub fn answer_system_uptime(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "uptime")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return None;
    }

    // uptime -p gives output like "up 2 days, 3 hours, 45 minutes"
    let answer = format!("System has been {}", output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

// === v0.0.123: New query class handlers ===

/// Answer logged in users query using who command
pub fn answer_logged_in_users(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "who")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No users currently logged in.".to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    // Parse who output - each line is a user session
    let sessions: Vec<&str> = output.lines().collect();
    let user_count = sessions.len();

    // Get unique users
    let unique_users: std::collections::HashSet<&str> = sessions.iter()
        .filter_map(|line| line.split_whitespace().next())
        .collect();

    let answer = if unique_users.len() == 1 && user_count == 1 {
        format!("1 user logged in: {}", unique_users.iter().next().unwrap_or(&"unknown"))
    } else if unique_users.len() == 1 {
        format!("{} sessions for user: {}", user_count, unique_users.iter().next().unwrap_or(&"unknown"))
    } else {
        format!("{} users logged in ({} sessions): {}",
            unique_users.len(),
            user_count,
            unique_users.into_iter().collect::<Vec<_>>().join(", "))
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: user_count,
        route_class: route_class.to_string(),
    })
}

/// Answer battery status query using upower or /sys
pub fn answer_battery_status(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "battery")?;

    let output = probe.stdout.trim();

    // Check if no battery (command failed or empty output)
    if output.is_empty() || probe.exit_code != 0 {
        return Some(DeterministicResult {
            answer: "No battery detected. This may be a desktop system.".to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    // Try to parse upower output first
    if output.contains("percentage:") {
        let mut percentage = String::new();
        let mut state = String::new();
        let mut time_to_empty = String::new();
        let mut time_to_full = String::new();

        for line in output.lines() {
            let line = line.trim();
            if line.starts_with("percentage:") {
                percentage = line.strip_prefix("percentage:").unwrap_or("").trim().to_string();
            } else if line.starts_with("state:") {
                state = line.strip_prefix("state:").unwrap_or("").trim().to_string();
            } else if line.starts_with("time to empty:") {
                time_to_empty = line.strip_prefix("time to empty:").unwrap_or("").trim().to_string();
            } else if line.starts_with("time to full:") {
                time_to_full = line.strip_prefix("time to full:").unwrap_or("").trim().to_string();
            }
        }

        let mut answer = format!("Battery: {}", percentage);
        if !state.is_empty() {
            answer.push_str(&format!(" ({})", state));
        }
        if !time_to_empty.is_empty() {
            answer.push_str(&format!("\nTime remaining: {}", time_to_empty));
        }
        if !time_to_full.is_empty() {
            answer.push_str(&format!("\nTime to full: {}", time_to_full));
        }

        return Some(DeterministicResult {
            answer,
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    // Fallback: raw percentage from /sys
    if let Ok(pct) = output.parse::<u32>() {
        let status = if pct > 80 { "Good" } else if pct > 20 { "OK" } else { "Low" };
        return Some(DeterministicResult {
            answer: format!("Battery: {}% ({})", pct, status),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    // Unknown format
    Some(DeterministicResult {
        answer: format!("Battery info: {}", output),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer system load query using /proc/loadavg
pub fn answer_system_load(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "load_average")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return None;
    }

    // /proc/loadavg format: "0.23 0.42 0.35 1/234 12345"
    let parts: Vec<&str> = output.split_whitespace().collect();
    if parts.len() >= 3 {
        let load1 = parts[0];
        let load5 = parts[1];
        let load15 = parts[2];

        let answer = format!(
            "System load averages:\n  1 min:  {}\n  5 min:  {}\n  15 min: {}",
            load1, load5, load15
        );

        return Some(DeterministicResult {
            answer,
            grounded: true,
            parsed_data_count: 3,
            route_class: route_class.to_string(),
        });
    }

    None
}

/// Answer last boot query using who -b
pub fn answer_last_boot(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "last_boot")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return None;
    }

    // who -b output: "         system boot  2024-01-15 10:30"
    let boot_time = output
        .strip_prefix("system boot")
        .or_else(|| output.split("system boot").nth(1))
        .map(|s| s.trim())
        .unwrap_or(output);

    let answer = format!("System last booted: {}", boot_time);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}
