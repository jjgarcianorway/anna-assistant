//! LLM command planner: ask the model what to run, run it, return the output.
//! Works for any question — no hardcoded intents, no keyword matching.

use anyhow::Result;
use tracing::debug;
use super::super::instant_answers::{run_shell, send_answer};
use crate::state::SharedState;

// Commands blocked regardless of what the LLM returns.
const BLOCKED: &[&str] = &[
    "rm ", "rm\t", "dd ", "mkfs", "fdisk", "parted", "wipefs", "shred",
    "reboot", "shutdown", "poweroff", "halt", ":(){", ">(", "mkswap",
    "truncate", "pacman -R", "pacman -D", "pacman -Rc",
];

const PLAN_PROMPT: &str = r#"You are a Linux system assistant. The daemon runs as root.

SYSTEM:
{system_context}
RULES:
- Reply with ONLY a JSON array of shell commands, nothing else
- Max 4 commands. No sudo (daemon is already root)
- Read-only commands only (no rm, dd, mkfs, reboot, shutdown, pacman -R, etc.)
- Each command must complete in under 10 seconds. Prefer concise output

EXAMPLES:
Question: "how is my system today?"
Reply: ["uptime", "free -h", "df -h /", "systemctl --failed --no-pager"]

Question: "what is my IP address?"
Reply: ["ip -4 addr show | grep -v '127.0.0.1'"]

Question: "what processes use the most CPU?"
Reply: ["ps aux --sort=-%cpu | head -11"]

Question: "{question}"
Reply:"#;

/// Build real system context from the globally-cached SystemIdentity.
/// Injected into the LLM prompt so it can plan commands correctly for this host.
fn build_system_context() -> String {
    let identity = crate::system_identity::get_system_identity();
    let aur_helper = detect_aur_helper();
    let mut ctx = format!(
        "OS: {} ({})\nHostname: {}\nPackage manager: {}\n",
        identity.distro_name,
        match identity.distro_family.as_str() {
            "arch" => "Arch-based", "debian" => "Debian-based",
            "fedora" => "Fedora-based", f => f,
        },
        identity.hostname,
        identity.package_manager(),
    );
    if let Some(ref h) = aur_helper {
        ctx.push_str(&format!("AUR helper: {}\n", h));
    }
    // Surface tools the LLM should know are available
    for tool in &["checkupdates", "arch-update", "sensors", "smartctl", "btop", "ncdu"] {
        if cmd_exists(tool) { ctx.push_str(&format!("Tool available: {}\n", tool)); }
    }
    ctx
}

fn cmd_exists(cmd: &str) -> bool {
    std::process::Command::new("which").arg(cmd)
        .output().map(|o| o.status.success()).unwrap_or(false)
}

pub async fn classify_and_execute(
    question: &str,
    writer: &mut tokio::net::unix::OwnedWriteHalf,
    state: &SharedState,
) -> Result<bool> {
    let ql = question.to_lowercase();

    // Fast paths run BEFORE model check — they need no LLM.

    // Fast path: thermal / fan status ("why are fans loud", "why is it hot", "what's hot")
    let is_thermal = (ql.contains("fan") || ql.contains("hot") || ql.contains("temp") || ql.contains("heat") || ql.contains("cool"))
        && (ql.contains("why") || ql.contains("loud") || ql.contains("noisy") || ql.contains("high") || ql.contains("status"));
    if is_thermal {
        return exec_thermal_status(writer).await;
    }

    // Fast path: fan / power reduction ("reduce fan speed", "quiet mode", "power saver")
    let is_fan_reduce = (ql.contains("fan") || ql.contains("noise") || ql.contains("loud") || ql.contains("quiet"))
        && (ql.contains("reduc") || ql.contains("lower") || ql.contains("less") || ql.contains("slow") || ql.contains("quiet"));
    let is_power_save = ql.contains("power") && (ql.contains("sav") || ql.contains("low") || ql.contains("eco"));
    if is_fan_reduce || is_power_save {
        return exec_set_power_saver(writer).await;
    }

    // Fast path: check for pending updates (read-only)
    let is_check = (ql.contains("update") || ql.contains("upgrade"))
        && (ql.contains("pending") || ql.contains("available") || ql.contains("check")
            || ql.contains("any") || ql.contains("what") || ql.contains("how many"));
    if is_check {
        return exec_check_updates(writer).await;
    }

    // Fast path: install updates (mutating)
    let is_update = (ql.contains("update") || ql.contains("upgrade") || ql.contains("arch-update"))
        && !ql.contains("what") && !ql.contains("check") && !ql.contains("pending")
        && !ql.contains("available") && !ql.contains("any") && !ql.contains("how many");
    if is_update {
        return exec_update(writer).await;
    }

    // LLM command planner for everything else.
    let model = {
        let guard = state.read().await;
        match guard.model.clone() {
            Some(m) => m,
            None => return Ok(false),
        }
    };

    let system_context = build_system_context();
    let prompt = PLAN_PROMPT
        .replace("{system_context}", &system_context)
        .replace("{question}", question);
    let response = match crate::ollama::chat_with_timeout(&model, &prompt, 8).await {
        Ok(r) => r,
        Err(e) => {
            // LLM unavailable — don't fall through to main pipeline (it will also fail
            // and return "no matching capability"). Return a clear error instead.
            debug!("Command planning failed: {}", e);
            send_answer(writer, "Anna is recovering from an Ollama outage — please try again in a moment.".to_string()).await?;
            return Ok(true);
        }
    };

    let commands = parse_commands(&response);
    debug!("Planned {} command(s) for '{}'", commands.len(), question);

    // Empty result means the LLM decided this isn't a system command query
    // (e.g. "how do I configure nginx?") — fall through to main pipeline.
    if commands.is_empty() {
        return Ok(false);
    }

    let safe: Vec<&str> = commands.iter()
        .map(String::as_str)
        .filter(|cmd| !is_blocked(cmd))
        .collect();

    if safe.is_empty() {
        debug!("All planned commands were blocked for '{}'", question);
        return Ok(false);
    }

    let mut parts: Vec<String> = Vec::new();
    for cmd in &safe {
        match run_shell(cmd) {
            Ok(out) if !out.trim().is_empty() => {
                parts.push(format!("```\n{}\n```", out.trim()));
            }
            Ok(_) => {}
            Err(e) => debug!("Command '{}' failed: {}", cmd, e),
        }
    }

    if parts.is_empty() {
        send_answer(writer, "Commands ran but produced no output.".to_string()).await?;
        return Ok(true);
    }

    send_answer(writer, parts.join("\n\n")).await?;
    Ok(true)
}

fn parse_commands(response: &str) -> Vec<String> {
    // Find JSON array in response — model may add prose before/after
    let start = response.find('[');
    let end = response.rfind(']');
    if let (Some(s), Some(e)) = (start, end) {
        if s < e {
            let json = &response[s..=e];
            if let Ok(cmds) = serde_json::from_str::<Vec<String>>(json) {
                return cmds.into_iter().filter(|c| !c.trim().is_empty()).collect();
            }
        }
    }
    Vec::new()
}

fn is_blocked(cmd: &str) -> bool {
    let lower = cmd.to_lowercase();
    BLOCKED.iter().any(|b| lower.contains(b))
        || lower.contains("arch-update") || lower.contains("paru -syu") || lower.contains("yay -syu")
}

fn exec_update_inner() -> String {
    let username = crate::user_context::get_real_user().unwrap_or_else(|_| "root".to_string());

    // Count available updates
    let count: u32 = run_shell("checkupdates 2>/dev/null | wc -l || echo 0")
        .unwrap_or_default().trim().parse().unwrap_or(0);

    if count == 0 {
        let aur = detect_aur_helper();
        let aur_msg = aur.as_ref().map(|h| {
            let n: u32 = run_shell(&format!("runuser -l {} -c '{} -Qu 2>/dev/null | wc -l'", username, h))
                .unwrap_or_default().trim().parse().unwrap_or(0);
            if n == 0 { String::new() } else { format!(" ({} AUR updates available)", n) }
        }).unwrap_or_default();
        return format!("System is up to date.{}", aur_msg);
    }

    // arch-update if installed
    if run_shell("which arch-update 2>/dev/null").map(|s| !s.trim().is_empty()).unwrap_or(false) {
        let out = run_shell(&format!("runuser -l {} -c 'arch-update 2>&1 | tail -30'", username)).unwrap_or_default();
        return format!("Running system update via arch-update...\n\n{}", out.trim());
    }

    // pacman -Syu + AUR helper
    let pacman = run_shell("pacman -Syu --noconfirm 2>&1 | tail -20").unwrap_or_default();
    let aur = detect_aur_helper();
    let aur_out = aur.as_ref().map(|h| {
        run_shell(&format!("runuser -l {} -c '{} -Syu --noconfirm 2>&1 | tail -20'", username, h)).unwrap_or_default()
    }).unwrap_or_default();

    let mut parts = vec![format!("Updating official repositories...\n{}", pacman.trim())];
    if !aur_out.trim().is_empty() {
        parts.push(format!("Updating AUR packages...\n{}", aur_out.trim()));
    }
    parts.join("\n\n")
}

/// Show thermal status: temperatures + top CPU consumers + power governor
async fn exec_thermal_status(writer: &mut tokio::net::unix::OwnedWriteHalf) -> Result<bool> {
    // Temps — compact: only lines with °C
    let temps = run_shell("sensors 2>/dev/null | grep -E '°C|RPM' | grep -v 'high\\|crit\\|low'")
        .unwrap_or_default();
    // Top CPU hogs
    let procs = run_shell("ps aux --sort=-%cpu | awk 'NR<=6{printf \"%-6s %-5s %s\\n\",$1,$3,$11}' | head -6")
        .unwrap_or_default();
    // Power governor
    let gov = run_shell("cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null")
        .unwrap_or_default();
    // Power profile if available
    let profile = if cmd_exists("powerprofilesctl") {
        run_shell("powerprofilesctl get 2>/dev/null").unwrap_or_default()
    } else { String::new() };

    let mut parts = Vec::new();
    if !temps.trim().is_empty() { parts.push(format!("Temperatures:\n```\n{}\n```", temps.trim())); }
    if !procs.trim().is_empty() { parts.push(format!("Top CPU consumers:\n```\n{}\n```", procs.trim())); }
    let mut meta = Vec::new();
    let gov = gov.trim();
    if !gov.is_empty() { meta.push(format!("CPU governor: {}", gov)); }
    let profile = profile.trim();
    if !profile.is_empty() { meta.push(format!("Power profile: {}", profile)); }
    if !meta.is_empty() { parts.push(meta.join(" | ")); }

    send_answer(writer, parts.join("\n\n")).await?;
    Ok(true)
}

/// Set power-saver mode to reduce fan speed — tries all available tools.
async fn exec_set_power_saver(writer: &mut tokio::net::unix::OwnedWriteHalf) -> Result<bool> {
    let mut actions = Vec::new();

    // power-profiles-daemon (most modern systems)
    if cmd_exists("powerprofilesctl") {
        let _ = run_shell("powerprofilesctl set power-saver 2>/dev/null");
        let current = run_shell("powerprofilesctl get 2>/dev/null").unwrap_or_default();
        actions.push(format!("Power profile set to: {}", current.trim()));
    }

    // CPU frequency governor
    if cmd_exists("cpupower") {
        let _ = run_shell("cpupower frequency-set -g powersave 2>/dev/null");
        let gov = run_shell("cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null")
            .unwrap_or_default();
        actions.push(format!("CPU governor: {}", gov.trim()));
    } else {
        // Direct sysfs fallback
        let _ = run_shell("for f in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do echo powersave > $f 2>/dev/null; done");
        let gov = run_shell("cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null")
            .unwrap_or_default();
        let gov = gov.trim();
        if !gov.is_empty() { actions.push(format!("CPU governor: {}", gov)); }
    }

    // TLP if available
    if cmd_exists("tlp") {
        let _ = run_shell("tlp bat 2>/dev/null");
        actions.push("TLP battery mode activated".to_string());
    }

    if actions.is_empty() {
        send_answer(writer,
            "No power management tools found (powerprofilesctl, cpupower, tlp). Fan speed is controlled by firmware.".to_string()
        ).await?;
    } else {
        send_answer(writer, actions.join("\n")).await?;
    }
    Ok(true)
}

async fn exec_check_updates(writer: &mut tokio::net::unix::OwnedWriteHalf) -> Result<bool> {
    let identity = crate::system_identity::get_system_identity();
    let username = crate::user_context::get_real_user().unwrap_or_else(|_| "root".to_string());

    // Distro-appropriate official update check
    let check_cmd = match identity.distro_family.as_str() {
        "arch" => "checkupdates 2>/dev/null",
        "debian" => "apt list --upgradable 2>/dev/null | tail -n +2",
        "fedora" => "dnf check-update 2>/dev/null | tail -n +3",
        _ => "echo '(unknown distro — cannot check updates)'",
    };
    let official = run_shell(check_cmd).unwrap_or_default();
    let official = official.trim();
    let official_count = if official.is_empty() { 0usize } else { official.lines().count() };

    // AUR updates (Arch only)
    let aur_helper = if identity.distro_family == "arch" { detect_aur_helper() } else { None };
    let (aur_out, aur_count) = if let Some(ref h) = aur_helper {
        let out = run_shell(&format!("runuser -l {} -c '{} -Qu 2>/dev/null'", username, h))
            .unwrap_or_default();
        let out = out.trim().to_string();
        let n = if out.is_empty() { 0 } else { out.lines().count() };
        (out, n)
    } else {
        (String::new(), 0)
    };

    let total = official_count + aur_count;
    if total == 0 {
        send_answer(writer, "System is up to date. No pending updates.".to_string()).await?;
        return Ok(true);
    }

    let mut msg = format!("{} update{} available:\n\n", total, if total == 1 { "" } else { "s" });
    if official_count > 0 {
        msg.push_str(&format!("Official ({}):\n```\n{}\n```\n", official_count, official));
    }
    if aur_count > 0 {
        msg.push_str(&format!("\n{} AUR ({}):\n```\n{}\n```\n",
            aur_helper.as_deref().unwrap_or("AUR"), aur_count, aur_out));
    }
    send_answer(writer, msg.trim().to_string()).await?;
    Ok(true)
}

async fn exec_update(writer: &mut tokio::net::unix::OwnedWriteHalf) -> Result<bool> {
    send_answer(writer, exec_update_inner()).await?;
    Ok(true)
}

fn detect_aur_helper() -> Option<String> {
    ["paru", "yay", "pikaur", "trizen", "aurman"].iter()
        .find(|&&h| run_shell(&format!("which {} 2>/dev/null", h))
            .map(|s| !s.trim().is_empty()).unwrap_or(false))
        .map(|&h| h.to_string())
}
