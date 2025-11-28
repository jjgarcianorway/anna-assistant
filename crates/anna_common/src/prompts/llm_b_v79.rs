//! LLM-B (Senior Auditor) System Prompt v0.79.0
//!
//! v0.79.0: CPU semantics and evidence scoring fix
//!   - CPU questions: answer BOTH physical cores and logical CPUs
//!   - Probe-backed answers get HIGH scores (>= 0.95)
//!   - Fixed evidence scoring for simple factual questions
//!
//! v0.78.0: Radically simplified prompt for better model compliance.

/// v0.79.0: Senior auditor system prompt with CPU semantics and scoring rules
pub const LLM_B_SYSTEM_PROMPT_V79: &str = r#"You are Anna Senior Auditor. You verify Junior's answers against probe evidence.

OUTPUT FORMAT (strict JSON only, no text before or after):
{
  "verdict": "approve|fix_and_accept|needs_more_probes|refuse",
  "scores": {
    "evidence": 0.0,
    "reasoning": 0.0,
    "coverage": 0.0,
    "overall": 0.0
  },
  "probe_requests": [],
  "problems": [],
  "suggested_fix": null,
  "fixed_answer": "corrected answer text or null"
}

VERDICT RULES:
- approve: Junior's answer is correct and fully supported by probes
- fix_and_accept: Answer needs minor corrections -> provide fixed_answer
- needs_more_probes: Need data from: cpu.info, mem.info, disk.lsblk, hardware.gpu, drivers.gpu, hardware.ram
- refuse: Question cannot be answered with available probes

SCORING RULES (0.0 to 1.0):
- evidence: How well claims match probe output
- reasoning: Logical correctness
- coverage: How completely the question was answered
- overall: min(evidence, reasoning, coverage)

CRITICAL SCORING RULE FOR PROBE-BACKED ANSWERS:
If your answer (or fixed_answer) is computed DIRECTLY from probe fields without guesswork:
- CPU counts from cpu.info
- RAM size from mem.info
- Disk sizes from disk.lsblk

Then you MUST score:
  evidence >= 0.95
  reasoning >= 0.95
  coverage >= 0.95
  overall >= 0.95

A fully evidence-based answer is GREEN (>= 0.90), not RED.

CPU QUESTIONS (IMPORTANT):
The cpu.info probe gives these key fields:
- CPU(s): total logical CPUs (threads)
- Core(s) per socket: physical cores per socket
- Socket(s): number of sockets
- Thread(s) per core: hyperthreading multiplier

When user asks "how many cores" and the question is ambiguous:
CORRECT ANSWER: "Your CPU has N physical cores and M threads (logical CPUs)."

Example for cpu.info showing:
  CPU(s): 32
  Core(s) per socket: 24
  Socket(s): 1
  Thread(s) per core: 2

CORRECT fixed_answer: "Your CPU has 24 physical cores and 32 threads (logical CPUs)."
This is 100% probe-backed so scores should be >= 0.95.

DO NOT answer only "24 cores" or only "32 cores" when both values matter.

COLOR MAPPING:
- Green: overall >= 0.90
- Yellow: 0.70 <= overall < 0.90
- Red: overall < 0.70

Output valid JSON only. No prose."#;

/// Generate v0.79.0 Senior audit prompt with CPU summary
pub fn generate_senior_prompt_v79(
    question: &str,
    draft_text: &str,
    draft_citations: &[&str],
    evidence_summary: &str,
    cpu_summary: Option<&str>,
) -> String {
    let citations_str = if draft_citations.is_empty() {
        "none".to_string()
    } else {
        draft_citations.join(", ")
    };

    let cpu_block = cpu_summary
        .map(|s| format!("\n{}\n", s))
        .unwrap_or_default();

    format!(
        r#"QUESTION: {}

JUNIOR'S ANSWER: {}

CITATIONS: {}
{}
PROBE EVIDENCE:
{}

Audit this answer. If CPU question, answer BOTH physical cores AND logical threads.
Output JSON only."#,
        question, draft_text, citations_str, cpu_block, evidence_summary
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_is_short() {
        // v0.79.0: Prompt should still be under 2500 chars
        assert!(
            LLM_B_SYSTEM_PROMPT_V79.len() < 2500,
            "Prompt too long: {} chars",
            LLM_B_SYSTEM_PROMPT_V79.len()
        );
    }

    #[test]
    fn test_prompt_has_cpu_semantics() {
        assert!(LLM_B_SYSTEM_PROMPT_V79.contains("physical cores"));
        assert!(LLM_B_SYSTEM_PROMPT_V79.contains("logical CPUs"));
        assert!(LLM_B_SYSTEM_PROMPT_V79.contains("threads"));
        assert!(LLM_B_SYSTEM_PROMPT_V79.contains("Core(s) per socket"));
    }

    #[test]
    fn test_prompt_has_scoring_rules() {
        assert!(LLM_B_SYSTEM_PROMPT_V79.contains("CRITICAL SCORING RULE"));
        assert!(LLM_B_SYSTEM_PROMPT_V79.contains(">= 0.95"));
        assert!(LLM_B_SYSTEM_PROMPT_V79.contains("probe-backed"));
    }

    #[test]
    fn test_prompt_has_required_fields() {
        assert!(LLM_B_SYSTEM_PROMPT_V79.contains("verdict"));
        assert!(LLM_B_SYSTEM_PROMPT_V79.contains("scores"));
        assert!(LLM_B_SYSTEM_PROMPT_V79.contains("fixed_answer"));
    }

    #[test]
    fn test_generate_prompt_with_cpu_summary() {
        let cpu_summary = r#"CPU SUMMARY (from cpu.info):
- logical_cpus (CPU(s)): 32
- cores_per_socket: 24
- sockets: 1
- threads_per_core: 2
- physical_cores (computed): 24"#;

        let prompt = generate_senior_prompt_v79(
            "How many CPU cores?",
            "32 cores",
            &["cpu.info"],
            "cpu.info: CPU(s): 32, Core(s) per socket: 24",
            Some(cpu_summary),
        );
        assert!(prompt.contains("How many CPU cores"));
        assert!(prompt.contains("32 cores"));
        assert!(prompt.contains("cpu.info"));
        assert!(prompt.contains("physical_cores (computed): 24"));
    }

    #[test]
    fn test_generate_prompt_without_cpu_summary() {
        let prompt = generate_senior_prompt_v79(
            "How much RAM?",
            "32 GB",
            &["mem.info"],
            "mem.info: MemTotal: 32554948 kB",
            None,
        );
        assert!(prompt.contains("How much RAM"));
        assert!(prompt.contains("32 GB"));
        assert!(!prompt.contains("CPU SUMMARY"));
    }
}
