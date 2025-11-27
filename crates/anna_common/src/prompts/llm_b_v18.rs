//! LLM-B (Senior) system prompt v0.18.0
//!
//! Senior is called when:
//! - Junior escalates (low confidence or needs audit)
//! - Complex multi-part questions
//! - Conflicting evidence

pub const LLM_B_SYSTEM_PROMPT_V18: &str = r#"You are Anna's Senior auditor (LLM-B) v0.18.0.

================================================================
ROLE
================================================================

You review Junior's work and produce the final answer.
You are called when Junior escalates or confidence is low.

================================================================
AVAILABLE RESPONSES (choose exactly one)
================================================================

A. Approve the answer:
   {"action": "approve_answer", "scores": {"evidence": N, "reasoning": N,
    "coverage": N, "overall": N, "reliability_note": "..."}}

B. Correct the answer:
   {"action": "correct_answer", "text": "...", "scores": {...},
    "corrections": ["what you fixed"]}

C. Request a probe:
   {"action": "request_probe", "probe_id": "...", "reason": "..."}

D. Request a command:
   {"action": "request_command", "cmd": "...", "reason": "..."}

E. Refuse to answer:
   {"action": "refuse", "reason": "...", "probes_attempted": ["..."]}

================================================================
RULES
================================================================

1. VERIFY ALL CLAIMS
   - Check every claim against probe outputs
   - If claim has no evidence, correct or remove it

2. SCORES (0-100)
   - evidence: % of claims backed by probes
   - reasoning: Is logic sound
   - coverage: Does it fully answer
   - overall: min(evidence, reasoning, coverage)

3. RELIABILITY NOTE
   - Brief explanation of evidence quality
   - Example: "strong evidence from cpu.info"
   - Example: "partial evidence, no network probes"

4. WHEN TO CORRECT
   - Wrong numbers (RAM size, CPU count)
   - Fabricated info (packages, configs not from probes)
   - Missing disclaimer for unsupported domains

5. WHEN TO REFUSE
   - No probe can answer this question
   - AND no useful heuristic to offer

6. PROBE CATALOG (only these exist)
   cpu.info, mem.info, disk.lsblk, hardware.gpu, drivers.gpu, hardware.ram

================================================================
AUDIT CHECKLIST
================================================================

For each claim in Junior's answer:

[ ] RAM claim? Check mem.info MemTotal (kB -> GB: divide by 1024^2)
[ ] CPU claim? Check cpu.info Model name, CPU(s), Flags
[ ] GPU claim? Check hardware.gpu, drivers.gpu
[ ] Disk claim? Check disk.lsblk
[ ] Network claim? NO PROBE - must say "no probe for network"
[ ] Package claim? NO PROBE - must say "no probe for packages"
[ ] Kernel claim? NO PROBE - must say "no probe for kernel"

================================================================
STYLE FOR CORRECTED ANSWERS
================================================================

- Short: 1-5 lines typical
- No emojis
- ASCII boxes optional
- Clearly state what you know vs don't know
- Example:

  Your CPU is an Intel Core i9-14900HX with 32 threads.
  [cpu.info]

OUTPUT ONLY VALID JSON. No prose.
"#;

/// Generate Senior prompt for audit
pub fn generate_senior_prompt(
    question: &str,
    history: &str, // JSON history
    junior_draft: Option<&str>,
    junior_scores: Option<&str>, // JSON scores
    escalation_reason: &str,
) -> String {
    let draft_section = match junior_draft {
        Some(draft) => format!("JUNIOR'S DRAFT:\n{}", draft),
        None => "JUNIOR'S DRAFT: None".to_string(),
    };

    let scores_section = match junior_scores {
        Some(scores) => format!("JUNIOR'S SCORES:\n{}", scores),
        None => "JUNIOR'S SCORES: None".to_string(),
    };

    format!(
        r#"ORIGINAL QUESTION: {}

HISTORY (probes and commands run):
{}

{}

{}

ESCALATION REASON: {}

Your task: Review and produce final answer. Output only valid JSON."#,
        question, history, draft_section, scores_section, escalation_reason
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_contains_responses() {
        assert!(LLM_B_SYSTEM_PROMPT_V18.contains("approve_answer"));
        assert!(LLM_B_SYSTEM_PROMPT_V18.contains("correct_answer"));
        assert!(LLM_B_SYSTEM_PROMPT_V18.contains("request_probe"));
        assert!(LLM_B_SYSTEM_PROMPT_V18.contains("refuse"));
    }

    #[test]
    fn test_prompt_contains_audit_checklist() {
        assert!(LLM_B_SYSTEM_PROMPT_V18.contains("AUDIT CHECKLIST"));
        assert!(LLM_B_SYSTEM_PROMPT_V18.contains("RAM claim"));
        assert!(LLM_B_SYSTEM_PROMPT_V18.contains("CPU claim"));
        assert!(LLM_B_SYSTEM_PROMPT_V18.contains("NO PROBE"));
    }

    #[test]
    fn test_generate_prompt_with_draft() {
        let prompt = generate_senior_prompt(
            "How much RAM?",
            "[]",
            Some("You have 32 GB RAM"),
            Some(r#"{"evidence": 90}"#),
            "Review requested",
        );
        assert!(prompt.contains("32 GB RAM"));
        assert!(prompt.contains("evidence"));
    }

    #[test]
    fn test_generate_prompt_without_draft() {
        let prompt = generate_senior_prompt(
            "How much RAM?",
            "[]",
            None,
            None,
            "No draft available",
        );
        assert!(prompt.contains("None"));
    }
}
