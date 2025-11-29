//! Junior (LLM-A) Prompts for v0.83.0 - Compact Validator
//!
//! v0.83.0 Performance Focus:
//! - Validate commands and answers proposed by Anna
//! - Numeric score and explicit decision
//! - Punish fabricated claims
//! - Encourage improvements
//! - Keep answers extremely compact
//! - Target: 5 second response time

use crate::answer_engine::{AvailableProbe, ProbeEvidenceV10};

/// v0.88.0 Junior system prompt - compact, decisive, fast
/// NOTE: Probe list is passed dynamically via user prompt (P: line), NOT hardcoded here
pub const LLM_A_SYSTEM_PROMPT_V83: &str = r#"You are Junior. Validate Linux system questions. Be FAST and COMPACT.

OUTPUT (strict JSON only):
{"plan":{"probes":["probe_id"],"answer_ready":false},"draft":"answer or null","score":0,"refuse":false}

RULES:
- score 0-100 (0=refuse, 100=perfect)
- If evidence exists AND sufficient: answer_ready=true, provide draft, score>=80
- If no evidence: answer_ready=false, request probes from P: list
- Fabricated claims: score=0, refuse=true
- Only use probes from the P: list in the user message
- KEEP ANSWERS SHORT. Max 2 sentences.
"#;

/// Generate compact Junior prompt for v0.83.0
pub fn generate_junior_prompt_v83(
    question: &str,
    available_probes: &[AvailableProbe],
    evidence: &[ProbeEvidenceV10],
    iteration: usize,
) -> String {
    let probes: Vec<&str> = available_probes.iter().map(|p| p.probe_id.as_str()).collect();
    let probes_str = probes.join(",");

    if evidence.is_empty() {
        format!("Q:{question}\nP:{probes_str}\n\nSelect probes. JSON only.")
    } else {
        let evidence_lines: Vec<String> = evidence
            .iter()
            .map(|e| {
                let raw = e.raw.as_ref().map(|r| {
                    if r.len() > 300 { format!("{}...", &r[..300]) } else { r.clone() }
                }).unwrap_or_else(|| "(empty)".to_string());
                format!("{}:{}", e.probe_id, raw)
            })
            .collect();
        let evidence_str = evidence_lines.join("\n");

        let urgency = if iteration >= 2 { "ANSWER NOW." } else { "" };

        format!("Q:{question}\nP:{probes_str}\n\nE:\n{evidence_str}\n\n{urgency}JSON only.")
    }
}

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
    fn test_v83_prompt_is_compact() {
        // v0.83.0: System prompt should be under 600 chars (545 actual)
        assert!(
            LLM_A_SYSTEM_PROMPT_V83.len() < 600,
            "System prompt too long: {} chars",
            LLM_A_SYSTEM_PROMPT_V83.len()
        );
    }

    #[test]
    fn test_v83_prompt_has_score() {
        assert!(LLM_A_SYSTEM_PROMPT_V83.contains("score"));
        assert!(LLM_A_SYSTEM_PROMPT_V83.contains("0-100"));
    }

    #[test]
    fn test_v83_generated_prompt_compact() {
        let probes = vec![make_probe("cpu.info"), make_probe("mem.info")];
        let prompt = generate_junior_prompt_v83("How much RAM?", &probes, &[], 1);
        assert!(prompt.len() < 100, "Generated prompt too long: {} chars", prompt.len());
    }

    #[test]
    fn test_v83_generated_prompt_with_evidence() {
        let probes = vec![make_probe("mem.info")];
        let evidence = vec![make_evidence("mem.info", "MemTotal: 32000000 kB")];
        let prompt = generate_junior_prompt_v83("How much RAM?", &probes, &evidence, 2);
        assert!(prompt.contains("E:"));
        assert!(prompt.contains("ANSWER NOW"));
    }
}
