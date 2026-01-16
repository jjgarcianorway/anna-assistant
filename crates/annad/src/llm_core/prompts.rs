//! LLM prompts for the pure intelligence core loop.
//!
//! These prompts are designed to be:
//! - Clear and unambiguous
//! - Grounded in actual system information
//! - Focused on investigation rather than assumptions

use super::InvestigationState;

/// System context that goes into every prompt
pub fn system_context() -> String {
    // Gather real system info
    let distro = std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("PRETTY_NAME="))
                .map(|l| l.trim_start_matches("PRETTY_NAME=").trim_matches('"').to_string())
        })
        .unwrap_or_else(|| "Arch Linux".to_string());

    let kernel = std::process::Command::new("uname")
        .arg("-r")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    format!(
        "You are Anna, an intelligent Linux system assistant.\n\
         System: {} (kernel {})\n\
         Package manager: pacman (Arch-based)\n\
         You help users by investigating their system and providing grounded answers.",
        distro, kernel
    )
}

/// Prompt to understand what the user is asking
pub fn understanding_prompt(question: &str) -> String {
    format!(
        r#"{context}

USER QUESTION: "{question}"

Analyze this question. Respond with:

INTENT: <one of: info, diagnostic, howto, fix, config>
NEED: <what information is needed to answer - one per line>

If this question is NOT about Linux/system administration, respond with:
OUT_OF_SCOPE: <brief explanation why>

Examples:
- "how much disk space" -> INTENT: info, NEED: disk usage statistics
- "why is my fan loud" -> INTENT: diagnostic, NEED: CPU usage, temperature, running processes
- "install docker" -> INTENT: howto, NEED: none (can answer from knowledge)
- "fix pacman lock" -> INTENT: fix, NEED: check if lock file exists, check running pacman

Respond now:"#,
        context = system_context(),
        question = question
    )
}

/// Prompt to decide what investigation steps to take next
pub fn next_step_prompt(question: &str, state: &InvestigationState) -> String {
    let findings_text = if state.findings.is_empty() {
        "No commands executed yet.".to_string()
    } else {
        state.findings.iter()
            .map(|f| {
                let status = if f.success { "OK" } else { "FAILED" };
                let output = if f.output.len() > 1000 {
                    format!("{}...(truncated)", &f.output[..1000])
                } else {
                    f.output.clone()
                };
                format!("$ {} [{}]\n{}", f.command, status, output)
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    };

    format!(
        r#"{context}

USER QUESTION: "{question}"

INVESTIGATION SO FAR (iteration {iteration}):
{findings}

OUTPUT EXACTLY ONE OF THESE FORMATS:

FORMAT 1 - Run commands (if you need more information):
COMMANDS:
<command1>
<command2>

FORMAT 2 - Answer (if you have enough information):
ANSWER

FORMAT 3 - Suggest fix (if you found a problem):
FIX: <command to fix>
PROBLEM: <what's wrong>
EXPLAIN: <why this fixes it>

COMMAND REFERENCE (use these exact commands):
=== System Info ===
- Kernel: uname -r
- Uptime: uptime -p
- Hostname: hostnamectl
- System load: uptime
- Boot time analysis: systemd-analyze blame | head -10

=== Hardware ===
- CPU info: lscpu | head -20
- CPU freq: cat /proc/cpuinfo | grep MHz | head -4
- RAM: free -h
- GPU: lspci | grep -i vga
- GPU driver: lsmod | grep -E 'nvidia|amdgpu|i915'
- USB devices: lsusb
- PCI devices: lspci | head -20
- Temperatures: cat /sys/class/thermal/thermal_zone*/temp 2>/dev/null
- Battery: cat /sys/class/power_supply/BAT*/capacity 2>/dev/null

=== Desktop Environment ===
- DE/WM: echo $XDG_CURRENT_DESKTOP
- Display server: echo $XDG_SESSION_TYPE
- Display resolution: xrandr 2>/dev/null | grep '*' || wlr-randr 2>/dev/null | grep current

=== User/Shell ===
- Current shell: echo $SHELL
- Username/UID: id
- Groups: groups
- Locale: locale
- Timezone: timedatectl | grep "Time zone"

=== Storage ===
- Disk space: df -h
- Disk usage: du -sh /* 2>/dev/null | sort -h | head -10
- Partitions: lsblk -o NAME,SIZE,TYPE,FSTYPE,MOUNTPOINT
- Mount options: findmnt / -o OPTIONS
- GPT/MBR: sudo fdisk -l 2>/dev/null | grep -E 'Disklabel|GPT|DOS' | head -3
- Swap: swapon --show
- Btrfs subvols: btrfs subvolume list / 2>/dev/null

=== Network ===
- IP address: ip -4 addr show | grep inet | grep -v 127.0.0.1
- Interfaces: ip link show
- DNS: cat /etc/resolv.conf
- Gateway: ip route | grep default
- Listening ports: ss -tlnp 2>/dev/null | head -15
- Public IP: curl -s ifconfig.me 2>/dev/null

=== Services ===
- Running services: systemctl list-units --type=service --state=running | head -20
- Failed services: systemctl --failed
- Service status: systemctl status <service>
- Active timers: systemctl list-timers --no-pager | head -10

=== Packages ===
- Package count: pacman -Q | wc -l
- Installed packages: pacman -Qe | head -30
- Package info: pacman -Qi <package>
- Package owner: pacman -Qo <file>
- Orphaned packages: pacman -Qtdq | head -10
- Recently installed: tail -20 /var/log/pacman.log | grep "installed"
- Cache size: du -sh /var/cache/pacman/pkg/

=== Logs/Errors ===
- Recent errors: journalctl -p err -b --no-pager | head -30
- Boot logs: journalctl -b --no-pager | head -50
- dmesg errors: dmesg | grep -iE 'error|fail|warn' | tail -20

=== Audio ===
- Audio server: pactl info 2>/dev/null | head -5 || pipewire --version

=== Security ===
- SSH keys: ls -la ~/.ssh/*.pub 2>/dev/null
- SUID files: find /usr/bin -perm -4000 2>/dev/null | head -10
- Users: cat /etc/passwd | grep -v nologin | tail -5

RULES:
1. NO interactive commands (top, htop, vim, nano, less, man)
2. Maximum 2 commands per response
3. Only output valid bash commands - no English text, no explanations
4. Do NOT repeat commands that already ran
5. If output is sufficient, respond with just: ANSWER

Respond now with ONLY the format above:"#,
        context = system_context(),
        question = question,
        iteration = state.iteration,
        findings = findings_text
    )
}

/// Prompt to generate the final grounded answer
/// Phase 15: Includes HARD RULES forbidding manual commands.
pub fn answer_prompt(question: &str, state: &InvestigationState) -> String {
    let findings_text = state.findings.iter()
        .map(|f| {
            let status = if f.success { "OK" } else { "FAILED" };
            format!("$ {} [{}]\n{}", f.command, status, f.output)
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    format!(
        r#"{context}

USER QUESTION: "{question}"

INVESTIGATION RESULTS:
{findings}

Based on the command outputs above, provide a helpful answer.

ABSOLUTE RULES (VIOLATIONS WILL BE BLOCKED):
You MUST NOT:
- Suggest manual commands (e.g., "run: ...", "execute: ...")
- Suggest sudo usage
- Suggest editing files manually (e.g., "edit /etc/...", "open with nano...")
- Output shell commands for the user to run
- Tell the user to "run" or "type" anything

You MUST:
- Answer the question directly with the information found
- DO NOT ask "would you like me to..." or offer to do something - just answer
- DO NOT suggest changes unless the user explicitly asked for a fix
- If the user asked about status/info, give them the status/info
- If you found a problem, report it, but do not offer to fix it unless asked

GROUNDING RULES:
1. Your answer MUST be grounded in the actual command output shown above
2. Do NOT invent information that isn't in the output
3. Be concise but complete
4. If the output shows "no results" or empty, that IS an answer
5. Format numbers and paths clearly

Provide your answer now:"#,
        context = system_context(),
        question = question,
        findings = findings_text
    )
}
