//! v0.85.0 Senior Prompt - Audit Only, No Creative Reasoning
//!
//! Target: <4KB total prompt size
//! Senior only validates, never creates new content

/// v0.85.0 Senior System Prompt - Pure auditor
pub const LLM_B_SYSTEM_PROMPT_V85: &str = r#"SENIOR v85. AUDITOR ONLY. JSON response.

TASK: Validate Junior's answer against evidence.

OUTPUT (strict JSON):
{"verdict":"approve"|"fix"|"refuse","fix":"..."|null,"evidence":0-100,"coverage":0-100,"reasoning":0-100}

VERDICTS:
- approve: Answer matches evidence, scores >=80%
- fix: Minor error, provide correction in "fix" field
- refuse: Major error, fabrication, or insufficient evidence

THRESHOLDS:
- evidence >=80: Answer is backed by probe output
- coverage >=90: Answer addresses the question fully
- reasoning >=90: Logic is sound, no contradictions

RULES:
1. NO creative additions
2. Only verify against provided evidence
3. Reject if ANY score <80%
4. One-shot decision"#;

/// Generate v0.85.0 Senior prompt
pub fn generate_senior_prompt_v85(
    question: &str,
    junior_answer: &str,
    evidence: &str,
    junior_score: u32,
) -> String {
    let mut prompt = format!("Q:{}\n", question);
    prompt.push_str(&format!("JUNIOR_ANSWER:{}\n", junior_answer));
    prompt.push_str(&format!("JUNIOR_SCORE:{}\n", junior_score));

    // Truncate evidence to keep prompt small
    let ev = if evidence.len() > 1500 {
        format!("{}...", &evidence[..1500])
    } else {
        evidence.to_string()
    };
    prompt.push_str(&format!("EVIDENCE:{}\n", ev));

    prompt.push_str("JSON:");
    prompt
}

/// Parse Senior's JSON response
#[derive(Debug, Clone)]
pub struct SeniorResponseV85 {
    pub verdict: String,
    pub fix: Option<String>,
    pub evidence: f64,
    pub coverage: f64,
    pub reasoning: f64,
}

impl SeniorResponseV85 {
    /// Parse from JSON string
    pub fn parse(json: &str) -> Option<Self> {
        // Try to extract JSON from response
        let json_str = if json.contains('{') {
            let start = json.find('{')?;
            let end = json.rfind('}')? + 1;
            &json[start..end]
        } else {
            json
        };

        // Parse with serde
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(json_str) {
            return Some(Self {
                verdict: v.get("verdict")?.as_str()?.to_string(),
                fix: v.get("fix").and_then(|f| f.as_str()).map(|s| s.to_string()),
                evidence: v.get("evidence").and_then(|e| e.as_f64()).unwrap_or(0.0) / 100.0,
                coverage: v.get("coverage").and_then(|c| c.as_f64()).unwrap_or(0.0) / 100.0,
                reasoning: v.get("reasoning").and_then(|r| r.as_f64()).unwrap_or(0.0) / 100.0,
            });
        }

        // Fallback: regex parsing
        let verdict = if json.contains("approve") {
            "approve"
        } else if json.contains("fix") {
            "fix"
        } else {
            "refuse"
        };

        Some(Self {
            verdict: verdict.to_string(),
            fix: None,
            evidence: 0.0,
            coverage: 0.0,
            reasoning: 0.0,
        })
    }

    /// Check if answer passes v0.85.0 thresholds
    pub fn passes_thresholds(&self) -> bool {
        self.evidence >= 0.80 && self.coverage >= 0.90 && self.reasoning >= 0.90
    }

    /// Get overall score (minimum of all)
    pub fn overall_score(&self) -> f64 {
        self.evidence.min(self.coverage).min(self.reasoning)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v85_senior_prompt_under_4kb() {
        assert!(LLM_B_SYSTEM_PROMPT_V85.len() < 4096);
    }

    #[test]
    fn test_v85_senior_has_verdicts() {
        assert!(LLM_B_SYSTEM_PROMPT_V85.contains("approve"));
        assert!(LLM_B_SYSTEM_PROMPT_V85.contains("fix"));
        assert!(LLM_B_SYSTEM_PROMPT_V85.contains("refuse"));
    }

    #[test]
    fn test_parse_senior_response() {
        let json = r#"{"verdict":"approve","fix":null,"evidence":95,"coverage":92,"reasoning":98}"#;
        let resp = SeniorResponseV85::parse(json).unwrap();
        assert_eq!(resp.verdict, "approve");
        assert!((resp.evidence - 0.95).abs() < 0.01);
        assert!(resp.passes_thresholds());
    }

    #[test]
    fn test_parse_senior_fail_thresholds() {
        let json = r#"{"verdict":"approve","evidence":70,"coverage":92,"reasoning":98}"#;
        let resp = SeniorResponseV85::parse(json).unwrap();
        assert!(!resp.passes_thresholds()); // evidence < 80
    }
}
