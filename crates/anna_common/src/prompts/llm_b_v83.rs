//! Senior (LLM-B) Prompts for v0.83.0 - Compact Escalation Handler
//!
//! v0.83.0 Performance Focus:
//! - Only used for escalation
//! - Provide corrected commands or clear refusal
//! - Never output verbose text
//! - Be decisive
//! - Score reliability aggressively
//! - Target: 6 second response time

/// v0.83.0 Senior system prompt - escalation only, compact, decisive
pub const LLM_B_SYSTEM_PROMPT_V83: &str = r#"You are Senior. Verify Junior's answer against probe evidence. Be DECISIVE and FAST.

OUTPUT (strict JSON only):
{"verdict":"approve|fix|refuse","score":0,"fix":"corrected answer or null"}

VERDICTS:
- approve: Answer correct, score>=90
- fix: Minor fix needed, provide "fix" field, score>=70
- refuse: Cannot answer, score<70

SCORING (0-100):
- 90-100: Fully probe-backed, Green
- 70-89: Partial evidence, Yellow
- 0-69: Insufficient/fabricated, Red, refuse

RULES:
- CPU: Report BOTH physical cores AND threads
- Fabricated claims: score=0, refuse
- Keep fixes SHORT. Max 2 sentences.
- No verbose explanations.
"#;

/// Generate compact Senior prompt for v0.83.0
pub fn generate_senior_prompt_v83(
    question: &str,
    draft_text: &str,
    evidence_summary: &str,
) -> String {
    format!(
        "Q:{question}\nA:{draft_text}\n\nE:\n{evidence_summary}\n\nVerify. JSON only."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v83_senior_prompt_is_compact() {
        // v0.83.0: System prompt should be under 700 chars (615 actual)
        assert!(
            LLM_B_SYSTEM_PROMPT_V83.len() < 700,
            "System prompt too long: {} chars",
            LLM_B_SYSTEM_PROMPT_V83.len()
        );
    }

    #[test]
    fn test_v83_senior_prompt_has_verdicts() {
        assert!(LLM_B_SYSTEM_PROMPT_V83.contains("approve"));
        assert!(LLM_B_SYSTEM_PROMPT_V83.contains("fix"));
        assert!(LLM_B_SYSTEM_PROMPT_V83.contains("refuse"));
    }

    #[test]
    fn test_v83_senior_prompt_has_scoring() {
        assert!(LLM_B_SYSTEM_PROMPT_V83.contains("90-100"));
        assert!(LLM_B_SYSTEM_PROMPT_V83.contains("70-89"));
        assert!(LLM_B_SYSTEM_PROMPT_V83.contains("0-69"));
    }

    #[test]
    fn test_v83_generated_senior_prompt_compact() {
        let prompt = generate_senior_prompt_v83(
            "How much RAM?",
            "32 GB",
            "mem.info: MemTotal: 32554948 kB",
        );
        assert!(prompt.len() < 150, "Generated prompt too long: {} chars", prompt.len());
        assert!(prompt.contains("Q:"));
        assert!(prompt.contains("A:"));
        assert!(prompt.contains("E:"));
    }
}
