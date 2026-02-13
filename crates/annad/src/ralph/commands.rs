//! Command execution and answer generation for the Ralph loop.

use anyhow::Result;

use crate::ollama;
use super::criteria::IterationState;
use super::answer_gen::check_recipes_for_commands;

pub use super::answer_gen::{generate_answer, self_evaluate};

/// Result of asking the LLM what to do next.
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

    // v0.3.187: Quick keyword check for CONFIG requests BEFORE recipes
    // This prevents learned recipes from bypassing system configuration detection
    let q_lower = question.to_lowercase();

    // v0.3.187: Agentic capability detection (before CONFIG check)
    let list_patterns = ["what did you create", "what have you created", "what automations", "list automations", "what scripts", "what services did you", "what did anna create", "show me what you created"];
    if list_patterns.iter().any(|p| q_lower.contains(p)) {
        return Ok(NextAction::ListCreated);
    }

    if (q_lower.contains("audit") || q_lower.contains("check") || q_lower.contains("scan"))
        && (q_lower.contains("ssh") || q_lower.contains("sshd"))
    {
        return Ok(NextAction::AuditSsh);
    }

    if q_lower.contains("wallpaper") && (q_lower.contains("random") || q_lower.contains("automatic") || q_lower.contains("daily") || q_lower.contains("every day") || q_lower.contains("set") || q_lower.contains("change")) {
        return Ok(NextAction::SetWallpaper);
    }

    if (q_lower.contains("compile") || q_lower.contains("build")) && q_lower.contains("kernel") {
        return Ok(NextAction::BuildKernel);
    }

    // User management: "create user", "delete user", "add user", "remove user", "create account"
    if (q_lower.contains("create") || q_lower.contains("add") || q_lower.contains("delete") || q_lower.contains("remove"))
        && (q_lower.contains(" user") || q_lower.contains(" account"))
        && !q_lower.contains("update") // avoid "update system" being misclassified
    {
        return Ok(NextAction::ManageUser);
    }

    // Automation creation: "automatically" + action verb / "every X days" + action
    let auto_verbs = ["delete", "clean", "remove", "backup", "sync", "archive", "rotate", "prune", "clear"];
    let auto_triggers = ["automatically", "every day", "daily", "every week", "weekly", "every hour", "every month", "auto-delete", "autoclean", "on schedule"];
    let has_auto_trigger = auto_triggers.iter().any(|t| q_lower.contains(t));
    let has_auto_verb = auto_verbs.iter().any(|v| q_lower.contains(v));
    if has_auto_trigger && has_auto_verb {
        return Ok(NextAction::CreateAutomation);
    }

    // v0.3.151: Check if this is an analytical question (NOT a config request)
    let analytical_patterns = [
        ("has", "changed"), ("has", "been"), ("did", "change"),
        ("when", "changed"), ("why", "changed"), ("what", "changed"),
        ("how has", "changed"), ("how did", "change"),
    ];

    let is_analytical = analytical_patterns.iter().any(|(prefix, suffix)| {
        q_lower.contains(prefix) && q_lower.contains(suffix)
    }) || (q_lower.starts_with("has ") || q_lower.starts_with("did ") || q_lower.starts_with("when ") || q_lower.starts_with("why "));

    // v0.3.151: Added "schedule", "cron", "timer", "automate" for scheduling tasks
    let config_keywords = [
        "update", "upgrade", "reboot", "restart", "shutdown",
        "install", "uninstall", "remove", "add",
        "enable", "disable", "activate", "deactivate",
        "configure", "setup", "migrate", "replace",
        "set", "change", "apply", "modify",
        "schedule", "cron", "timer", "automate",
    ];

    let has_config_keyword = !is_analytical && config_keywords.iter().any(|kw| q_lower.contains(kw));

    // v0.3.148: If CONFIG keyword detected, return CONFIG immediately (no LLM needed!)
    if has_config_keyword && state.commands.is_empty() {
        tracing::info!("CONFIG keyword detected in '{}', skipping LLM classification", question);
        return Ok(NextAction::Config);
    }

    // v0.3.111: Check recipes only if no CONFIG keyword present
    if state.commands.is_empty() && !has_config_keyword {
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

Determine what to do. Output EXACTLY ONE of:

If you need to run commands first, output:
COMMANDS:
<command1>
<command2>

If this is a system configuration request (change settings, enable/disable, install, etc.), output exactly:
CONFIG

If you can answer from knowledge alone (how-to, explanations), output exactly:
NONE

If the data already collected is sufficient to answer, output exactly:
DONE

COMMAND REFERENCE:
SYSTEM: uname -r, uptime -p, hostnamectl
HARDWARE: lscpu | head -20, free -h, lsusb, lspci | head -20
DESKTOP: echo $XDG_CURRENT_DESKTOP, echo $XDG_SESSION_TYPE
STORAGE: df -h, lsblk, findmnt / -o OPTIONS, swapon --show
NETWORK: ip -4 addr show, cat /etc/resolv.conf, ip route | grep default, ss -tlnp | head -15
SERVICES: systemctl --failed, systemctl list-units --type=service --state=running | head -20
PACKAGES: pacman -Q | wc -l, pacman -Qe | head -30
LOGS: journalctl -p err -b --no-pager | head -30
CONFIG FILES: pacman -Ql <pkg> | grep -E '\.(conf|cfg|ini|toml|yaml|yml)$', find ~/.config/<app> -type f

SEMANTIC DEPTH RULES (critical for useful answers):
- Disk/size questions: NEVER stop at a container directory. If du shows ~/.steam, ~/Games, ~/.local/share, ~/Downloads as big, drill one level deeper: du -sh ~/.steam/steam/steamapps/common/* | sort -rh | head -20 to show actual game names.
- Top folders: show the CONTENTS of large generic dirs, not just the dir itself. The user wants to know WHAT is big, not WHERE the container is.
- Config file location: prefer pacman -Ql <appname> over guessing — the package manager knows exactly which files belong to the package and where they are.
- Log files: if /var/log is big, show du -sh /var/log/* sorted to name the actual logs.
- Always ask: "Is this result actionable?" — if the answer points at a container, go one level deeper.

RULES:
- For info/diagnostic questions: use COMMANDS format
- For "set", "change", "disable", "enable", "install", "configure", "prevent", "replace", "setup", "migrate", "update", "upgrade", "reboot", "restart", "shutdown", "apply", "modify", "add", "remove", "uninstall", "activate", "deactivate" requests: use CONFIG
- For bootloader changes (grub, limine, systemd-boot): use CONFIG
- For snapshot/snapper setup: use CONFIG
- For "how do I", "what is", "explain" questions: use NONE
- Output ONLY the format above, no explanations

Output now:"#,
        context = system_context(),
        question = question,
        output_context = output_context,
        feedback_context = feedback_context,
    );

    let response = ollama::chat_with_timeout(model, &prompt, 30).await?;
    let response = response.trim();
    let response_upper = response.to_uppercase();

    if response_upper.starts_with("CONFIG") || response_upper == "CONFIG" {
        return Ok(NextAction::Config);
    }

    if response_upper == "NONE" || response_upper == "DONE" || response.is_empty() {
        return Ok(NextAction::None);
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
        | NextAction::ManageUser | NextAction::BuildKernel => Ok(Vec::new()),
    }
}
