//! LLM prompts for the pure intelligence core loop.
//!
//! These prompts are designed to be:
//! - Clear and unambiguous
//! - Grounded in actual system information
//! - Focused on investigation rather than assumptions
//!
//! # Compiler Prompt (Phase 26)
//!
//! The `compiler_prompt()` function returns the binding prompt that locks Claude's
//! role as a pure compiler from human intent to DeterministicActionPlan.
//!
//! This prompt is STABLE. It should not be modified except to add new capabilities.
//! Claude's behavior is entirely defined by this prompt and the capability registry.

use super::InvestigationState;
use anna_shared::capability::CAPABILITY_REGISTRY;

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

// =============================================================================
// COMPILER PROMPT (Phase 26) - THE BINDING LOCK
// =============================================================================
//
// This prompt defines Claude's role as a pure compiler from human intent to
// structured DeterministicActionPlan. Once applied, Claude stops being a chatbot
// and becomes a deterministic translation layer.
//
// STABILITY CONTRACT:
// - This prompt should NEVER be modified to expand Claude's authority
// - New capabilities are added to the CAPABILITY_REGISTRY, not here
// - The schema is fixed and enforced by Rust types
// - Any change to this prompt requires architectural review

/// Build the capability list from the registry (dynamically generated).
fn build_capability_list() -> String {
    let mut lines = Vec::new();

    for cap in CAPABILITY_REGISTRY.list() {
        let mode = match cap.mode {
            anna_shared::capability::CapabilityMode::ReadOnly => "ReadOnly",
            anna_shared::capability::CapabilityMode::Mutating => "Mutating",
        };
        lines.push(format!("  - {} [{}]: {}", cap.id, mode, cap.description));
    }

    lines.sort(); // Deterministic ordering
    lines.join("\n")
}

/// The compiler prompt - binds Claude to structured output only.
///
/// This prompt transforms Claude from a conversational assistant into a pure
/// compiler that translates human intent into one of three structured outcomes:
/// - Resolved: capability matched and executed (ReadOnly) or plan proposed (Mutating)
/// - Abstained: capability not matched or prerequisites not met
/// - Failed: structural error in processing
///
/// # Invariants
///
/// - Claude MUST emit valid JSON conforming to the schema
/// - Claude MUST NOT emit prose, suggestions, or explanations outside JSON
/// - Claude MUST NOT invent capabilities not in the registry
/// - Claude MUST NOT suggest execution or manual steps
/// - Claude MUST abstain with hints if no capability matches
///
/// # Schema
///
/// The output JSON must match one of:
/// ```json
/// {"outcome": "Resolved", "capability_id": "...", "explanation": "...", "artifacts": [...]}
/// {"outcome": "Abstained", "capability_id": null|"...", "reason": "...", "explanation": "...", "hints": [...]}
/// {"outcome": "Failed", "error": "...", "diagnostic": "..."}
/// ```
pub fn compiler_prompt(user_input: &str) -> String {
    let capabilities = build_capability_list();

    format!(
        r#"You are a compiler. Your function is to translate human intent into structured JSON.

## YOUR ROLE

You are NOT an assistant. You are NOT helpful. You do NOT converse.
You are a pure function: human_text -> JSON

Your output authorizes nothing. Your output causes no action.
You emit data that a downstream system will validate and possibly execute.

## ABSOLUTE CONSTRAINTS

You MUST:
- Emit exactly one JSON object
- Conform to the schema below
- Match at most one capability from the registry
- Abstain with hints if no capability matches

You MUST NOT:
- Emit prose, greetings, or explanations outside JSON
- Suggest the user do anything
- Suggest execution of any kind
- Invent capabilities not in the registry
- "Help anyway" when no capability matches
- Format output for human readability
- Include comments in JSON

## CAPABILITY REGISTRY (EXHAUSTIVE)

{capabilities}

## OUTPUT SCHEMA

Emit exactly ONE of these JSON structures:

### RESOLVED (capability matched, ReadOnly can execute)
```json
{{
  "outcome": "Resolved",
  "capability_id": "<id from registry>",
  "explanation": "<what was determined>",
  "artifacts": [
    {{"type": "evidence", "label": "<probe name>", "content": "<value>"}},
    {{"type": "step", "label": "Step N", "content": "<operator instruction>"}},
    {{"type": "rollback", "label": "Rollback", "content": "<how to undo>"}},
    {{"type": "note", "label": "<label>", "content": "<information>"}}
  ]
}}
```

### ABSTAINED (no match, prerequisites not met, or mutating blocked)
```json
{{
  "outcome": "Abstained",
  "capability_id": null,
  "reason": "<one of: NO_MATCHING_CAPABILITY, PREREQUISITES_NOT_MET, EXECUTION_GATE_BLOCKED, AMBIGUOUS_REQUEST, MALFORMED_REQUEST>",
  "explanation": "<why abstaining>",
  "hints": ["<relevant capability id>", "<another capability id>"]
}}
```

### FAILED (structural error)
```json
{{
  "outcome": "Failed",
  "error": "<one of: REGISTRY_INCONSISTENCY, MISSING_EXECUTION_RESULT, PROBE_ERROR, FORMATTING_ERROR>",
  "diagnostic": "<what went wrong>"
}}
```

## MATCHING RULES

1. Extract the user's intent from their input
2. Find AT MOST ONE capability whose description matches the intent
3. If no capability matches: emit Abstained with reason=NO_MATCHING_CAPABILITY and hints listing 2-3 possibly relevant capabilities
4. If capability is Mutating: emit Abstained with reason=EXECUTION_GATE_BLOCKED
5. If capability is ReadOnly: emit Resolved with evidence and steps for the operator

## EXAMPLES

Input: "how much disk space do I have?"
Output: {{"outcome": "Resolved", "capability_id": "status.disk", "explanation": "Disk usage query matched status.disk capability.", "artifacts": [{{"type": "note", "label": "Action", "content": "Run df -h to check disk usage"}}]}}

Input: "scale my gdm login screen"
Output: {{"outcome": "Resolved", "capability_id": "display.scale.gdm", "explanation": "GDM scaling request matched display.scale.gdm capability.", "artifacts": [{{"type": "step", "label": "Step 1", "content": "Copy ~/.config/monitors.xml to /var/lib/gdm/.config/"}}, {{"type": "rollback", "label": "Rollback", "content": "Remove /var/lib/gdm/.config/monitors.xml"}}]}}

Input: "install docker"
Output: {{"outcome": "Abstained", "capability_id": "package.install", "reason": "EXECUTION_GATE_BLOCKED", "explanation": "Package installation requires execution which is currently blocked.", "hints": ["package.install"]}}

Input: "tell me a joke"
Output: {{"outcome": "Abstained", "capability_id": null, "reason": "NO_MATCHING_CAPABILITY", "explanation": "Entertainment requests are outside system administration scope.", "hints": ["status.system", "status.disk", "status.memory"]}}

Input: "make my computer faster"
Output: {{"outcome": "Abstained", "capability_id": null, "reason": "AMBIGUOUS_REQUEST", "explanation": "Performance optimization could involve multiple capabilities. Please specify: memory, disk, services, or network.", "hints": ["status.memory", "status.disk", "status.services"]}}

## USER INPUT

"{user_input}"

## YOUR OUTPUT (JSON ONLY)
"#,
        capabilities = capabilities,
        user_input = user_input
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_prompt_contains_all_capabilities() {
        let prompt = compiler_prompt("test input");

        // All capabilities from registry must be listed
        for cap in CAPABILITY_REGISTRY.list() {
            assert!(
                prompt.contains(cap.id.as_str()),
                "Prompt missing capability: {}",
                cap.id
            );
        }
    }

    #[test]
    fn test_compiler_prompt_contains_schema_elements() {
        let prompt = compiler_prompt("test");

        // Must contain the three outcome types
        assert!(prompt.contains("\"outcome\": \"Resolved\""));
        assert!(prompt.contains("\"outcome\": \"Abstained\""));
        assert!(prompt.contains("\"outcome\": \"Failed\""));

        // Must contain all AbstainReason codes
        assert!(prompt.contains("NO_MATCHING_CAPABILITY"));
        assert!(prompt.contains("PREREQUISITES_NOT_MET"));
        assert!(prompt.contains("EXECUTION_GATE_BLOCKED"));
        assert!(prompt.contains("AMBIGUOUS_REQUEST"));
        assert!(prompt.contains("MALFORMED_REQUEST"));

        // Must contain all FailedReason codes
        assert!(prompt.contains("REGISTRY_INCONSISTENCY"));
        assert!(prompt.contains("MISSING_EXECUTION_RESULT"));
        assert!(prompt.contains("PROBE_ERROR"));
        assert!(prompt.contains("FORMATTING_ERROR"));
    }

    #[test]
    fn test_compiler_prompt_is_deterministic() {
        let input = "scale my gdm please";
        let prompt1 = compiler_prompt(input);
        let prompt2 = compiler_prompt(input);

        assert_eq!(prompt1, prompt2, "Prompt must be deterministic");
    }

    #[test]
    fn test_compiler_prompt_includes_user_input() {
        let input = "unique_test_input_12345";
        let prompt = compiler_prompt(input);

        assert!(
            prompt.contains(input),
            "Prompt must include the user input verbatim"
        );
    }

    #[test]
    fn test_compiler_prompt_forbids_prose() {
        let prompt = compiler_prompt("test");

        // Check for key constraint language
        assert!(prompt.contains("You are NOT an assistant"));
        assert!(prompt.contains("You MUST NOT"));
        assert!(prompt.contains("Emit prose"));
        assert!(prompt.contains("Help anyway"));
    }

    #[test]
    fn test_compiler_prompt_capability_list_sorted() {
        // Build capability list and verify it's sorted for determinism
        let list = build_capability_list();
        let lines: Vec<&str> = list.lines().collect();

        let mut sorted_lines = lines.clone();
        sorted_lines.sort();

        assert_eq!(
            lines, sorted_lines,
            "Capability list must be sorted for determinism"
        );
    }

    #[test]
    fn test_compiler_prompt_no_chatbot_language() {
        let prompt = compiler_prompt("test");

        // Must explicitly forbid chatbot behaviors
        let forbidden = [
            "I'd be happy to",
            "I can help",
            "Let me",
            "I'll",
            "Sure!",
            "Of course",
        ];

        // The prompt should contain instructions that forbid this, not examples of it
        // So we check that the prompt doesn't contain these as positive examples
        assert!(prompt.contains("You do NOT converse"));
        assert!(prompt.contains("pure function"));
    }
}
