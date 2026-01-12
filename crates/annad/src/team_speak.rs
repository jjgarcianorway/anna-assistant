//! Hollywood IT Teams Experience
//! v0.0.998: Team-style status messages for fly-on-the-wall experience
//!
//! This module transforms technical operations into team-like dialogue,
//! making users feel like they're watching an IT department work.

use std::collections::HashMap;
use std::sync::LazyLock;

/// Transform a command into a friendly team-style description
pub fn describe_command(cmd: &str) -> String {
    let cmd_lower = cmd.to_lowercase();

    // Check common commands first
    if cmd_lower.starts_with("df ") || cmd_lower == "df" {
        return "Checking disk usage...".to_string();
    }
    if cmd_lower.starts_with("free ") || cmd_lower == "free" {
        return "Checking memory status...".to_string();
    }
    if cmd_lower.starts_with("ps ") {
        return "Looking at running processes...".to_string();
    }
    if cmd_lower.starts_with("systemctl status") {
        let service = cmd.split_whitespace().nth(2).unwrap_or("service");
        return format!("Checking {} status...", service);
    }
    if cmd_lower.starts_with("systemctl restart") {
        let service = cmd.split_whitespace().nth(2).unwrap_or("service");
        return format!("Restarting {}...", service);
    }
    if cmd_lower.starts_with("pacman -") || cmd_lower.starts_with("yay -") {
        if cmd_lower.contains("-qi") || cmd_lower.contains("-q ") {
            return "Checking installed packages...".to_string();
        }
        if cmd_lower.contains("-ss") || cmd_lower.contains("-s ") {
            return "Searching packages...".to_string();
        }
        if cmd_lower.contains("-syu") {
            return "Checking for updates...".to_string();
        }
    }
    if cmd_lower.starts_with("journalctl") {
        return "Checking system logs...".to_string();
    }
    if cmd_lower.starts_with("ip addr") || cmd_lower.starts_with("ip a") {
        return "Getting network info...".to_string();
    }
    if cmd_lower.starts_with("ss ") || cmd_lower.starts_with("netstat") {
        return "Checking network connections...".to_string();
    }
    if cmd_lower.starts_with("lsblk") {
        return "Checking block devices...".to_string();
    }
    if cmd_lower.starts_with("mount") {
        return "Checking mounted filesystems...".to_string();
    }
    if cmd_lower.starts_with("uname") {
        return "Getting kernel info...".to_string();
    }
    if cmd_lower.starts_with("cat /etc/os-release") || cmd_lower.contains("lsb_release") {
        return "Checking OS version...".to_string();
    }
    if cmd_lower.starts_with("cat /proc/cpuinfo") || cmd_lower.contains("lscpu") {
        return "Getting CPU info...".to_string();
    }
    if cmd_lower.starts_with("nvidia-smi") {
        return "Checking GPU status...".to_string();
    }
    if cmd_lower.starts_with("sensors") || cmd_lower.contains("hwinfo") {
        return "Reading hardware sensors...".to_string();
    }
    if cmd_lower.starts_with("top") || cmd_lower.starts_with("htop") {
        return "Checking system load...".to_string();
    }
    if cmd_lower.starts_with("ping ") {
        let host = cmd.split_whitespace().nth(1).unwrap_or("host");
        return format!("Testing connection to {}...", host);
    }
    if cmd_lower.starts_with("curl") || cmd_lower.starts_with("wget") {
        return "Fetching data...".to_string();
    }
    if cmd_lower.starts_with("git status") {
        return "Checking repository status...".to_string();
    }
    if cmd_lower.starts_with("git log") {
        return "Looking at commit history...".to_string();
    }
    if cmd_lower.starts_with("docker ps") {
        return "Checking Docker containers...".to_string();
    }
    if cmd_lower.starts_with("which ") || cmd_lower.starts_with("command -v") {
        let tool = cmd.split_whitespace().last().unwrap_or("tool");
        return format!("Looking for {}...", tool);
    }

    // Generic fallback - extract first word
    let first_word = cmd.split_whitespace().next().unwrap_or("command");
    format!("Running {}...", first_word)
}

/// Get a success message for completed operations
pub fn success_message(operation: &str) -> String {
    match operation {
        "restart" => "Service restarted successfully".to_string(),
        "install" => "Installation complete".to_string(),
        "backup" => "Backup created".to_string(),
        "update" => "Updates applied".to_string(),
        "fix" => "Issue resolved".to_string(),
        _ => "Done".to_string(),
    }
}

/// Get a working-on-it message for long operations
pub fn working_message(stage: &str) -> String {
    match stage {
        "thinking" => "Analyzing the situation...".to_string(),
        "searching" => "Searching for relevant information...".to_string(),
        "validating" => "Verifying the results...".to_string(),
        "preparing" => "Preparing solution...".to_string(),
        "applying" => "Applying changes...".to_string(),
        _ => "Working on it...".to_string(),
    }
}

/// Get team-style commentary for different phases
pub fn phase_commentary(phase: &str, context: Option<&str>) -> String {
    match phase {
        "intent_classify" => "Let me understand what you need...".to_string(),
        "wiki_search" => "Checking the Arch Wiki for guidance...".to_string(),
        "wiki_found" => {
            if let Some(article) = context {
                format!("Found relevant article: {}", article)
            } else {
                "Found some useful information".to_string()
            }
        }
        "commands_ready" => "Got the commands ready".to_string(),
        "executing" => "Running diagnostics...".to_string(),
        "analyzing" => "Looking at the output...".to_string(),
        "needs_more" => "Need a bit more information...".to_string(),
        "answering" => "Here's what I found".to_string(),
        "fix_offer" => "I can fix this for you".to_string(),
        "confirming" => "Just to make sure I understand...".to_string(),
        _ => String::new(),
    }
}

/// Transform technical error messages into friendlier explanations
pub fn humanize_error(error: &str) -> String {
    let error_lower = error.to_lowercase();

    if error_lower.contains("permission denied") {
        return "Looks like we need admin access for this".to_string();
    }
    if error_lower.contains("command not found") {
        return "That tool isn't installed yet".to_string();
    }
    if error_lower.contains("connection refused") {
        return "Can't connect - the service might be down".to_string();
    }
    if error_lower.contains("no such file") {
        return "File not found".to_string();
    }
    if error_lower.contains("timeout") {
        return "Taking too long - might be a network issue".to_string();
    }
    if error_lower.contains("disk full") || error_lower.contains("no space left") {
        return "Disk is full - we'll need to free up some space".to_string();
    }

    // Return original if no match
    error.to_string()
}

/// Get encouraging message for long operations
pub fn patience_message(iteration: u32) -> Option<String> {
    match iteration {
        2 => Some("Still working on it...".to_string()),
        3 => Some("This needs a bit more investigation...".to_string()),
        4 => Some("Almost there...".to_string()),
        5 => Some("Bear with me, gathering all the details...".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_describe_command() {
        assert_eq!(describe_command("df -h"), "Checking disk usage...");
        assert_eq!(describe_command("free -h"), "Checking memory status...");
        assert_eq!(describe_command("systemctl status nginx"), "Checking nginx status...");
    }

    #[test]
    fn test_humanize_error() {
        assert!(humanize_error("permission denied").contains("admin"));
        assert!(humanize_error("command not found").contains("installed"));
    }
}
