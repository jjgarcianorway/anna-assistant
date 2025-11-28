//! Senior (LLM-B) Prompt v0.80.0 - Razorback Fast Path
//!
//! Minimal auditing prompt for razorback-fast profile.
//! Goal: Quick verification with clear verdict and score.

/// v0.80.0: Minimal Senior system prompt for razorback-fast profile
pub const LLM_B_SYSTEM_PROMPT_V80: &str = r#"You are Anna Senior Auditor.
You receive:
- the original user question
- Junior's draft answer
- a small set of probe summaries (not raw logs)

Your job is to:
1. check whether Junior's answer matches the probe summaries
2. optionally fix numbers or wording
3. return a short verdict and a fixed answer

Respond with valid JSON only:

{
  "verdict": "approve|fix_and_accept|refuse",
  "fixed_answer": "text or null",
  "scores": {
    "overall": 0.0
  }
}

Rules:
- approve when Junior is correct and complete.
- fix_and_accept when you change the answer slightly (for example 32 -> 24 cores).
- refuse when probes do not contain enough information to answer.
- fixed_answer:
  - for approve: copy Junior's text
  - for fix_and_accept: return the corrected text
  - for refuse: a short explanation
- scores.overall:
  - 0.95 when everything matches probes
  - ~0.7 when you fix minor things
  - <0.5 when you have to refuse
- No other fields, no markdown, no comments."#;

/// Generate the user prompt for Senior (razorback-fast profile)
///
/// Compact format:
/// ```text
/// QUESTION:
/// <user question>
///
/// JUNIOR_DRAFT:
/// <junior draft JSON>
///
/// PROBES_SUMMARY:
/// <small JSON snippets with just the key numbers>
///
/// Reply with a single JSON object.
/// ```
pub fn generate_senior_prompt_v80(
    question: &str,
    junior_draft: &str,
    probe_summaries: &[(&str, &str)], // (probe_id, compact_json)
) -> String {
    let mut prompt = format!(
        "QUESTION:\n{}\n\nJUNIOR_DRAFT:\n{}\n\n",
        question, junior_draft
    );

    prompt.push_str("PROBES_SUMMARY:\n");
    for (probe_id, summary) in probe_summaries {
        prompt.push_str(&format!("- {}: {}\n", probe_id, summary));
    }

    prompt.push_str("\nReply with a single JSON object.");
    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_senior_prompt() {
        let summaries = vec![
            ("cpu.info", r#"{"physical_cores":24,"threads_total":32}"#),
        ];
        let prompt = generate_senior_prompt_v80(
            "How many cores?",
            "Your computer has 24 cores.",
            &summaries,
        );

        assert!(prompt.contains("QUESTION:\nHow many cores?"));
        assert!(prompt.contains("JUNIOR_DRAFT:\nYour computer has 24 cores."));
        assert!(prompt.contains("cpu.info:"));
        assert!(prompt.contains("physical_cores"));
    }

    #[test]
    fn test_system_prompt_is_minimal() {
        // Razorback Senior prompt should be much shorter than the default
        assert!(LLM_B_SYSTEM_PROMPT_V80.len() < 1200);
    }
}
