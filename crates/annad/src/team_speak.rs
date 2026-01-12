//! Hollywood IT Teams Experience
//! v0.0.998: Team-style status messages for fly-on-the-wall experience
//! v0.0.999: Full IT department specialist dialogue
//!
//! This module transforms technical operations into team-like dialogue,
//! making users feel like they're watching an IT department work.

use crate::department::{self, Specialist, SpecialistRole};

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

// =============================================================================
// v0.0.999: IT Department Specialist Dialogue
// =============================================================================

/// Generate Anna's assignment message to a specialist
pub fn anna_assigns_to(specialist: &Specialist, question: &str) -> String {
    let short_q = if question.len() > 60 {
        format!("{}...", &question[..57])
    } else {
        question.to_string()
    };

    match specialist.role {
        SpecialistRole::Junior => {
            format!(
                "Hey {}! Got a {} question for you: \"{}\"",
                specialist.name, specialist.department.to_lowercase(), short_q
            )
        }
        SpecialistRole::Senior => {
            format!(
                "{}, need your expertise on this: \"{}\"",
                specialist.name, short_q
            )
        }
        SpecialistRole::Manager => {
            format!(
                "{}, user needs help with: \"{}\"",
                specialist.name, short_q
            )
        }
    }
}

/// Generate specialist's acknowledgment of assignment
pub fn specialist_acknowledges(specialist: &Specialist) -> String {
    match specialist.role {
        SpecialistRole::Junior => {
            let responses = [
                format!("On it! Let me check a few things..."),
                format!("Sure thing! Running some diagnostics..."),
                format!("Got it! Let me look into this..."),
                format!("Checking now..."),
            ];
            responses[specialist.name.len() % responses.len()].clone()
        }
        SpecialistRole::Senior => {
            let responses = [
                format!("Looking into it. Give me a moment..."),
                format!("I'll investigate. Running deep diagnostics..."),
                format!("Let me check the logs and configs..."),
            ];
            responses[specialist.name.len() % responses.len()].clone()
        }
        SpecialistRole::Manager => {
            format!("I'll coordinate with the team on this.")
        }
    }
}

/// Generate specialist's working message based on operation
pub fn specialist_working(specialist: &Specialist, operation: &str) -> String {
    let op = operation.to_lowercase();

    if op.contains("check") || op.contains("look") {
        format!("{}: Checking...", specialist.name)
    } else if op.contains("search") {
        format!("{}: Searching...", specialist.name)
    } else if op.contains("run") || op.contains("exec") {
        format!("{}: Running diagnostics...", specialist.name)
    } else if op.contains("analyz") {
        format!("{}: Analyzing output...", specialist.name)
    } else {
        format!("{}: Working on it...", specialist.name)
    }
}

/// Generate escalation message from junior to senior
pub fn escalation_request(junior: &Specialist, senior: &Specialist, reason: &str) -> String {
    format!(
        "{} → {}: This one's a bit complex. {}. Can you take a look?",
        junior.name, senior.name, reason
    )
}

/// Generate senior accepting escalation
pub fn senior_accepts_escalation(senior: &Specialist) -> String {
    let responses = [
        format!("{}: I'll take it from here.", senior.name),
        format!("{}: Let me dig deeper into this.", senior.name),
        format!("{}: Good call escalating. Looking now...", senior.name),
    ];
    responses[senior.name.len() % responses.len()].clone()
}

/// Generate specialist's finding message
pub fn specialist_found_something(specialist: &Specialist, finding: &str) -> String {
    match specialist.role {
        SpecialistRole::Junior => {
            format!("{}: Found something! {}", specialist.name, finding)
        }
        SpecialistRole::Senior => {
            format!("{}: Here's what I found - {}", specialist.name, finding)
        }
        SpecialistRole::Manager => {
            format!("{}: Analysis complete. {}", specialist.name, finding)
        }
    }
}

/// Generate specialist reporting back to Anna
pub fn specialist_reports_to_anna(specialist: &Specialist, summary: &str) -> String {
    format!(
        "{} → Anna: {}",
        specialist.name, summary
    )
}

/// Generate Anna thanking the specialist
pub fn anna_thanks_specialist(specialist: &Specialist) -> String {
    match specialist.role {
        SpecialistRole::Junior => {
            format!("Thanks {}! Good work.", specialist.name)
        }
        SpecialistRole::Senior => {
            format!("Thanks {}. That was thorough.", specialist.name)
        }
        SpecialistRole::Manager => {
            format!("Appreciate it, {}.", specialist.name)
        }
    }
}

/// Get the specialist for a question and generate assignment
pub fn dispatch_question(question: &str) -> Option<(String, &'static Specialist)> {
    let specialist = department::get_specialist_for_topic(question)?;
    let msg = anna_assigns_to(specialist, question);
    Some((msg, specialist))
}

/// Generate a ticket opened message
pub fn ticket_opened(case_number: &str, department: &str) -> String {
    format!("Ticket {} opened → {} department", case_number, department)
}

/// Generate full dialogue for command execution
pub fn command_dialogue(specialist: &Specialist, cmd: &str) -> Vec<String> {
    let desc = describe_command(cmd);
    vec![
        format!("{}: {}", specialist.name, desc),
    ]
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

    #[test]
    fn test_dispatch_question() {
        // Network question should dispatch to network team
        let result = dispatch_question("my wifi is not working");
        assert!(result.is_some());
        let (msg, specialist) = result.unwrap();
        assert_eq!(specialist.department, "Network");
        assert!(msg.contains("Michael") || msg.contains("Sarah"));
    }

    #[test]
    fn test_ticket_opened() {
        let msg = ticket_opened("CN-0001-12012026", "Network");
        assert!(msg.contains("CN-0001"));
        assert!(msg.contains("Network"));
    }
}
