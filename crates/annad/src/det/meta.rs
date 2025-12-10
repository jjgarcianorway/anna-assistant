//! Meta and small-talk answer functions (v0.0.171).
//!
//! Handles conversational queries, config file locations, ticket history, and staff roster.

use anna_shared::rpc::ProbeResult;

use crate::deterministic::DeterministicResult;
use crate::parsers::find_probe;

/// Answer meta/small-talk queries with static responses (bypass LLM)
pub fn answer_meta_small_talk(query: &str, route_class: &str) -> DeterministicResult {
    let q = query.to_lowercase();

    let answer = if q.contains("how are you") {
        "I'm functioning well! Ready to help with your Linux system questions."
    } else if q.contains("what is your name")
        || q.contains("what's your name")
        || q.contains("who are you")
    {
        "I'm Anna, your Linux system assistant. I help answer questions about your computer's hardware, software, and configuration."
    } else if q.contains("are you ok") || q.contains("are you okay") {
        "Yes, I'm operational and ready to assist with your system questions."
    } else if q.contains("are you using llm")
        || q.contains("are you an ai")
        || q.contains("are you a bot")
    {
        "Yes, I use an LLM (Large Language Model) to understand questions and generate responses. I combine this with deterministic probes to gather accurate system information."
    } else if q.contains("are you human") || q.contains("are you real") {
        "I'm an AI assistant - not human, but designed to help you with Linux system administration tasks."
    } else if q == "hello" || q == "hi" || q == "hey" {
        "Hello! I'm Anna, your Linux system assistant. How can I help you today?"
    } else if q == "thanks" || q == "thank you" {
        "You're welcome! Let me know if you have more questions."
    } else if q.starts_with("good morning")
        || q.starts_with("good afternoon")
        || q.starts_with("good evening")
    {
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
pub fn answer_kernel_version(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
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

/// Answer ticket history query - shows support desk activity summary
pub fn answer_ticket_history(route_class: &str) -> DeterministicResult {
    use anna_shared::email::inbox_path;
    use anna_shared::ticket_tracker::TicketTracker;

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
                content
                    .lines()
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
            answer.push_str(&format!(
                "- {} ({})\n  {}\n",
                ticket.case_number, ticket.status, ticket.query
            ));
        }
        answer.push('\n');
    }

    if inbox_count > 0 {
        answer.push_str(&format!(
            "**Inbox:** {} pending {}\n",
            inbox_count,
            if inbox_count == 1 { "query" } else { "queries" }
        ));
        answer.push_str("  Location: ~/.anna/inbox\n\n");
    }

    if open_tickets.is_empty() && inbox_count == 0 {
        answer.push_str("No open tickets or pending queries.\n\n");
    }

    // Add recent history if available
    if !recent_tickets.is_empty() && open_tickets.is_empty() {
        answer.push_str("**Recent Tickets:**\n");
        for ticket in recent_tickets.iter().take(3) {
            answer.push_str(&format!(
                "- {} ({})\n  {}\n",
                ticket.case_number, ticket.status, ticket.query
            ));
        }
        answer.push('\n');
    }

    // Add workflow explanation
    answer.push_str("**How it works:**\n");
    answer.push_str("1. Ask me a question (immediate) or drop it in ~/.anna/inbox (async)\n");
    answer.push_str("2. I create a support ticket and assign the right team\n");
    answer.push_str("3. You get a verified answer with reliability score\n\n");
    answer.push_str("To continue a conversation, just ask me about that ticket.");

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
        teams
            .entry(person.team.to_string())
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
                answer.push_str(&format!(
                    "  {} ({}){}\n",
                    person.display_name, person.role_title, specs
                ));
            }
        }
    }

    if off_shift_count > 0 {
        answer.push_str(&format!(
            "\n{} staff members are currently off shift.",
            off_shift_count
        ));
    }

    DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: on_shift.len(),
        route_class: route_class.to_string(),
    }
}
