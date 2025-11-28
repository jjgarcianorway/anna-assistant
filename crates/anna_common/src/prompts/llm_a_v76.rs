//! Junior (LLM-A) Prompts for v0.76.0 - Minimal Planner
//!
//! v0.76.0 BREAKTHROUGH: Radically minimal prompt for 4B models.
//!
//! The problem was NOT hardware or timeouts.
//! The problem was Junior drowning in 500+ lines of context.
//! A 4B model cannot process huge prompts - it needs minimal context.
//!
//! This prompt:
//! - ~40 lines total (vs 500+ before)
//! - Strict JSON-only output
//! - No explanations, no examples, no filler
//! - Response time: ~1-2 seconds (vs 70-150 before)

use crate::answer_engine::{AvailableProbe, ProbeEvidenceV10};

/// Minimal system prompt for Junior - v0.76.0
///
/// Key changes from v0.14.0:
/// - Removed role description (unnecessary)
/// - Removed probe usage guides (Junior doesn't need to know HOW to use probes)
/// - Removed evidence discipline (that's Senior's job)
/// - Removed scoring guidelines (Senior handles that)
/// - Removed style guide (not needed)
/// - Just: what you are, what you output, format
pub const LLM_A_SYSTEM_PROMPT_V76: &str = r#"You are Anna Junior. Create plans to answer Linux questions.

OUTPUT FORMAT (strict JSON):
{
  "plan": {
    "intent": "hardware|storage|system|meta|unsupported",
    "probe_requests": [{"probe_id": "xxx", "reason": "why"}],
    "can_answer_without_more_probes": false
  },
  "draft_answer": null,
  "needs_more_probes": true,
  "refuse_to_answer": false
}

RULES:
- Request 0-3 probes from available list
- If you have evidence: set can_answer_without_more_probes=true, needs_more_probes=false, provide draft_answer
- draft_answer format: {"text": "answer", "citations": [{"probe_id": "xxx"}]}
- No prose, only JSON
"#;

/// Generate minimal Junior prompt for v0.76.0
///
/// Total prompt size: ~15 lines including question and probe list
/// (vs 500+ lines in v0.14.0)
pub fn generate_junior_prompt_v76(
    question: &str,
    available_probes: &[AvailableProbe],
    evidence: &[ProbeEvidenceV10],
    iteration: usize,
) -> String {
    // Extract just probe IDs - no descriptions, no pretty JSON
    let probes: Vec<&str> = available_probes.iter().map(|p| p.probe_id.as_str()).collect();
    let probes_str = probes.join(", ");

    if evidence.is_empty() {
        format!(
            "PROBES: {probes_str}\n\
             QUESTION: {question}\n\
             \n\
             Select probes. JSON only."
        )
    } else {
        // Compact evidence summary - just probe_id: raw (truncated)
        let evidence_lines: Vec<String> = evidence.iter().map(|e| {
            let raw = e.raw.as_ref().map(|r| {
                if r.len() > 500 {
                    format!("{}...", &r[..500])
                } else {
                    r.clone()
                }
            }).unwrap_or_else(|| "(no output)".to_string());
            format!("{}: {}", e.probe_id, raw)
        }).collect();
        let evidence_str = evidence_lines.join("\n");

        let urgency = if iteration >= 2 {
            "ANSWER NOW with draft_answer."
        } else {
            "Answer now."
        };

        format!(
            "PROBES: {probes_str}\n\
             QUESTION: {question}\n\
             \n\
             EVIDENCE:\n{evidence_str}\n\
             \n\
             {urgency} JSON only."
        )
    }
}

/// The 6 real probes - for reference
pub const PROBE_LIST_V76: &[&str] = &[
    "cpu.info",
    "mem.info",
    "disk.lsblk",
    "hardware.gpu",
    "drivers.gpu",
    "hardware.ram",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::answer_engine::{EvidenceStatus, ProbeCost};

    fn make_probe(id: &str) -> AvailableProbe {
        AvailableProbe {
            probe_id: id.to_string(),
            description: "test".to_string(),
            cost: ProbeCost::Cheap,
        }
    }

    fn make_evidence(id: &str, raw: &str) -> ProbeEvidenceV10 {
        ProbeEvidenceV10 {
            probe_id: id.to_string(),
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            status: EvidenceStatus::Ok,
            command: "test".to_string(),
            raw: Some(raw.to_string()),
            parsed: None,
        }
    }

    #[test]
    fn test_system_prompt_is_minimal() {
        // v0.76.0: System prompt should be under 50 lines
        let lines = LLM_A_SYSTEM_PROMPT_V76.lines().count();
        assert!(lines < 50, "System prompt too long: {} lines", lines);
    }

    #[test]
    fn test_system_prompt_has_json_format() {
        assert!(LLM_A_SYSTEM_PROMPT_V76.contains("plan"));
        assert!(LLM_A_SYSTEM_PROMPT_V76.contains("probe_requests"));
        assert!(LLM_A_SYSTEM_PROMPT_V76.contains("needs_more_probes"));
    }

    #[test]
    fn test_generated_prompt_is_compact() {
        let probes = vec![make_probe("cpu.info"), make_probe("mem.info")];
        let prompt = generate_junior_prompt_v76(
            "How much RAM do I have?",
            &probes,
            &[],
            1,
        );
        let lines = prompt.lines().count();
        assert!(lines < 10, "Generated prompt too long: {} lines", lines);
        assert!(prompt.contains("cpu.info, mem.info"));
    }

    #[test]
    fn test_generated_prompt_with_evidence() {
        let probes = vec![make_probe("mem.info")];
        let evidence = vec![make_evidence("mem.info", "MemTotal: 32000000 kB")];
        let prompt = generate_junior_prompt_v76(
            "How much RAM?",
            &probes,
            &evidence,
            2,
        );
        assert!(prompt.contains("EVIDENCE"));
        assert!(prompt.contains("ANSWER NOW"));
        assert!(prompt.contains("MemTotal"));
    }

    #[test]
    fn test_prompt_truncates_long_evidence() {
        let probes = vec![make_probe("disk.lsblk")];
        let long_output = "x".repeat(1000);
        let evidence = vec![make_evidence("disk.lsblk", &long_output)];
        let prompt = generate_junior_prompt_v76("test", &probes, &evidence, 1);
        // Should truncate to 500 chars + "..."
        assert!(prompt.len() < 700);
    }
}
