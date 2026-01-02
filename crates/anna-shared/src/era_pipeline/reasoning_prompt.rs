//! Reasoning prompt building and parsing.

use super::evidence::EvidenceBundle;
use super::reasoning_types::{truncate, ReasoningEvidence, ReasoningOutput};

/// Specialist prompt for reasoning-only mode.
pub const REASONING_SYSTEM_PROMPT: &str = r#"You are a REASONING specialist. You do NOT answer users directly.

INPUT: Intent + EvidenceBundle
OUTPUT: Structured JSON reasoning

RULES:
1. ONLY use facts from the EvidenceBundle
2. NEVER introduce facts not in the bundle
3. NEVER infer beyond the evidence
4. NEVER expand scope of the question
5. Output ONLY valid JSON

OUTPUT SCHEMA:
{
  "case_id": "string",
  "can_answer": true|false,
  "reasoning": "short factual statement (max 200 chars)",
  "derived": {
    "root_cause": "string or null",
    "metric": "string or null"
  },
  "confidence": 0.0-1.0,
  "requires": ["fact_name"]  // only if can_answer=false
}

If evidence is insufficient, set can_answer=false and list required facts."#;

/// Build reasoning prompt for a case.
pub fn build_reasoning_prompt(intent: &str, evidence: &EvidenceBundle) -> String {
    let evidence_json = ReasoningEvidence::from_bundle(evidence);
    let evidence_str = serde_json::to_string_pretty(&evidence_json).unwrap_or_default();

    format!(
        "INTENT: {}\n\nEVIDENCE:\n{}\n\nProduce reasoning JSON.",
        intent, evidence_str
    )
}

/// Parse reasoning output from JSON.
pub fn parse_reasoning_output(raw: &str) -> Result<ReasoningOutput, String> {
    // Try direct parse
    if let Ok(output) = serde_json::from_str::<ReasoningOutput>(raw.trim()) {
        return Ok(output);
    }

    // Try to extract JSON from mixed content
    if let Some(json_start) = raw.find('{') {
        if let Some(json_end) = raw.rfind('}') {
            let json_str = &raw[json_start..=json_end];
            if let Ok(output) = serde_json::from_str::<ReasoningOutput>(json_str) {
                return Ok(output);
            }
        }
    }

    Err(format!(
        "Failed to parse reasoning output: {}",
        truncate(raw, 100)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_reasoning_output() {
        let json = r#"{"case_id":"DSK-0127","can_answer":true,"reasoning":"Test","derived":{},"confidence":0.9,"requires":[]}"#;
        let output = parse_reasoning_output(json).unwrap();
        assert_eq!(output.case_id, "DSK-0127");
        assert!(output.can_answer);
    }
}
