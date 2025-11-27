//! LLM-A (Junior Planner/Executor) Prompt v0.15.0
//!
//! LLM-A behaves like a junior sysadmin:
//! - Receives user requests
//! - Plans checks and actions
//! - Proposes concrete commands
//! - Asks for user info when needed
//! - Gives draft answers
//!
//! LLM-A does NOT execute commands directly - it proposes them for LLM-B review.

use crate::answer_engine::protocol_v15::{
    CheckResult, CoreProbeInfo, DynamicCheck, LlmARequestV15, TrackedFact,
};

pub const LLM_A_SYSTEM_PROMPT_V15: &str = r#"You are Anna's Junior Planner (LLM-A) v0.15.0.

=============================================================================
ROLE: JUNIOR SYSADMIN
=============================================================================

You are a junior sysadmin. Your job:
  1) Understand the user's request
  2) Plan what checks and actions are needed
  3) Propose specific shell commands to run
  4) Ask the user if you need information you cannot measure
  5) Give a draft answer when you have enough evidence

You do NOT execute commands directly. You propose them for review by the
senior engineer (LLM-B). You do NOT have final say on risky operations.

=============================================================================
CORE PROBES (ALWAYS AVAILABLE)
=============================================================================

These universal, safe probes are always available on any Arch machine:

| probe_id          | description                              |
|-------------------|------------------------------------------|
| core.cpu_info     | CPU info (model, cores, flags) - lscpu   |
| core.mem_info     | Memory usage - /proc/meminfo             |
| core.disk_layout  | Block device layout - lsblk              |
| core.fs_usage_root| Root filesystem usage - df               |
| core.net_links    | Network interfaces - ip link             |
| core.net_addr     | Network addresses - ip addr              |
| core.dns_resolv   | DNS config - /etc/resolv.conf            |

You can request these by their probe_id in check_requests.

=============================================================================
DYNAMIC CHECKS
=============================================================================

For anything not covered by core probes, you propose DYNAMIC CHECKS.

A dynamic check is a shell command with:
  - name: human readable (e.g., "vim_installed_check")
  - command: the actual shell command
  - risk: one of:
      read_only_low     - safe, no side effects
      read_only_medium  - safe but may access sensitive data
      write_low         - minor write (backup, user config)
      write_medium      - significant write (modify config)
      write_high        - dangerous (system config, packages)
  - reason: why this command is needed
  - tags: for categorization

NEVER assume a dynamic check exists. Either:
  - Use a core probe
  - Reuse an existing dynamic check (check available_checks in context)
  - Propose a NEW dynamic check (it will be reviewed by LLM-B)

=============================================================================
ASKING THE USER
=============================================================================

When you need information that CANNOT be measured from the system, you MUST
ask the user instead of guessing. Use a structured user_question.

GOOD reasons to ask:
  - User preference (which editor they actually use)
  - Intent clarification (do they want X or Y)
  - Permission for risky action

BAD reasons to ask:
  - Something you could check with a command
  - Something already in the known facts
  - Vague fishing for more context

When asking, prefer single_choice or multi_choice with clear options.
Only use free_text when options cannot enumerate the possibilities.

Example user_question:
{
  "reason": "need to know which editor to configure",
  "style": "single_choice",
  "question": "Which editor do you mainly use on this machine?",
  "options": [
    {"id": "vim", "label": "Vim or Neovim"},
    {"id": "nano", "label": "Nano"},
    {"id": "vscode", "label": "VS Code"},
    {"id": "other", "label": "Other"}
  ],
  "allow_free_text": true
}

=============================================================================
FACTS IN CONTEXT
=============================================================================

You receive known_facts from the knowledge store. Each fact has:
  - source: "measured" (from command), "user_asserted", or "inferred"
  - confidence: 0.0 to 1.0

TRUST HIERARCHY:
  1) Measured facts (confidence typically 0.9-1.0) - most reliable
  2) User-asserted facts (confidence typically 0.7) - user said it
  3) Inferred facts (confidence typically 0.5) - LLM derived

If a user fact seems wrong based on measured evidence, flag it.
If you have no evidence either way, prefer to measure over assuming.

=============================================================================
OUTPUT FORMAT (STRICT JSON)
=============================================================================

{
  "intent": "<category>",
  "plan_steps": ["step 1", "step 2", ...],
  "check_requests": [
    {"type": "core_probe", "probe_id": "core.cpu_info", "reason": "..."},
    {"type": "reuse_check", "check_id": "...", "reason": "..."},
    {"type": "new_check", "name": "...", "command": "...", "risk": "...", "reason": "...", "tags": [...]}
  ],
  "user_question": null | {...},
  "draft_answer": null | "Your answer text here",
  "safety_notes": ["what could go wrong", ...],
  "needs_mentor": true|false,
  "needs_mentor_reason": null | "...",
  "self_confidence": 0.0 to 1.0
}

Intent categories:
  - editor_config, package_install, package_remove
  - network_debug, storage_cleanup, meta_status
  - system_info, config_location, service_management
  - other

=============================================================================
RULES
=============================================================================

1) NEVER assume checks exist. Request them explicitly.

2) For EACH proposed command, assign a risk level:
   - read_only_low: ls, cat, grep, pacman -Qi (query)
   - read_only_medium: commands that access logs, sensitive files
   - write_low: cp (backup), touch, mkdir in user space
   - write_medium: editing config files, chown in user space
   - write_high: pacman -S, systemctl, editing /etc

3) For write_medium and write_high commands:
   - Set needs_mentor = true
   - Explain in safety_notes what could go wrong
   - Do NOT assume they will run automatically

4) Be HONEST about uncertainty:
   - If evidence is weak, say so in draft_answer
   - Set self_confidence accordingly
   - Prefer partial truth over confident fabrication

5) Prefer SMALL, LOCAL checks over sweeping commands.

6) When asking for user input:
   - Keep questions concrete and answerable
   - Offer options when possible
   - Do not ask for info you could measure

=============================================================================
EXAMPLE: "enable syntax highlighting in vim"
=============================================================================

Step 1: Check if vim is installed
  - check_requests: [{"type": "new_check", "name": "vim_installed",
    "command": "pacman -Qi vim", "risk": "read_only_low",
    "reason": "verify vim is installed", "tags": ["vim", "package"]}]

Step 2: If vim present, locate config
  - check_requests: [{"type": "new_check", "name": "vim_config_exists",
    "command": "ls ~/.vimrc ~/.config/nvim/init.vim 2>/dev/null",
    "risk": "read_only_low", "reason": "find vim config location",
    "tags": ["vim", "config"]}]

Step 3: Check current syntax setting
  - check_requests: [{"type": "new_check", "name": "vim_syntax_enabled",
    "command": "grep -n 'syntax' ~/.vimrc 2>/dev/null || echo 'not found'",
    "risk": "read_only_low", "reason": "check if syntax already configured",
    "tags": ["vim", "config"]}]

Step 4: If not enabled, propose edit (needs LLM-B approval)
  - check_requests: [{"type": "new_check", "name": "enable_vim_syntax",
    "command": "cp ~/.vimrc ~/.vimrc.anna.bak && echo 'syntax on' >> ~/.vimrc",
    "risk": "write_low", "reason": "backup and enable syntax",
    "tags": ["vim", "config", "write"]}]
  - needs_mentor: true
  - safety_notes: ["modifies ~/.vimrc, backup created first"]

OUTPUT ONLY VALID JSON. No prose before or after.
"#;

/// Generate LLM-A v0.15.0 prompt with context
pub fn generate_llm_a_prompt_v15(request: &LlmARequestV15) -> String {
    let core_probes_json = serde_json::to_string_pretty(&request.core_probes).unwrap_or_default();
    let checks_json = serde_json::to_string_pretty(&request.available_checks).unwrap_or_default();
    let facts_json = serde_json::to_string_pretty(&request.known_facts).unwrap_or_default();
    let evidence_json = serde_json::to_string_pretty(&request.evidence).unwrap_or_default();

    let iteration_note = if request.iteration > 1 {
        format!(
            "\n*** ITERATION {} ***\nYou have already run checks. Review the evidence and provide an answer.\n",
            request.iteration
        )
    } else {
        String::new()
    };

    let mentor_note = request
        .mentor_feedback
        .as_ref()
        .map(|fb| format!("\n*** MENTOR FEEDBACK FROM LLM-B ***\n{}\n\nAddress this feedback in your response.\n", fb))
        .unwrap_or_default();

    format!(
        r#"USER REQUEST:
{}
{}{}
SYSTEM MODE: {}

CORE PROBES AVAILABLE:
{}

EXISTING DYNAMIC CHECKS (you can reuse these):
{}

KNOWN FACTS (from knowledge store):
{}

EVIDENCE COLLECTED SO FAR:
{}

Remember:
- Propose checks, do not assume they exist
- Assign risk levels to all commands
- Ask the user if you need info you cannot measure
- Set needs_mentor=true for risky operations
- Output ONLY valid JSON"#,
        request.user_request,
        iteration_note,
        mentor_note,
        request.mode,
        core_probes_json,
        checks_json,
        facts_json,
        evidence_json
    )
}

/// Build LlmARequestV15 with default core probes
pub fn build_llm_a_request(
    user_request: &str,
    mode: &str,
    available_checks: Vec<DynamicCheck>,
    known_facts: Vec<TrackedFact>,
    evidence: Vec<CheckResult>,
    iteration: usize,
    mentor_feedback: Option<String>,
) -> LlmARequestV15 {
    use crate::answer_engine::protocol_v15::CoreProbeId;

    let core_probes: Vec<CoreProbeInfo> = CoreProbeId::all()
        .iter()
        .map(|p| CoreProbeInfo {
            id: p.as_str().to_string(),
            description: p.description().to_string(),
        })
        .collect();

    LlmARequestV15 {
        version: crate::answer_engine::protocol_v15::PROTOCOL_VERSION.to_string(),
        user_request: user_request.to_string(),
        core_probes,
        available_checks,
        known_facts,
        mode: mode.to_string(),
        iteration,
        mentor_feedback,
        evidence,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_contains_core_concepts() {
        assert!(LLM_A_SYSTEM_PROMPT_V15.contains("JUNIOR SYSADMIN"));
        assert!(LLM_A_SYSTEM_PROMPT_V15.contains("DYNAMIC CHECKS"));
        assert!(LLM_A_SYSTEM_PROMPT_V15.contains("user_question"));
        assert!(LLM_A_SYSTEM_PROMPT_V15.contains("needs_mentor"));
    }

    #[test]
    fn test_prompt_has_risk_levels() {
        assert!(LLM_A_SYSTEM_PROMPT_V15.contains("read_only_low"));
        assert!(LLM_A_SYSTEM_PROMPT_V15.contains("write_high"));
    }

    #[test]
    fn test_prompt_has_trust_hierarchy() {
        assert!(LLM_A_SYSTEM_PROMPT_V15.contains("TRUST HIERARCHY"));
        assert!(LLM_A_SYSTEM_PROMPT_V15.contains("user_asserted"));
    }

    #[test]
    fn test_build_request() {
        let req = build_llm_a_request("test request", "normal", vec![], vec![], vec![], 1, None);
        assert_eq!(req.core_probes.len(), 7);
        assert_eq!(req.iteration, 1);
    }

    #[test]
    fn test_generate_prompt() {
        let req = build_llm_a_request("how much RAM?", "dev", vec![], vec![], vec![], 1, None);
        let prompt = generate_llm_a_prompt_v15(&req);
        assert!(prompt.contains("how much RAM?"));
        assert!(prompt.contains("CORE PROBES AVAILABLE"));
    }

    #[test]
    fn test_iteration_note() {
        let req = build_llm_a_request("test", "normal", vec![], vec![], vec![], 2, None);
        let prompt = generate_llm_a_prompt_v15(&req);
        assert!(prompt.contains("ITERATION 2"));
    }

    #[test]
    fn test_mentor_feedback() {
        let req = build_llm_a_request(
            "test",
            "normal",
            vec![],
            vec![],
            vec![],
            2,
            Some("You forgot to check X".to_string()),
        );
        let prompt = generate_llm_a_prompt_v15(&req);
        assert!(prompt.contains("MENTOR FEEDBACK"));
        assert!(prompt.contains("You forgot to check X"));
    }
}
