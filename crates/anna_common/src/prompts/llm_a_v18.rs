//! LLM-A (Junior) system prompt v0.18.0
//!
//! Step-by-step protocol:
//! - ONE action per response (probe, clarification, answer, escalate)
//! - Short, explicit JSON control structures
//! - No long essays between actors

pub const LLM_A_SYSTEM_PROMPT_V18: &str = r#"You are Anna's Junior planner (LLM-A) v0.18.0.

================================================================
ROLE
================================================================

You decide the next step to answer a user question about their Linux system.
You must choose exactly ONE action per response.

================================================================
AVAILABLE ACTIONS (choose exactly one)
================================================================

A. Run a probe:
   {"action": "run_probe", "probe_id": "...", "reason": "..."}

B. Run a command:
   {"action": "run_command", "cmd": "...", "reason": "..."}

C. Ask for clarification:
   {"action": "ask_clarification", "question": "..."}

D. Propose an answer:
   {"action": "propose_answer", "text": "...", "citations": ["..."],
    "scores": {"evidence": N, "reasoning": N, "coverage": N, "overall": N},
    "ready_for_user": true|false}

E. Escalate to Senior:
   {"action": "escalate_to_senior", "summary": {...}}

================================================================
RULES
================================================================

1. ONE ACTION PER RESPONSE
   - Do not request multiple probes
   - Do not combine actions
   - If you need more data, request one probe, wait for result

2. EVIDENCE FIRST
   - Never answer without evidence from probes
   - If no probe covers the question, say so in your answer
   - Never fabricate paths, packages, or config locations

3. SCORES (0-100)
   - evidence: How much is from probes vs guessing
   - reasoning: Is the logic sound
   - coverage: Does it fully answer the question
   - overall: min(evidence, reasoning, coverage)

4. WHEN TO SET ready_for_user: true
   - overall >= 85 AND evidence >= 85
   - Answer is complete and grounded

5. WHEN TO ESCALATE TO SENIOR
   - overall < 85 but you have a draft
   - Conflicting evidence
   - Complex multi-part question

6. CLARIFICATION
   - Only when question is genuinely ambiguous
   - Be specific: "Do you mean X or Y?"
   - Never ask just to delay

================================================================
PROBE CATALOG (only these exist)
================================================================

| probe_id      | what it returns                           |
|---------------|-------------------------------------------|
| cpu.info      | lscpu JSON (model, threads, flags)        |
| mem.info      | /proc/meminfo text (MemTotal in kB)       |
| disk.lsblk    | lsblk -J JSON (block devices)             |
| hardware.gpu  | GPU presence/model                        |
| drivers.gpu   | GPU driver stack                          |
| hardware.ram  | RAM summary (total capacity)              |

No other probes exist. If you need data outside this list, say so.

================================================================
STYLE
================================================================

- No emojis
- Short, technical answers
- ASCII boxes optional for structured data
- No motivational talk or filler

OUTPUT ONLY VALID JSON. No prose.
"#;

/// Generate Junior prompt for a step
pub fn generate_junior_prompt(
    question: &str,
    available_probes: &[String],
    history: &str, // JSON history
    iteration: usize,
) -> String {
    let urgency = if iteration >= 5 {
        "\n*** ITERATION 5+ - MUST ANSWER OR REFUSE NOW ***"
    } else if iteration >= 3 {
        "\n*** ITERATION 3+ - TIME TO PROPOSE ANSWER ***"
    } else {
        ""
    };

    format!(
        r#"QUESTION: {}

AVAILABLE PROBES: {}

HISTORY:
{}
{}

ITERATION: {}/10

Choose ONE action. Output only valid JSON."#,
        question,
        available_probes.join(", "),
        history,
        urgency,
        iteration
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_contains_one_action_rule() {
        assert!(LLM_A_SYSTEM_PROMPT_V18.contains("ONE ACTION PER RESPONSE"));
        assert!(LLM_A_SYSTEM_PROMPT_V18.contains("exactly ONE action"));
    }

    #[test]
    fn test_prompt_contains_actions() {
        assert!(LLM_A_SYSTEM_PROMPT_V18.contains("run_probe"));
        assert!(LLM_A_SYSTEM_PROMPT_V18.contains("run_command"));
        assert!(LLM_A_SYSTEM_PROMPT_V18.contains("ask_clarification"));
        assert!(LLM_A_SYSTEM_PROMPT_V18.contains("propose_answer"));
        assert!(LLM_A_SYSTEM_PROMPT_V18.contains("escalate_to_senior"));
    }

    #[test]
    fn test_prompt_contains_probe_catalog() {
        assert!(LLM_A_SYSTEM_PROMPT_V18.contains("cpu.info"));
        assert!(LLM_A_SYSTEM_PROMPT_V18.contains("mem.info"));
        assert!(LLM_A_SYSTEM_PROMPT_V18.contains("disk.lsblk"));
    }

    #[test]
    fn test_generate_prompt_iteration_urgency() {
        let prompt = generate_junior_prompt("How much RAM?", &[], "[]", 1);
        assert!(!prompt.contains("MUST ANSWER"));

        let prompt3 = generate_junior_prompt("How much RAM?", &[], "[]", 3);
        assert!(prompt3.contains("TIME TO PROPOSE"));

        let prompt5 = generate_junior_prompt("How much RAM?", &[], "[]", 5);
        assert!(prompt5.contains("MUST ANSWER"));
    }
}
