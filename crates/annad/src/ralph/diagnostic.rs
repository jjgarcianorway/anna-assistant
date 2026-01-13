//! Diagnostic path for ambiguous queries.
//! Runs pre-selected diagnostics instead of asking for clarification.

use anna_shared::rpc::{DialogueStep, StepType};
use tracing::info;

use crate::core_loop::{execute_command, strip_ansi_codes};
use crate::department;
use crate::team_speak;

/// Get diagnostic commands for ambiguous queries.
/// Returns (commands, intro_text) for running diagnostics.
pub fn get_diagnostic_path(question: &str) -> Option<(&'static [&'static str], &'static str)> {
    let q = question.to_lowercase();

    // "it's slow" / "make it faster" / "system is slow"
    if (q.contains("slow") || q.contains("faster") || q.contains("laggy") || q.contains("sluggish"))
        && !q.contains("boot") && !q.contains("start")
    {
        return Some((
            &["uptime", "free -h", "top -bn1 | head -15", "df -h | grep -E '^/dev'"],
            "Running performance diagnostics to identify the bottleneck..."
        ));
    }

    // "fix my wifi" / "wifi not working" / "no internet"
    if q.contains("wifi") || q.contains("internet") || (q.contains("network") && !q.contains("what")) {
        return Some((
            &["ip link show", "ip -4 addr show", "ping -c 2 8.8.8.8 2>&1", "cat /etc/resolv.conf | grep nameserver"],
            "Checking network connectivity..."
        ));
    }

    // "something is wrong" / "nothing works" / "I broke something"
    if q.contains("something is wrong") || q.contains("nothing works") || q.contains("broke something")
        || q.contains("broken") || q.contains("check if everything")
    {
        return Some((
            &["systemctl --failed", "journalctl -p err -b --no-pager | head -20", "df -h | grep -E '^/dev'", "free -h"],
            "Running general health check..."
        ));
    }

    // "why won't it start" / "not starting" / "can't start"
    if (q.contains("won't start") || q.contains("not start") || q.contains("can't start") || q.contains("doesn't start"))
        && !q.contains("specific")
    {
        return Some((
            &["systemctl --failed", "journalctl -p err -b --no-pager | head -20", "dmesg | tail -20"],
            "Checking for startup failures..."
        ));
    }

    // "display is weird" / "screen problem"
    if (q.contains("display") || q.contains("screen") || q.contains("monitor"))
        && (q.contains("weird") || q.contains("problem") || q.contains("issue") || q.contains("wrong"))
    {
        return Some((
            &["echo $XDG_SESSION_TYPE", "xrandr 2>/dev/null || wlr-randr 2>/dev/null", "lsmod | grep -E 'nvidia|amdgpu|i915'", "journalctl -b | grep -iE 'drm|gpu' | tail -10"],
            "Checking display configuration..."
        ));
    }

    // "fan is loud" / "fan spinning"
    if q.contains("fan") && (q.contains("loud") || q.contains("spin") || q.contains("noise")) {
        return Some((
            &["cat /sys/class/thermal/thermal_zone*/temp 2>/dev/null | head -5", "top -bn1 | head -10", "sensors 2>/dev/null | head -20"],
            "Checking CPU temperature and load..."
        ));
    }

    // "where did my files go" / "files missing"
    if (q.contains("files") || q.contains("folder") || q.contains("directory"))
        && (q.contains("gone") || q.contains("missing") || q.contains("where") || q.contains("disappeared"))
    {
        return Some((
            &["df -h | grep -E '^/dev'", "mount | grep -E '^/dev'", "ls -la ~ | head -15"],
            "Checking filesystem and mount points..."
        ));
    }

    // "help" alone
    if q.trim() == "help" || q.trim() == "help me" || q.trim() == "i need help" {
        return Some((
            &["systemctl --failed", "df -h | grep -E '^/dev'", "free -h"],
            "What can I help you with? Here's your system status:"
        ));
    }

    // "what's using bandwidth" / "bandwidth hog"
    if q.contains("bandwidth") || (q.contains("network") && q.contains("using")) {
        return Some((
            &["ss -tunp | head -20", "nethogs -t -c 3 2>/dev/null | head -15 || echo 'nethogs not installed - run: pacman -S nethogs'"],
            "Checking network usage..."
        ));
    }

    // "what's using CPU" / "CPU hog"
    if (q.contains("cpu") || q.contains("processor")) && (q.contains("using") || q.contains("hog") || q.contains("100%")) {
        return Some((
            &["top -bn1 | head -15", "ps aux --sort=-%cpu | head -10"],
            "Checking CPU usage..."
        ));
    }

    // "what's using memory/RAM"
    if (q.contains("memory") || q.contains("ram")) && (q.contains("using") || q.contains("hog") || q.contains("eating")) {
        return Some((
            &["free -h", "ps aux --sort=-%mem | head -10"],
            "Checking memory usage..."
        ));
    }

    // "why did X fail" / "last error"
    if (q.contains("why did") && q.contains("fail")) || q.contains("last error") || q.contains("recent error")
        || q.contains("what went wrong") || q.contains("what failed")
    {
        return Some((
            &["systemctl --failed", "journalctl -p err -b --no-pager | tail -20"],
            "Checking recent failures..."
        ));
    }

    // "is my system compromised" / "security check"
    if q.contains("compromised") || q.contains("hacked") || q.contains("security check") || q.contains("suspicious") {
        return Some((
            &["last -10", "who", "ss -tunp | grep ESTABLISHED | head -10", "find /tmp -type f -perm -111 2>/dev/null | head -5"],
            "Running basic security check..."
        ));
    }

    None
}

/// Try diagnostic path for ambiguous queries.
/// Runs pre-selected diagnostics and returns results for LLM analysis.
pub fn try_diagnostic_path(question: &str) -> Option<(Vec<String>, Vec<String>, &'static str, Vec<DialogueStep>)> {
    let (commands, intro) = get_diagnostic_path(question)?;

    info!("Diagnostic path: running {} commands", commands.len());

    let mut outputs = Vec::new();
    let mut executed = Vec::new();
    let mut dialogue = vec![
        DialogueStep {
            step_type: StepType::UserQuestion,
            content: question.to_string(),
        },
    ];

    // Add fly-on-the-wall elements to diagnostic path
    let dept_name = department::determine_department(question);
    let mut ticket = department::create_ticket(question, dept_name);

    dialogue.push(DialogueStep {
        step_type: StepType::TicketCreated,
        content: ticket.case_number.clone(),
    });

    if let Some(spec) = department::get_specialist_for_topic(question) {
        // Assign ticket to specialist for stats tracking
        ticket.assign(spec.name);
        department::update_ticket(&ticket);

        let assignment = team_speak::anna_assigns_to(spec, question);
        dialogue.push(DialogueStep {
            step_type: StepType::TeamAssignment,
            content: assignment,
        });
        let ack = team_speak::specialist_acknowledges(spec);
        dialogue.push(DialogueStep {
            step_type: StepType::SpecialistWorking,
            content: format!("{}: {}", spec.name, ack),
        });
    }

    for cmd in commands {
        dialogue.push(DialogueStep {
            step_type: StepType::CommandExec,
            content: cmd.to_string(),
        });

        match execute_command(cmd) {
            Ok(output) => {
                let clean = strip_ansi_codes(&output);
                dialogue.push(DialogueStep {
                    step_type: StepType::CommandOutput,
                    content: truncate(&clean, 500),
                });
                outputs.push(clean);
                executed.push(cmd.to_string());
            }
            Err(e) => {
                dialogue.push(DialogueStep {
                    step_type: StepType::CommandOutput,
                    content: format!("Error: {}", e),
                });
            }
        }
    }

    Some((executed, outputs, intro, dialogue))
}

/// Truncate string with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let mut end = max_len;
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}...", &s[..end])
    }
}
