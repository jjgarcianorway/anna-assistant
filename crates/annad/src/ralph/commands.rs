//! Command execution and answer generation for the Ralph loop.

use anyhow::Result;

use crate::ollama;
use super::criteria::IterationState;
use super::answer_gen::check_recipes_for_commands;

pub use super::answer_gen::{generate_answer, self_evaluate};

/// Result of asking the LLM what to do next.
/// All variants (except Commands/None/Config/Done) are LLM-classified — no keyword matching.
pub enum NextAction {
    /// Run these investigation commands.
    Commands(Vec<String>),
    /// No commands needed (already answered or how-to).
    None,
    /// This is a config request - generate an ActionPlan via LLM.
    Config,
    /// List all artifacts Anna has created.
    ListCreated,
    /// Create a systemd timer/service automation from user intent.
    CreateAutomation,
    /// Set up wallpaper automation for the current DE/WM.
    SetWallpaper,
    /// Audit sshd_config for vulnerabilities.
    AuditSsh,
    /// Manage user accounts (create, delete, change password).
    ManageUser,
    /// Generate kernel compilation plan for this hardware.
    BuildKernel,
    /// Generate a system health PDF report.
    GeneratePdf,
    /// Generate a comprehensive multi-section text system report.
    FullReport,
}

/// Get commands to run for answering the question.
/// May also detect config requests and return NextAction::Config.
/// v0.3.111: Checks learned recipes first for faster response.
/// v0.3.146: Check CONFIG keywords BEFORE recipes to prevent bypass.
pub async fn get_next_action(
    model: &str,
    question: &str,
    state: &IterationState,
) -> Result<NextAction> {
    use crate::llm_core::prompts::system_context;

    // v0.3.195: No keyword matching. LLM classifies all intents.
    // Check recipes first (fast path for previously-seen questions).
    if state.commands.is_empty() {
        if let Some(commands) = check_recipes_for_commands(question) {
            tracing::info!("Using {} commands from learned recipe", commands.len());
            return Ok(NextAction::Commands(commands));
        }
    }

    let feedback_context = if let Some(ref feedback) = state.feedback {
        format!(
            "\n\nPrevious attempt feedback: {}\nAlready tried: {:?}",
            feedback, state.commands
        )
    } else {
        String::new()
    };

    let output_context = if !state.outputs.is_empty() {
        format!(
            "\n\nData collected so far:\n{}",
            state.outputs.join("\n---\n")
        )
    } else {
        String::new()
    };

    let prompt = format!(
        r#"{context}

Question: "{question}"{output_context}{feedback_context}

Classify what this request needs. Output EXACTLY ONE token:

COMMANDS:        — need to run shell commands to gather data first
<command1>
<command2>

CONFIG           — user wants to change/configure/install/enable/disable something
NONE             — can answer from knowledge alone (how-to, explanations, concepts)
DONE             — already have enough collected data to answer

FULL_REPORT      — user wants a comprehensive multi-section system status report
PDF_REPORT       — user wants a system health report as a PDF file
LIST_CREATED     — user is asking what automations/scripts/artifacts Anna has created
AUDIT_SSH        — user wants the SSH configuration audited for security issues
SET_WALLPAPER    — user wants wallpaper changed or scheduled automatically
BUILD_KERNEL     — user wants to compile a custom kernel for this hardware
MANAGE_USER      — user wants to create, delete, or modify a user account
CREATE_AUTOMATION — user wants something to run automatically on a schedule

COMMAND REFERENCE:
SYSTEM: uname -r, uptime -p, hostnamectl
HARDWARE: lscpu | head -20, free -h, lsusb, lspci | head -20
DESKTOP: echo $XDG_CURRENT_DESKTOP, echo $XDG_SESSION_TYPE
STORAGE: df -h, lsblk, findmnt / -o OPTIONS, swapon --show
NETWORK: ip -4 addr show, cat /etc/resolv.conf, ip route | grep default, ss -tlnp | head -15
SERVICES: systemctl --failed, systemctl list-units --type=service --state=running | head -20
PACKAGES: pacman -Q | wc -l, pacman -Qe | head -30
LOGS: journalctl -p err -b --no-pager | head -30
SECURITY: last -n 20, journalctl _COMM=sshd -b | tail -30, cat /etc/ssh/sshd_config

SEMANTIC DEPTH RULES:
- Disk/size questions: drill into large directories — show contents, not the container.
- Always ask: "Is this result actionable?" — if not, go one level deeper.
- Config file location: prefer pacman -Ql <pkg> over guessing.

Output ONLY the token above, no explanations.

Output now:"#,
        context = system_context(),
        question = question,
        output_context = output_context,
        feedback_context = feedback_context,
    );

    let response = ollama::chat_with_timeout(model, &prompt, 30).await?;
    let response = response.trim();
    let response_upper = response.to_uppercase();

    let first_token = response_upper.lines().next().unwrap_or("").trim().to_string();

    match first_token.as_str() {
        "CONFIG"             => return Ok(NextAction::Config),
        "NONE" | "DONE"      => return Ok(NextAction::None),
        "FULL_REPORT"        => return Ok(NextAction::FullReport),
        "PDF_REPORT"         => return Ok(NextAction::GeneratePdf),
        "LIST_CREATED"       => return Ok(NextAction::ListCreated),
        "AUDIT_SSH"          => return Ok(NextAction::AuditSsh),
        "SET_WALLPAPER"      => return Ok(NextAction::SetWallpaper),
        "BUILD_KERNEL"       => return Ok(NextAction::BuildKernel),
        "MANAGE_USER"        => return Ok(NextAction::ManageUser),
        "CREATE_AUTOMATION"  => return Ok(NextAction::CreateAutomation),
        _ if response.is_empty() => return Ok(NextAction::None),
        _ => {}
    }

    // Parse commands (strip "COMMANDS:" prefix if present)
    let cmd_text = if response_upper.starts_with("COMMANDS:") {
        &response[9..]
    } else {
        response
    };

    let commands: Vec<String> = cmd_text
        .lines()
        .map(|l| l.trim())
        .filter(|l| {
            if l.is_empty() || l.starts_with('#') {
                return false;
            }
            let upper = l.to_uppercase();
            if upper == "DONE" || upper == "NONE" || upper.starts_with("DONE:")
                || upper == "CONFIG" || upper == "COMMANDS:" {
                return false;
            }
            // Filter LLM format echoes: "FORMAT N - ...", headers ending with ":"
            if upper.starts_with("FORMAT ") || l.ends_with(':') {
                return false;
            }
            // Must look like an actual shell command, not a prose sentence
            // Real commands don't have parenthetical explanations in the middle
            if l.contains("(to ") || l.contains(" - Run ") || l.contains(" commands (") {
                return false;
            }
            true
        })
        .map(|l| l.to_string())
        .take(5)
        .collect();

    if commands.is_empty() {
        Ok(NextAction::None)
    } else {
        Ok(NextAction::Commands(commands))
    }
}

/// Backwards-compatible wrapper for non-streaming path.
pub async fn get_commands(
    model: &str,
    question: &str,
    state: &IterationState,
) -> Result<Vec<String>> {
    match get_next_action(model, question, state).await? {
        NextAction::Commands(cmds) => Ok(cmds),
        NextAction::None | NextAction::Config | NextAction::ListCreated
        | NextAction::CreateAutomation | NextAction::SetWallpaper | NextAction::AuditSsh
        | NextAction::ManageUser | NextAction::BuildKernel
        | NextAction::GeneratePdf | NextAction::FullReport => Ok(Vec::new()),
    }
}
