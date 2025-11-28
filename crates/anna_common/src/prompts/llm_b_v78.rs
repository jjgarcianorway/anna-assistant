//! LLM-B (Senior Auditor) Minimal System Prompt v0.78.0
//!
//! v0.78.0: Radically simplified prompt for better model compliance.
//! The old 280-line prompt was being ignored by the model.
//! This version is ~40 lines with strict JSON format only.

/// v0.78.0: Minimal Senior auditor system prompt
/// Dramatically reduced from ~10k chars to ~1.5k chars
pub const LLM_B_SYSTEM_PROMPT_V78: &str = r#"You are Anna Senior Auditor. You verify Junior's answers against probe evidence.

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
- fix_and_accept: Answer is mostly right but needs minor corrections -> provide fixed_answer
- needs_more_probes: Need data from: cpu.info, mem.info, disk.lsblk, hardware.gpu, drivers.gpu, hardware.ram
- refuse: Question cannot be answered with available probes

SCORING (0.0 to 1.0):
- evidence: How well claims match probe output
- reasoning: Logical correctness
- coverage: How completely the question was answered
- overall: min(evidence, reasoning, coverage)

RULES:
- All fields are REQUIRED
- overall >= 0.9 = Green, 0.7-0.89 = Yellow, < 0.7 = Red
- If Junior's answer matches the probe data, approve with high scores
- If answer has small errors, use fix_and_accept and provide corrected text
- ONLY request probes from: cpu.info, mem.info, disk.lsblk, hardware.gpu, drivers.gpu, hardware.ram
- For simple factual questions with good evidence, approve with scores around 0.95

Output valid JSON only. No prose."#;

/// Generate v0.78.0 Senior audit prompt (simplified)
pub fn generate_senior_prompt_v78(
    question: &str,
    draft_text: &str,
    draft_citations: &[&str],
    evidence_summary: &str,
) -> String {
    let citations_str = if draft_citations.is_empty() {
        "none".to_string()
    } else {
        draft_citations.join(", ")
    };

    format!(
        r#"QUESTION: {}

JUNIOR'S ANSWER: {}

CITATIONS: {}

PROBE EVIDENCE:
{}

Audit this answer. Output JSON only."#,
        question, draft_text, citations_str, evidence_summary
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_is_short() {
        // v0.78.0: Prompt should be under 2000 chars (was ~10k before)
        assert!(
            LLM_B_SYSTEM_PROMPT_V78.len() < 2000,
            "Prompt too long: {} chars",
            LLM_B_SYSTEM_PROMPT_V78.len()
        );
    }

    #[test]
    fn test_prompt_has_required_fields() {
        assert!(LLM_B_SYSTEM_PROMPT_V78.contains("verdict"));
        assert!(LLM_B_SYSTEM_PROMPT_V78.contains("scores"));
        assert!(LLM_B_SYSTEM_PROMPT_V78.contains("evidence"));
        assert!(LLM_B_SYSTEM_PROMPT_V78.contains("fixed_answer"));
    }

    #[test]
    fn test_generate_prompt() {
        let prompt = generate_senior_prompt_v78(
            "How many CPU cores?",
            "32 cores",
            &["cpu.info"],
            "cpu.info: CPU(s): 32",
        );
        assert!(prompt.contains("How many CPU cores"));
        assert!(prompt.contains("32 cores"));
        assert!(prompt.contains("cpu.info"));
    }
}
