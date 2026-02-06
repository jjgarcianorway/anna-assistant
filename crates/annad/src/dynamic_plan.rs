//! Dynamic Plan Generation - LLM generates ActionPlans for system config requests.
//! Phase 37: Instead of hardcoded handlers, LLM figures out what to do.

use anna_shared::action_plan::{ActionPlan, ActionStep};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Risk level for a generated plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    /// Safe to execute immediately (config files, dconf, reversible changes)
    Low,
    /// Requires confirmation (packages, services, system files)
    High,
    /// Cannot execute (destructive, security-sensitive)
    Blocked,
}

/// Paths/patterns that are LOW risk (config tweaks, reversible).
const LOW_RISK_PATHS: &[&str] = &[
    // Config directories
    "/etc/dconf/",
    "/etc/systemd/logind.conf",
    "/etc/systemd/logind.conf.d/",
    "/var/lib/gdm/.config/",
    "/var/lib/gdm3/.config/",
    // GNOME/dconf commands
    "dconf update",
    "dconf write",
    "gsettings set",
    "gsettings reset",
    // Systemd commands (non-service)
    "systemctl daemon-reload",
    "loginctl",
    // Directory/file setup for configs
    "mkdir -p /var/lib/gdm",
    "mkdir -p /etc/dconf",
    "mkdir -p /etc/systemd/logind.conf.d",
    // Display settings
    "xrandr",
    "wlr-randr",
    // Safe file operations on known config paths
    "cp ~/.config/monitors.xml /var/lib/gdm",
    "chown gdm:gdm /var/lib/gdm",
    // NetworkManager (read/status only)
    "nmcli connection show",
    "nmcli device status",
    // Timezone/locale (reversible)
    "timedatectl set-timezone",
    "localectl set-locale",
];

/// Paths/patterns that are HIGH risk (need confirmation).
const HIGH_RISK_PATHS: &[&str] = &[
    "/etc/passwd",
    "/etc/shadow",
    "/etc/group",
    "/etc/sudoers",
    "/etc/fstab",
    "pacman -S",
    "pacman -R",
    "systemctl enable",
    "systemctl disable",
    "systemctl start",
    "systemctl stop",
    "rm -rf",
    "dd if=",
    "mkfs",
    // Bootloader operations (HIGH risk - boot failure possible)
    "grub-install",
    "grub-mkconfig",
    "limine-deploy",
    "/boot/grub/grub.cfg",
    "/boot/limine.cfg",
    "/boot/loader/loader.conf",
    "bootctl install",
    "efibootmgr",
    // Snapshot operations (HIGH risk - system state changes)
    "snapper -c root create-config",
    "snapper undochange",
    "btrfs subvolume",
];

/// Paths/patterns that are BLOCKED (never execute).
const BLOCKED_PATTERNS: &[&str] = &[
    "rm -rf /",
    "dd if=/dev/zero of=/dev/sd",
    "chmod 777 /",
    ":(){ :|:& };:",
    "> /dev/sda",
    "mkfs.ext4 /dev/sd",
];

/// Assess risk level of a command.
pub fn assess_command_risk(command: &str) -> RiskLevel {
    let cmd_lower = command.to_lowercase();

    // Check blocked patterns first
    for pattern in BLOCKED_PATTERNS {
        if cmd_lower.contains(pattern) {
            return RiskLevel::Blocked;
        }
    }

    // Check high risk patterns
    for pattern in HIGH_RISK_PATHS {
        if cmd_lower.contains(&pattern.to_lowercase()) {
            return RiskLevel::High;
        }
    }

    // Check low risk patterns - if matches any, it's low risk
    for pattern in LOW_RISK_PATHS {
        if cmd_lower.contains(&pattern.to_lowercase()) {
            return RiskLevel::Low;
        }
    }

    // Default to high risk for unknown commands
    RiskLevel::High
}

/// Assess overall risk of a plan.
pub fn assess_plan_risk(plan: &ActionPlan) -> RiskLevel {
    let mut highest_risk = RiskLevel::Low;

    for step in &plan.steps {
        let step_risk = assess_command_risk(&step.command);
        match step_risk {
            RiskLevel::Blocked => return RiskLevel::Blocked,
            RiskLevel::High => highest_risk = RiskLevel::High,
            RiskLevel::Low => {}
        }
    }

    highest_risk
}

/// LLM response format for plan generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmPlanResponse {
    pub can_help: bool,
    pub reason: Option<String>,
    pub steps: Vec<LlmPlanStep>,
    #[serde(default, deserialize_with = "deserialize_verification")]
    pub verification: Option<String>,
}

/// Deserialize verification - handle both string and array.
fn deserialize_verification<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum VerificationValue {
        Single(String),
        Multiple(Vec<String>),
        Null,
    }

    match Option::<VerificationValue>::deserialize(deserializer)? {
        Some(VerificationValue::Single(s)) => Ok(Some(s)),
        Some(VerificationValue::Multiple(cmds)) => {
            // Join multiple verification commands with &&
            Ok(Some(cmds.join(" && ")))
        }
        Some(VerificationValue::Null) | None => Ok(None),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmPlanStep {
    pub description: String,
    pub command: String,
    pub needs_sudo: bool,
}

/// LLM verification response format.
/// v0.3.140: Self-verification loop for plan completeness.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmVerificationResponse {
    pub is_complete: bool,
    #[serde(default)]
    pub issues: Vec<String>,
    #[serde(default)]
    pub missing_steps: Vec<String>,
    #[serde(default)]
    pub suggestions: Vec<String>,
}

/// System prompt for plan generation.
/// v0.3.135: Strengthened JSON format requirement.
pub const PLAN_GENERATION_PROMPT: &str = r#"You are Anna, an Arch Linux system administrator. Extract commands from documentation to fulfill user requests.

CRITICAL: You MUST respond with ONLY valid JSON. No explanations before or after. Just JSON.

Your capabilities:
- Bootloader: install, replace (GRUB→limine, GRUB→systemd-boot), configure
- Snapper: install, configure, snapshot management
- System config: packages, services, settings
- Network, display, desktop configuration

BOOTLOADER REPLACEMENT - LIMINE INSTALLATION (Based on Arch Wiki):
CRITICAL: Adapt to ACTUAL current bootloader detected in investigation (GRUB/systemd-boot/rEFInd).

COMPLETE PROCEDURE (all steps required):
1. Backup current bootloader:
   - If "GRUB detected": cp -r /boot/grub /boot/grub.backup
   - If "systemd-boot detected": cp -r /boot/loader /boot/loader.backup
   - If "rEFInd detected": cp -r /boot/EFI/refind /boot/EFI/refind.backup

2. Get system values (use REAL values from investigation):
   ROOT_UUID=$(findmnt -n -o UUID /)
   BOOT_DEV=$(lsblk -ndo pkname $(findmnt -n -o SOURCE /boot))

3. Install limine:
   pacman -S --noconfirm limine

4. Deploy limine bootloader:
   limine-deploy /dev/$BOOT_DEV

5. Create /boot/limine.cfg (MUST include ALL kernel parameters from investigation):
   Example format:
   TIMEOUT=5

   :Arch Linux
       PROTOCOL=linux
       KERNEL_PATH=boot:///vmlinuz-linux
       CMDLINE=root=UUID=[real-uuid] rw [ALL kernel params from /proc/cmdline]
       MODULE_PATH=boot:///initramfs-linux.img

6. For UEFI systems (if "Boot mode: UEFI"):
   Create boot entry: efibootmgr --create --disk /dev/$BOOT_DEV --part [boot-partition-number] --loader '\EFI\BOOT\BOOTX64.EFI' --label 'Limine'

7. Verify:
   efibootmgr (check entry exists)

NEVER skip steps. ALL steps required for safe installation.

SNAPPER SETUP (example: "setup snapper", "enable snapshots"):
Based on Arch Wiki:
1. Check filesystem: findmnt -n -o FSTYPE / (must be btrfs)
2. Install: pacman -S --noconfirm snapper snap-pac grub-btrfs
3. Create config: snapper -c root create-config /
4. Enable timers: systemctl enable --now snapper-timeline.timer snapper-cleanup.timer
5. Verify: snapper list-configs

JSON RESPONSE FORMAT (respond with ONLY this, nothing else):
{
  "can_help": true,
  "reason": null,
  "steps": [
    {
      "description": "Backup current bootloader",
      "command": "cp -r /boot/grub /boot/grub.backup",
      "needs_sudo": true
    }
  ],
  "verification": "efibootmgr | grep -i limine"
}

If you cannot help, set can_help=false and provide reason. Otherwise, extract exact commands from the documentation above.

REMEMBER: Output ONLY JSON. No markdown, no explanations, just the JSON object.

User request: "#;

/// Plan verification prompt - LLM cross-checks plan against facts.
/// v0.3.140: Self-verification loop for reliability.
pub const PLAN_VERIFICATION_PROMPT: &str = r#"You are reviewing a system configuration plan. Check if it's complete and correct.

CRITICAL: You MUST respond with ONLY valid JSON. No explanations before or after.

Your task:
1. Compare the plan against the investigation findings
2. Compare the plan against the wiki documentation
3. Identify any missing steps or incorrect syntax
4. Determine if the plan is complete

JSON RESPONSE FORMAT:
{
  "is_complete": true,
  "issues": [],
  "missing_steps": [],
  "suggestions": []
}

OR if incomplete:
{
  "is_complete": false,
  "issues": [
    "Investigation shows UEFI but plan lacks efibootmgr command",
    "Limine config syntax doesn't match wiki example"
  ],
  "missing_steps": [
    "Create UEFI boot entry with efibootmgr"
  ],
  "suggestions": [
    "Add: efibootmgr --create --disk /dev/X --part Y --loader '\\EFI\\BOOT\\BOOTX64.EFI' --label 'Limine'"
  ]
}

CHECKLIST:
- If "Boot mode: UEFI" in investigation → Plan must include efibootmgr
- If "Boot mode: BIOS" in investigation → Plan should use legacy boot steps
- All kernel parameters from investigation must be in the config file
- All UUIDs/device names must be real values from investigation (no variables)
- Config file syntax must match wiki examples
- All wiki steps should be present

BOOTLOADER REPLACEMENT SPECIFIC (v0.3.141 - SAFETY FIRST):
- Investigation must show which bootloader is CURRENTLY installed (GRUB/systemd-boot/rEFInd)
- Backup step must match the ACTUAL current bootloader (not assumed)
- Plan must include creating the new bootloader's config file (e.g., /boot/limine.cfg)
- Config file must contain ALL kernel parameters from investigation
- For UEFI: Must include efibootmgr command to create boot entry
- Verification step must confirm new bootloader is bootable
- Plan should explain what's being replaced (e.g., "Replacing systemd-boot with limine")

NEVER ASSUME - VERIFY:
- Don't assume GRUB - check investigation for "GRUB detected"
- Don't assume systemd-boot - check investigation for "systemd-boot detected"
- Safety first, even if verification takes longer

REMEMBER: Output ONLY JSON. No markdown, no explanations."#;

/// Parse LLM response into ActionPlan.
pub fn parse_llm_plan(response: &str, original_request: &str) -> Option<ActionPlan> {
    use tracing::{debug, warn};

    // Try to extract JSON from response
    let json_str = match extract_json(response) {
        Some(j) => {
            debug!("Extracted JSON ({} chars)", j.len());
            j
        }
        None => {
            warn!("Could not extract JSON from LLM response");
            return None;
        }
    };

    let llm_plan: LlmPlanResponse = match serde_json::from_str(&json_str) {
        Ok(p) => p,
        Err(e) => {
            warn!("JSON parse error: {}", e);
            warn!("Attempted to parse: {}", if json_str.len() > 500 { &json_str[..500] } else { &json_str });
            return None;
        }
    };

    if !llm_plan.can_help {
        use tracing::info;
        info!("LLM declined to help: {:?}", llm_plan.reason);
        return None;
    }

    if llm_plan.steps.is_empty() {
        warn!("LLM returned empty steps");
        return None;
    }

    // Build ActionPlan from LLM response
    let summary = if original_request.len() > 50 {
        format!("{}...", &original_request[..47])
    } else {
        original_request.to_string()
    };

    let mut plan = ActionPlan::new(
        "llm-generated",
        &summary,
        &format!("Execute: {}", summary),
    );

    for step in llm_plan.steps {
        plan.add_step_full(
            ActionStep::new(&step.description, &step.command, step.needs_sudo)
        );
    }

    if let Some(verify_cmd) = llm_plan.verification {
        plan.set_verification(&verify_cmd, "", "Verify changes applied");
    }

    Some(plan)
}

/// Find the end of a JSON object by counting braces.
fn find_json_end(s: &str) -> Option<usize> {
    let mut depth = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, ch) in s.char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if in_string => escape_next = true,
            '"' => in_string = !in_string,
            '{' if !in_string => depth += 1,
            '}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Extract JSON from LLM response (handles markdown code blocks, mixed text).
/// v0.3.135: Much more robust - handles common LLM output formats.
fn extract_json(response: &str) -> Option<String> {
    use tracing::debug;

    let trimmed = response.trim();

    // 1. Try direct parse first (clean JSON at start)
    if trimmed.starts_with('{') {
        debug!("Trying direct JSON parse (starts with brace)");
        if let Some(end) = find_json_end(trimmed) {
            let json = &trimmed[..=end];
            debug!("Extracted JSON by brace counting: {} chars", json.len());
            return Some(json.to_string());
        }
    }

    // 2. Look for ```json code blocks
    if let Some(start) = response.find("```json") {
        debug!("Found ```json code block");
        let rest = &response[start + 7..];
        if let Some(end) = rest.find("```") {
            let json = rest[..end].trim();
            debug!("Extracted from ```json block: {} chars", json.len());
            return Some(json.to_string());
        }
    }

    // 3. Look for ``` code blocks starting with {
    for pattern in &["```\n{", "```{", "``` {"] {
        if let Some(start) = response.find(pattern) {
            debug!("Found generic code block: {}", pattern);
            let rest = &response[start + pattern.len() - 1..]; // Keep the {
            if let Some(end) = rest.find("```") {
                let json = rest[..end].trim();
                debug!("Extracted from ``` block: {} chars", json.len());
                return Some(json.to_string());
            }
        }
    }

    // 4. Search for first { and match to last }
    if let Some(start) = response.find('{') {
        debug!("Searching for JSON starting at position {}", start);
        let rest = &response[start..];
        if let Some(end) = find_json_end(rest) {
            let json = &rest[..=end];
            debug!("Extracted JSON from mixed text: {} chars", json.len());
            return Some(json.to_string());
        }
    }

    debug!("No JSON found in response");
    None
}

/// Check if a request looks like a system config request.
pub fn is_config_request(question: &str) -> bool {
    let q = question.to_lowercase();

    // Config/settings keywords
    let config_keywords = [
        "prevent", "disable", "enable", "stop", "configure", "set up", "setup",
        "change", "modify", "turn off", "turn on", "make", "don't let", "keep",
        "sleep", "suspend", "hibernate", "blank", "dim", "idle", "timeout",
        "scale", "resolution", "display", "screen", "monitor",
        "login", "gdm", "greeter", "lock",
        "theme", "appearance", "font", "cursor",
        "keyboard", "mouse", "touchpad",
        "network", "wifi", "bluetooth",
        "sound", "audio", "volume",
        "power", "battery", "lid",
    ];

    // Must contain at least one config keyword
    let has_config_keyword = config_keywords.iter().any(|k| q.contains(k));

    // Exclude pure questions (how does X work, what is X)
    let is_pure_question = q.starts_with("what is ") ||
                          q.starts_with("how does ") ||
                          q.starts_with("why does ") ||
                          q.starts_with("explain ");

    has_config_keyword && !is_pure_question
}

/// Parse LLM verification response.
/// v0.3.140: Self-verification loop for plan completeness.
pub fn parse_verification_response(response: &str) -> Option<LlmVerificationResponse> {
    use tracing::{debug, warn};

    // Try to extract JSON from response
    let json_str = match extract_json(response) {
        Some(j) => {
            debug!("Extracted verification JSON ({} chars)", j.len());
            j
        }
        None => {
            warn!("Could not extract JSON from verification response");
            return None;
        }
    };

    match serde_json::from_str(&json_str) {
        Ok(v) => Some(v),
        Err(e) => {
            warn!("Verification JSON parse error: {}", e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_assessment_low() {
        assert_eq!(assess_command_risk("dconf update"), RiskLevel::Low);
        assert_eq!(assess_command_risk("mkdir -p /etc/dconf/db/gdm.d"), RiskLevel::Low);
        assert_eq!(assess_command_risk("tee /etc/systemd/logind.conf.d/lid.conf"), RiskLevel::Low);
    }

    #[test]
    fn test_risk_assessment_high() {
        assert_eq!(assess_command_risk("pacman -S firefox"), RiskLevel::High);
        assert_eq!(assess_command_risk("systemctl enable sshd"), RiskLevel::High);
    }

    #[test]
    fn test_risk_assessment_blocked() {
        assert_eq!(assess_command_risk("rm -rf /"), RiskLevel::Blocked);
        assert_eq!(assess_command_risk("dd if=/dev/zero of=/dev/sda"), RiskLevel::Blocked);
    }

    #[test]
    fn test_is_config_request() {
        assert!(is_config_request("prevent GDM from sleeping"));
        assert!(is_config_request("disable screen blanking"));
        assert!(is_config_request("stop my laptop from suspending"));
        assert!(!is_config_request("what is systemd"));
        assert!(!is_config_request("how does dconf work"));
    }

    #[test]
    fn test_extract_json() {
        let direct = r#"{"can_help": true, "steps": []}"#;
        assert!(extract_json(direct).is_some());

        let code_block = "Here's the plan:\n```json\n{\"can_help\": true}\n```";
        assert!(extract_json(code_block).is_some());
    }
}
