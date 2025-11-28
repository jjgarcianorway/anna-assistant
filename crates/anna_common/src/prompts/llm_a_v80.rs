//! Junior (LLM-A) Prompt v0.80.0 - Razorback Fast Path
//!
//! Minimal prompt for razorback-fast profile.
//! Goal: Complete simple questions in <5 seconds with maximum reliability.

/// v0.80.0: Minimal Junior system prompt for razorback-fast profile
pub const LLM_A_SYSTEM_PROMPT_V80: &str = r#"You are Anna Junior.
Your job is to:
1. decide which probes to run
2. when evidence is available, propose a short draft answer

You must respond with valid JSON only, no comments or extra text.

JSON schema:

{
  "probe_requests": [
    {"probe_id": "cpu.info", "reason": "why this helps"}
  ],
  "draft_answer": {
    "text": "short answer or null if unknown",
    "citations": ["cpu.info"]
  }
}

Rules:
- You may request 0 to 3 probes from this list only:
  cpu.info, mem.info, disk.lsblk, hardware.gpu, drivers.gpu, hardware.ram
- If evidence is already provided in the prompt, do not request more probes. Use it and fill draft_answer.
- draft_answer.text must be a short, human-readable sentence or null.
- citations must list probe ids you actually used. Use [] if none.
- No markdown, no prose, only the JSON object."#;

/// Generate the user prompt for Junior (razorback-fast profile)
///
/// Short format:
/// ```text
/// PROBES: cpu.info, mem.info, ...
/// QUESTION: <user question>
/// EVIDENCE:
/// <compact probe summaries>
/// Reply with a single JSON object.
/// ```
pub fn generate_junior_prompt_v80(
    question: &str,
    available_probes: &[String],
    evidence_summaries: &[ProbeSummary],
) -> String {
    let probes_list = available_probes.join(", ");

    let mut prompt = format!(
        "PROBES: {}\n\nQUESTION: {}\n\n",
        probes_list, question
    );

    if !evidence_summaries.is_empty() {
        prompt.push_str("EVIDENCE:\n");
        for summary in evidence_summaries {
            prompt.push_str(&format!(
                "- {}: {}\n",
                summary.probe_id, summary.compact_json
            ));
        }
        prompt.push('\n');
    }

    prompt.push_str("Reply with a single JSON object.");
    prompt
}

/// Compact probe summary for Junior prompt
/// Pre-computed in Rust to reduce LLM context size
#[derive(Debug, Clone)]
pub struct ProbeSummary {
    pub probe_id: String,
    /// Compact JSON with just the key numbers (not the full raw output)
    pub compact_json: String,
}

impl ProbeSummary {
    pub fn new(probe_id: &str, compact_json: &str) -> Self {
        Self {
            probe_id: probe_id.to_string(),
            compact_json: compact_json.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_junior_prompt_no_evidence() {
        let probes = vec![
            "cpu.info".to_string(),
            "mem.info".to_string(),
        ];
        let prompt = generate_junior_prompt_v80("How many CPU cores?", &probes, &[]);

        assert!(prompt.contains("PROBES: cpu.info, mem.info"));
        assert!(prompt.contains("QUESTION: How many CPU cores?"));
        assert!(!prompt.contains("EVIDENCE:"));
        assert!(prompt.contains("Reply with a single JSON object"));
    }

    #[test]
    fn test_generate_junior_prompt_with_evidence() {
        let probes = vec!["cpu.info".to_string()];
        let evidence = vec![
            ProbeSummary::new(
                "cpu.info",
                r#"{"threads_total":32,"physical_cores":24,"model":"AMD Ryzen"}"#,
            ),
        ];
        let prompt = generate_junior_prompt_v80("How many CPU cores?", &probes, &evidence);

        assert!(prompt.contains("EVIDENCE:"));
        assert!(prompt.contains("cpu.info:"));
        assert!(prompt.contains("threads_total"));
    }

    #[test]
    fn test_system_prompt_is_minimal() {
        // Razorback prompt should be much shorter than the default
        assert!(LLM_A_SYSTEM_PROMPT_V80.len() < 1000);
    }
}
