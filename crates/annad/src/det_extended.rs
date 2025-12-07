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

// === v0.0.124: New query class handlers ===

/// Answer hostname query using hostname command
pub fn answer_hostname(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "hostname")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return None;
    }

    let answer = format!("Hostname: {}", output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer OS info query using /etc/os-release
pub fn answer_os_info(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "os_release")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return None;
    }

    let mut name = String::new();
    let mut version = String::new();
    let mut pretty_name = String::new();

    for line in output.lines() {
        if line.starts_with("PRETTY_NAME=") {
            pretty_name = line.strip_prefix("PRETTY_NAME=")
                .unwrap_or("")
                .trim_matches('"')
                .to_string();
        } else if line.starts_with("NAME=") {
            name = line.strip_prefix("NAME=")
                .unwrap_or("")
                .trim_matches('"')
                .to_string();
        } else if line.starts_with("VERSION=") {
            version = line.strip_prefix("VERSION=")
                .unwrap_or("")
                .trim_matches('"')
                .to_string();
        }
    }

    let answer = if !pretty_name.is_empty() {
        format!("OS: {}", pretty_name)
    } else if !name.is_empty() && !version.is_empty() {
        format!("OS: {} {}", name, version)
    } else if !name.is_empty() {
        format!("OS: {}", name)
    } else {
        return None;
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer network connectivity query using ping
pub fn answer_network_connectivity(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "ping_check")?;

    let answer = if probe.exit_code == 0 {
        // Parse ping output for latency
        let output = probe.stdout.trim();
        let latency = output.lines()
            .find(|line| line.contains("time="))
            .and_then(|line| {
                line.split("time=").nth(1)
                    .and_then(|s| s.split_whitespace().next())
            });

        if let Some(lat) = latency {
            format!("Online - ping to 8.8.8.8: {} ms", lat)
        } else {
            "Online - network connectivity confirmed".to_string()
        }
    } else {
        "Offline - cannot reach 8.8.8.8 (Google DNS)".to_string()
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer mounted filesystems query using findmnt
pub fn answer_mounted_filesystems(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "findmnt")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No mounted filesystems found.".to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    let lines: Vec<&str> = output.lines().collect();
    let mount_count = lines.len().saturating_sub(1); // Subtract header

    let answer = format!("Mounted filesystems ({}):\n{}", mount_count, output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: mount_count,
        route_class: route_class.to_string(),
    })
}

/// Answer USB devices query using lsusb
pub fn answer_usb_devices(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "lsusb")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No USB devices detected.".to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    let device_count = output.lines().count();

    // Simplify output - extract just device names
    let devices: Vec<String> = output.lines()
        .filter_map(|line| {
            // lsusb format: "Bus 001 Device 002: ID 1234:5678 Device Name"
            line.split(": ").nth(1).map(|s| {
                // Remove ID prefix
                if let Some(pos) = s.find(' ') {
                    s[pos+1..].trim().to_string()
                } else {
                    s.to_string()
                }
            })
        })
        .collect();

    let answer = if device_count <= 10 {
        format!("USB devices ({}):\n  {}", device_count, devices.join("\n  "))
    } else {
        let preview: Vec<&str> = devices.iter().take(8).map(|s| s.as_str()).collect();
        format!("USB devices ({}):\n  {}\n  ...and {} more",
            device_count, preview.join("\n  "), device_count - 8)
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: device_count,
        route_class: route_class.to_string(),
    })
}

// === v0.0.125: New query class handlers ===

/// Answer listening ports query using ss
pub fn answer_listening_ports(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "listening_ports")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No listening ports found.".to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    let lines: Vec<&str> = output.lines().collect();
    let port_count = lines.len().saturating_sub(1); // Subtract header

    let answer = format!("Listening ports ({}):\n{}", port_count, output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: port_count,
        route_class: route_class.to_string(),
    })
}

/// Answer running services query using systemctl
pub fn answer_running_services(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "running_services")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No running services found.".to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    let service_count = output.lines().count();

    // Extract just the service names
    let services: Vec<&str> = output.lines()
        .filter_map(|line| line.split_whitespace().next())
        .collect();

    let answer = if service_count <= 15 {
        format!("Running services ({}):\n  {}", service_count, services.join("\n  "))
    } else {
        let preview: Vec<&str> = services.iter().take(12).copied().collect();
        format!("Running services ({}):\n  {}\n  ...and {} more",
            service_count, preview.join("\n  "), service_count - 12)
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: service_count,
        route_class: route_class.to_string(),
    })
}

/// Answer current user query using id
pub fn answer_current_user(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "current_user")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return None;
    }

    // Parse id output: uid=1000(username) gid=1000(group) groups=...
    let mut username = String::new();
    let mut uid = String::new();
    let mut groups = Vec::new();

    for part in output.split_whitespace() {
        if part.starts_with("uid=") {
            if let Some(name) = part.split('(').nth(1) {
                username = name.trim_end_matches(')').to_string();
            }
            if let Some(id) = part.strip_prefix("uid=") {
                uid = id.split('(').next().unwrap_or("").to_string();
            }
        } else if part.starts_with("groups=") {
            let grp = part.strip_prefix("groups=").unwrap_or("");
            for g in grp.split(',') {
                if let Some(name) = g.split('(').nth(1) {
                    groups.push(name.trim_end_matches(')').to_string());
                }
            }
        }
    }

    let answer = format!("User: {} (uid={})\nGroups: {}", username, uid, groups.join(", "));

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer system architecture query using uname -m
pub fn answer_system_architecture(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "arch")?;
    if probe.exit_code != 0 {
        return None;
    }

    let arch = probe.stdout.trim();
    if arch.is_empty() {
        return None;
    }

    let desc = match arch {
        "x86_64" => "64-bit x86 (AMD64/Intel64)",
        "i686" | "i386" => "32-bit x86",
        "aarch64" => "64-bit ARM",
        "armv7l" => "32-bit ARM (ARMv7)",
        "riscv64" => "64-bit RISC-V",
        _ => arch,
    };

    let answer = format!("Architecture: {} ({})", arch, desc);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer environment variables query
pub fn answer_environment_vars(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "env_vars")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No environment variables found.".to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    let var_count = output.lines().count();

    // Show key variables first if present
    let important_vars = ["PATH", "HOME", "USER", "SHELL", "TERM", "DISPLAY", "XDG_SESSION_TYPE"];
    let mut key_vars = Vec::new();
    let mut other_count = 0;

    for line in output.lines() {
        let key = line.split('=').next().unwrap_or("");
        if important_vars.contains(&key) {
            key_vars.push(line);
        } else {
            other_count += 1;
        }
    }

    let answer = if !key_vars.is_empty() {
        format!("Environment variables ({}):\n  {}\n  ...and {} others",
            var_count, key_vars.join("\n  "), other_count)
    } else {
        format!("Environment variables ({}):\n{}", var_count, output)
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: var_count,
        route_class: route_class.to_string(),
    })
}

// === v0.0.126: New System & Network Queries ===

/// Answer process tree query
pub fn answer_process_tree(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "pstree")?;
    if probe.exit_code != 0 {
        return Some(DeterministicResult {
            answer: "pstree not available (install psmisc package)".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No process tree available.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let line_count = output.lines().count();
    let answer = format!("Process tree ({} lines):\n```\n{}\n```", line_count, output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: line_count,
        route_class: route_class.to_string(),
    })
}

/// Answer DNS servers query
pub fn answer_dns_servers(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "dns_servers")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No DNS servers configured in /etc/resolv.conf".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let mut servers = Vec::new();
    for line in output.lines() {
        if let Some(ip) = line.strip_prefix("nameserver ") {
            servers.push(ip.trim());
        }
    }

    let answer = if servers.is_empty() {
        "No DNS servers configured.".to_string()
    } else {
        format!("DNS servers: {}", servers.join(", "))
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: servers.len(),
        route_class: route_class.to_string(),
    })
}

/// Answer default gateway query
pub fn answer_default_gateway(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "default_gateway")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No default gateway configured.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    // Parse: "default via 192.168.1.1 dev eth0 proto dhcp metric 100"
    let parts: Vec<&str> = output.split_whitespace().collect();
    let gateway = parts.get(2).unwrap_or(&"unknown");
    let interface = parts.iter()
        .position(|&p| p == "dev")
        .and_then(|i| parts.get(i + 1))
        .unwrap_or(&"unknown");

    let answer = format!("Default gateway: {} (via {})", gateway, interface);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer open files count query
pub fn answer_open_files(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "open_files")?;
    if probe.exit_code != 0 {
        return Some(DeterministicResult {
            answer: "lsof not available or requires elevated permissions".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let count_str = probe.stdout.trim();
    let count: usize = count_str.parse().unwrap_or(0);

    let answer = format!("Open files: {} file descriptors system-wide", count);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer system locale query
pub fn answer_system_locale(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "locale")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No locale settings available.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    // Extract key locale values
    let mut lang = None;
    let mut lc_all = None;

    for line in output.lines() {
        if let Some(val) = line.strip_prefix("LANG=") {
            lang = Some(val.trim_matches('"'));
        }
        if let Some(val) = line.strip_prefix("LC_ALL=") {
            lc_all = Some(val.trim_matches('"'));
        }
    }

    let primary = lc_all.unwrap_or_else(|| lang.unwrap_or("not set"));
    let answer = format!("System locale: {}\n\nFull output:\n{}", primary, output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: output.lines().count(),
        route_class: route_class.to_string(),
    })
}

// === v0.0.127: Hardware & Storage Queries ===

/// Answer block devices query
pub fn answer_block_devices(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "block_devices")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No block devices found.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let device_count = output.lines().count().saturating_sub(1); // Minus header
    let answer = format!("Block devices ({}):\n```\n{}\n```", device_count, output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: device_count,
        route_class: route_class.to_string(),
    })
}

/// Answer installed kernels query
pub fn answer_installed_kernels(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "installed_kernels")?;
    if probe.exit_code != 0 && probe.stdout.is_empty() {
        return Some(DeterministicResult {
            answer: "Could not determine installed kernels.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No kernels found.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let kernel_count = output.lines().count();
    let answer = format!("Installed kernels ({}):\n{}", kernel_count, output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: kernel_count,
        route_class: route_class.to_string(),
    })
}

/// Answer CPU frequency query
pub fn answer_cpu_frequency(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "cpu_frequency")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "CPU frequency information not available.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    // Parse MHz from output like "cpu MHz		: 3600.000"
    let freq = if let Some(mhz_line) = output.lines().find(|l| l.contains("MHz")) {
        if let Some(value) = mhz_line.split(':').nth(1) {
            let mhz: f64 = value.trim().parse().unwrap_or(0.0);
            if mhz > 1000.0 {
                format!("{:.2} GHz", mhz / 1000.0)
            } else {
                format!("{:.0} MHz", mhz)
            }
        } else {
            output.to_string()
        }
    } else {
        output.to_string()
    };

    let answer = format!("CPU frequency: {}", freq);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer memory slots query
pub fn answer_memory_slots(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "memory_slots")?;

    let output = probe.stdout.trim();
    if output.contains("Requires root") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "Memory slot information requires root access (sudo dmidecode).".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let answer = format!("Memory slots:\n{}", output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: output.lines().count(),
        route_class: route_class.to_string(),
    })
}

/// Answer ZFS status query
pub fn answer_zfs_status(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "zfs_status")?;

    let output = probe.stdout.trim();
    if output.contains("not installed") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "ZFS is not installed on this system.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    if output.contains("no pools available") {
        return Some(DeterministicResult {
            answer: "ZFS is installed but no pools are configured.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let answer = format!("ZFS pool status:\n```\n{}\n```", output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

// === v0.0.128: Security & Admin Queries ===

/// Answer boot loader query
pub fn answer_boot_loader(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "boot_loader")?;

    let output = probe.stdout.trim();
    if output.contains("not detected") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "Could not detect boot loader configuration.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    // Detect boot loader type
    let loader_type = if output.contains("systemd-boot") || output.contains("Boot Loader Specification") {
        "systemd-boot"
    } else if output.contains("GRUB") || output.contains("grub") {
        "GRUB"
    } else {
        "Unknown"
    };

    let answer = format!("Boot loader: {}\n```\n{}\n```", loader_type, output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer firewall status query
pub fn answer_firewall_status(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "firewall_status")?;

    let output = probe.stdout.trim();
    if output.contains("No firewall detected") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "No active firewall detected (iptables, nftables, or ufw).".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    // Detect firewall type
    let fw_type = if output.contains("Chain") {
        "iptables"
    } else if output.contains("table") && output.contains("chain") {
        "nftables"
    } else if output.contains("Status:") {
        "ufw"
    } else {
        "Unknown"
    };

    let answer = format!("Firewall ({}): Active\n```\n{}\n```", fw_type, output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer systemd units query
pub fn answer_systemd_units(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "systemd_units")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No systemd units found.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let unit_count = output.lines().count();
    let answer = format!("Systemd units ({}):\n```\n{}\n```", unit_count, output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: unit_count,
        route_class: route_class.to_string(),
    })
}

/// Answer crontabs query
pub fn answer_crontabs(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "crontabs")?;

    let output = probe.stdout.trim();
    if output.contains("No crontab") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "No crontab entries for current user.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let job_count = output.lines().filter(|l| !l.starts_with('#') && !l.is_empty()).count();
    let answer = format!("Crontab ({} jobs):\n```\n{}\n```", job_count, output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: job_count,
        route_class: route_class.to_string(),
    })
}

/// Answer SSH connections query
pub fn answer_ssh_connections(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "ssh_connections")?;

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No active SSH connections.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let conn_count = output.lines().count();
    let answer = format!("SSH connections ({}):\n```\n{}\n```", conn_count, output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: conn_count,
        route_class: route_class.to_string(),
    })
}

// === v0.0.129: Docker & Logging Queries ===

/// Answer Docker containers query
pub fn answer_docker_containers(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "docker_containers")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "Docker is not installed or not running.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let container_count = output.lines().count().saturating_sub(1); // Minus header
    let status = if container_count == 0 {
        "No running containers."
    } else {
        ""
    };

    let answer = if container_count == 0 {
        status.to_string()
    } else {
        format!("Docker containers ({}):\n```\n{}\n```", container_count, output)
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: container_count,
        route_class: route_class.to_string(),
    })
}

/// Answer Docker images query
pub fn answer_docker_images(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "docker_images")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "Docker is not installed or not running.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let image_count = output.lines().count().saturating_sub(1); // Minus header
    let answer = if image_count == 0 {
        "No Docker images found.".to_string()
    } else {
        format!("Docker images ({}):\n```\n{}\n```", image_count, output)
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: image_count,
        route_class: route_class.to_string(),
    })
}

/// Answer systemd timers query
pub fn answer_systemd_timers(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "systemd_timers")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No systemd timers found.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let timer_count = output.lines().count();
    let answer = format!("Systemd timers ({}):\n```\n{}\n```", timer_count, output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: timer_count,
        route_class: route_class.to_string(),
    })
}

/// Answer last logins query
pub fn answer_last_logins(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "last_logins")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "Login history not available.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let login_count = output.lines().filter(|l| !l.is_empty() && !l.starts_with("wtmp")).count();
    let answer = format!("Recent logins ({}):\n```\n{}\n```", login_count, output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: login_count,
        route_class: route_class.to_string(),
    })
}

/// Answer failed logins query
pub fn answer_failed_logins(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "failed_logins")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "No failed login attempts found (or data not available).".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let failure_count = output.lines().count();
    let answer = format!("Failed login attempts ({}):\n```\n{}\n```", failure_count, output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: failure_count,
        route_class: route_class.to_string(),
    })
}

// === v0.0.130: System & Security Queries ===

/// Answer systemd journal query
pub fn answer_systemd_journal(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "systemd_journal")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "Systemd journal not available.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let line_count = output.lines().count();
    let answer = format!("Recent system logs ({} entries):\n```\n{}\n```", line_count, output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: line_count,
        route_class: route_class.to_string(),
    })
}

/// Answer network namespaces query
pub fn answer_network_namespaces(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "network_namespaces")?;

    let output = probe.stdout.trim();
    if output.contains("No network namespaces") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "No network namespaces configured.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let ns_count = output.lines().count();
    let answer = format!("Network namespaces ({}):\n{}", ns_count, output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: ns_count,
        route_class: route_class.to_string(),
    })
}

/// Answer available shells query
pub fn answer_available_shells(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "available_shells")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "Shell list not available.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let shells: Vec<&str> = output.lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .collect();

    let answer = format!("Available shells ({}):\n{}", shells.len(), shells.join("\n"));

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: shells.len(),
        route_class: route_class.to_string(),
    })
}

/// Answer sudoers info query
pub fn answer_sudoers_info(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "sudoers_info")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.is_empty() || output.contains("password is required") {
        return Some(DeterministicResult {
            answer: "Sudo access information not available (may require password).".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let answer = format!("Sudo access:\n```\n{}\n```", output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer installed desktops query
pub fn answer_installed_desktops(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "installed_desktops")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "No desktop environments detected.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let de_count = output.lines().count();
    let answer = format!("Installed desktop environments ({}):\n{}", de_count, output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: de_count,
        route_class: route_class.to_string(),
    })
}

// === v0.0.131: Virtualization and security answer functions ===

/// Answer virtualization info query
pub fn answer_virtualization_info(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "virtualization_info")?;

    let output = probe.stdout.trim();
    let answer = if output == "none" || output.is_empty() {
        "Running on bare metal (no virtualization detected).".to_string()
    } else {
        format!("Virtualization: **{}**", output)
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer SELinux status query
pub fn answer_selinux_status(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "selinux_status")?;

    let output = probe.stdout.trim();
    if output.contains("not installed") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "SELinux is not installed on this system.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let answer = format!("SELinux status:\n```\n{}\n```", output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer AppArmor status query
pub fn answer_apparmor_status(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "apparmor_status")?;

    let output = probe.stdout.trim();
    if output.contains("not installed") || output.is_empty() || output == "N" {
        return Some(DeterministicResult {
            answer: "AppArmor is not installed or not enabled on this system.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    if output == "Y" {
        return Some(DeterministicResult {
            answer: "AppArmor is **enabled** on this system.".to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    let answer = format!("AppArmor status:\n```\n{}\n```", output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer systemd slices query
pub fn answer_systemd_slices(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "systemd_slices")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "Cgroup slice information not available.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let answer = format!("Systemd cgroup slices:\n```\n{}\n```", output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer coredump list query
pub fn answer_coredump_list(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "coredump_list")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.contains("No coredumps") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "No coredumps found on this system.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let dump_count = output.lines().count().saturating_sub(1); // Subtract header line
    let answer = format!("Coredumps ({} found):\n```\n{}\n```", dump_count, output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: dump_count,
        route_class: route_class.to_string(),
    })
}

// === v0.0.132: Kernel and network answer functions ===

/// Answer kernel modules query
pub fn answer_kernel_modules(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "kernel_modules")?;

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No kernel modules information available.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let module_count = output.lines().count().saturating_sub(1); // Subtract header line
    let answer = format!("Loaded kernel modules ({}):\n```\n{}\n```", module_count, output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: module_count,
        route_class: route_class.to_string(),
    })
}

/// Answer systemd targets query
pub fn answer_systemd_targets(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "systemd_targets")?;

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No systemd targets found.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let target_count = output.lines().count();
    let answer = format!("Active systemd targets ({}):\n```\n{}\n```", target_count, output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: target_count,
        route_class: route_class.to_string(),
    })
}

/// Answer IP routes query
pub fn answer_ip_routes(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "ip_routes")?;

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No IP routes found.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let route_count = output.lines().count();
    let answer = format!("IP routing table ({} routes):\n```\n{}\n```", route_count, output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: route_count,
        route_class: route_class.to_string(),
    })
}

/// Answer ARP table query
pub fn answer_arp_table(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "arp_table")?;

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No ARP entries found.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let entry_count = output.lines().count();
    let answer = format!("ARP table ({} entries):\n```\n{}\n```", entry_count, output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: entry_count,
        route_class: route_class.to_string(),
    })
}

/// Answer iptables rules query
pub fn answer_iptables_rules(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "iptables_rules")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.contains("requires root") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "iptables rules not available (may require root privileges).".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let answer = format!("iptables rules:\n```\n{}\n```", output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}
